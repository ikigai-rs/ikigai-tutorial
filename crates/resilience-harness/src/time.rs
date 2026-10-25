//! The harness's time: a clock the reader advances, and a timer that waits on it.
//!
//! A breaker's cooldown, a timeout's budget and a retry's wait are all TIME, and a page that
//! ran them on the wall clock would answer differently for every reader, and for every run
//! of a test: a cell whose answer depends on timing is a flaky gate. So the harness has a
//! clock of its own, [`HarnessClock`], which moves only when something says so, and a timer,
//! [`VirtualTimer`], that "waits" by moving that clock.
//!
//! # How the timer waits
//!
//! A wait is over when it is the EARLIEST one pending, and finishing it moves the clock to
//! its deadline. That is the whole rule, and it is what makes a race between two waits come
//! out the way the same race would on a real clock, in no real time at all:
//!
//! - `Timeout` races the upstream against its budget. In `slow` mode the upstream waits 10s
//!   and the budget is 2s; both waits are pending, the budget's is earlier, so it finishes
//!   first, the clock moves 2s, and the timeout fires. The upstream's wait is dropped
//!   unfinished, as the real one would be.
//! - `Retry` waits out a `RetryAfter` delay alone: it is the earliest (the only) wait, so it
//!   finishes and the clock moves by the delay.
//!
//! A wait never finishes on its first poll. It registers its deadline, asks to be polled
//! again and returns pending, so that every wait started in the same poll of a task (both
//! sides of a `select`) is registered before any of them is judged the earliest.
//!
//! This is a discrete-event clock, not `setTimeout`: the page's kernel never waits on the
//! browser at all, which is why the browser `Timer` the page could build (a `setTimeout`
//! behind the `spawn_local` and `oneshot` bridge) is not needed here.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use ikigai_core::{Clock, Time};

/// The instant the harness clock starts at: 2026-10-06T09:00:00Z, the instant the book's
/// stories start (`ledger_host::STORY_START`), in milliseconds. Only differences are ever
/// shown, so the choice is cosmetic; a real instant keeps every stamp a plausible time.
pub const START: u64 = 1_791_277_200_000;

/// A clock that moves only when it is told to: by the reader ([`advance`](Self::advance)),
/// or by a [`VirtualTimer`] finishing a wait.
#[derive(Debug)]
pub struct HarnessClock(AtomicU64);

impl Default for HarnessClock {
    fn default() -> Self {
        HarnessClock(AtomicU64::new(START))
    }
}

impl HarnessClock {
    /// Move the clock forward by `by`.
    pub fn advance(&self, by: Duration) {
        self.0.fetch_add(millis(by), Ordering::SeqCst);
    }

    /// Move the clock to `at`, if that is later than now. Never backward.
    fn advance_to(&self, at: u64) {
        self.0.fetch_max(at, Ordering::SeqCst);
    }

    /// How far the clock has moved since it started.
    pub fn elapsed(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::SeqCst) - START)
    }
}

impl Clock for HarnessClock {
    fn now(&self) -> Time {
        Time::from_millis(self.0.load(Ordering::SeqCst))
    }
}

/// A duration in whole milliseconds.
fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

/// A duration the way the harness says it: whole seconds, else milliseconds.
pub fn span(duration: Duration) -> String {
    if duration.subsec_millis() == 0 {
        format!("{}s", duration.as_secs())
    } else {
        format!("{}ms", duration.as_millis())
    }
}

/// The waits pending on a [`VirtualTimer`], by registration order.
#[derive(Default)]
struct Pending {
    next: u64,
    /// id → (deadline in milliseconds, the waker of the task polling it)
    waits: BTreeMap<u64, (u64, Waker)>,
}

impl Pending {
    /// The id of the earliest wait: the soonest deadline, the first registered on a tie.
    fn earliest(&self) -> Option<u64> {
        self.waits
            .iter()
            .min_by_key(|(id, (deadline, _))| (*deadline, **id))
            .map(|(id, _)| *id)
    }

    /// Wake every pending wait, so each looks again at whether it is now the earliest.
    fn wake_all(&self) {
        for (_, waker) in self.waits.values() {
            waker.wake_by_ref();
        }
    }
}

/// A [`Timer`](ikigai_throttle::Timer) on the [`HarnessClock`]: see the module docs.
pub struct VirtualTimer {
    clock: Arc<HarnessClock>,
    pending: Mutex<Pending>,
}

impl VirtualTimer {
    /// A timer that waits on `clock`.
    pub fn new(clock: Arc<HarnessClock>) -> Self {
        VirtualTimer {
            clock,
            pending: Mutex::default(),
        }
    }

    /// A future that is ready once `delay` has passed on the harness clock.
    pub fn wait(&self, delay: Duration) -> Wait<'_> {
        Wait {
            timer: self,
            delay,
            id: None,
        }
    }

    fn pending(&self) -> std::sync::MutexGuard<'_, Pending> {
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[async_trait::async_trait]
impl ikigai_throttle::Timer for VirtualTimer {
    async fn sleep(&self, delay: Duration) {
        self.wait(delay).await
    }
}

/// One wait on a [`VirtualTimer`].
pub struct Wait<'a> {
    timer: &'a VirtualTimer,
    delay: Duration,
    /// Set on the first poll, when the wait registers its deadline.
    id: Option<u64>,
}

impl Future for Wait<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let timer = self.timer;
        let mut pending = timer.pending();
        let Some(id) = self.id else {
            // First poll: register, and come back once every sibling wait has too.
            let id = pending.next;
            pending.next += 1;
            let deadline = timer
                .clock
                .now()
                .as_millis()
                .saturating_add(millis(self.delay));
            pending.waits.insert(id, (deadline, cx.waker().clone()));
            self.id = Some(id);
            cx.waker().wake_by_ref();
            return Poll::Pending;
        };
        if pending.earliest() == Some(id) {
            let (deadline, _) = pending.waits.remove(&id).expect("a registered wait");
            timer.clock.advance_to(deadline);
            self.id = None;
            // Another wait may be the earliest now.
            pending.wake_all();
            return Poll::Ready(());
        }
        if let Some((_, waker)) = pending.waits.get_mut(&id) {
            waker.clone_from(cx.waker());
        }
        Poll::Pending
    }
}

impl Drop for Wait<'_> {
    /// A wait dropped unfinished (the side of a race that lost) leaves the timer, and the
    /// waits behind it look again at whether one of them is now the earliest.
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let mut pending = self.timer.pending();
            pending.waits.remove(&id);
            pending.wake_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use futures::future::{select, Either};

    fn timer() -> (Arc<HarnessClock>, VirtualTimer) {
        let clock = Arc::new(HarnessClock::default());
        (Arc::clone(&clock), VirtualTimer::new(clock))
    }

    #[test]
    fn a_lone_wait_moves_the_clock_by_its_delay() {
        let (clock, timer) = timer();
        block_on(timer.wait(Duration::from_secs(1)));
        assert_eq!(clock.elapsed(), Duration::from_secs(1));
        block_on(timer.wait(Duration::from_millis(500)));
        assert_eq!(clock.elapsed(), Duration::from_millis(1_500));
    }

    /// The race `Timeout` runs: the work first, the budget second, and the earlier wins
    /// whichever is polled first.
    #[test]
    fn of_two_racing_waits_the_earlier_finishes_and_the_clock_stops_there() {
        let (clock, timer) = timer();
        let slow = Box::pin(timer.wait(Duration::from_secs(10)));
        let budget = Box::pin(timer.wait(Duration::from_secs(2)));
        let won = block_on(select(slow, budget));
        assert!(matches!(won, Either::Right(_)), "the budget is earlier");
        drop(won);
        assert_eq!(clock.elapsed(), Duration::from_secs(2));
        assert!(timer.pending().waits.is_empty(), "the loser left the timer");
    }

    #[test]
    fn the_clock_never_moves_backward() {
        let (clock, timer) = timer();
        clock.advance(Duration::from_secs(5));
        block_on(timer.wait(Duration::ZERO));
        assert_eq!(clock.elapsed(), Duration::from_secs(5));
    }

    #[test]
    fn a_span_reads_as_an_operator_writes_it() {
        assert_eq!(span(Duration::from_secs(5)), "5s");
        assert_eq!(span(Duration::ZERO), "0s");
        assert_eq!(span(Duration::from_millis(1_500)), "1500ms");
    }
}
