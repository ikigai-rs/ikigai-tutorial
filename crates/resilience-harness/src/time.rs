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
//! ## Background work
//!
//! The rule above is for ONE task at a time: a cell's line, with every wait it races inside
//! it. It holds across several tasks only while each task registers its next wait in the same
//! poll its last one finished: a task that is woken but not yet polled has no wait pending, so
//! the earliest-wait rule cannot see it, and another task's later wait finishes first and moves
//! the clock past where the first should have stopped (`a_task_that_yields_between_waits_…`
//! below shows it). So work that runs in the BACKGROUND, the calls a burst starts, never moves
//! the clock itself. A wait registered while the timer is [driven](VirtualTimer::driving) is a
//! background wait: it finishes once the clock has reached its deadline, and the harness moves
//! the clock to that deadline only when every background task has stalled
//! ([`Background::run_until`](crate::Background::run_until), a discrete-event step). A
//! foreground wait's earliest is judged among foreground waits only, so a background wait
//! never holds up a cell.
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

    /// Move the clock to `at` (milliseconds since 1970), if that is later than now. Never
    /// backward.
    pub fn advance_to(&self, at: u64) {
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

/// One pending wait: its deadline in milliseconds, the waker of the task polling it, and
/// whether it was registered by background work.
struct Pended {
    deadline: u64,
    waker: Waker,
    background: bool,
}

/// The waits pending on a [`VirtualTimer`], by registration order.
#[derive(Default)]
struct Pending {
    next: u64,
    waits: BTreeMap<u64, Pended>,
}

impl Pending {
    /// The id of the earliest FOREGROUND wait: the soonest deadline, the first registered on a
    /// tie. Background waits are not in the race: they finish on the clock, not by it.
    fn earliest(&self) -> Option<u64> {
        self.waits
            .iter()
            .filter(|(_, wait)| !wait.background)
            .min_by_key(|(id, wait)| (wait.deadline, **id))
            .map(|(id, _)| *id)
    }

    /// The soonest deadline of a background wait, if one is pending.
    fn next_background(&self) -> Option<u64> {
        self.waits
            .values()
            .filter(|wait| wait.background)
            .map(|wait| wait.deadline)
            .min()
    }

    /// Wake every pending wait, so each looks again at whether it is now the earliest.
    fn wake_all(&self) {
        for wait in self.waits.values() {
            wait.waker.wake_by_ref();
        }
    }
}

/// A [`Timer`](ikigai_throttle::Timer) on the [`HarnessClock`]: see the module docs.
pub struct VirtualTimer {
    clock: Arc<HarnessClock>,
    pending: Mutex<Pending>,
    /// Set while background work is polled: a wait registered then is a background wait.
    driving: std::sync::atomic::AtomicBool,
}

impl VirtualTimer {
    /// A timer that waits on `clock`.
    pub fn new(clock: Arc<HarnessClock>) -> Self {
        VirtualTimer {
            clock,
            pending: Mutex::default(),
            driving: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Run `poll` as background work: every wait it registers is a background wait, which
    /// finishes once the clock reaches its deadline and never moves the clock itself.
    pub fn driving<T>(&self, poll: impl FnOnce() -> T) -> T {
        let was = self.driving.swap(true, Ordering::SeqCst);
        let out = poll();
        self.driving.store(was, Ordering::SeqCst);
        out
    }

    /// Wake every background wait whose deadline the clock has reached, so the work polling
    /// it looks again: moving the clock wakes nothing by itself.
    pub fn wake_due(&self) {
        let now = self.clock.now().as_millis();
        for wait in self.pending().waits.values() {
            if wait.background && wait.deadline <= now {
                wait.waker.wake_by_ref();
            }
        }
    }

    /// The soonest deadline of a background wait, in milliseconds since 1970: where the
    /// harness moves the clock next when it runs background work forward.
    pub fn next_background(&self) -> Option<u64> {
        self.pending().next_background()
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
            let background = timer.driving.load(Ordering::SeqCst);
            pending.waits.insert(
                id,
                Pended {
                    deadline,
                    waker: cx.waker().clone(),
                    background,
                },
            );
            self.id = Some(id);
            cx.waker().wake_by_ref();
            return Poll::Pending;
        };
        let wait = pending.waits.get(&id).expect("a registered wait");
        let over = if wait.background {
            wait.deadline <= timer.clock.now().as_millis()
        } else {
            pending.earliest() == Some(id)
        };
        if over {
            let wait = pending.waits.remove(&id).expect("a registered wait");
            timer.clock.advance_to(wait.deadline);
            self.id = None;
            // Another wait may be the earliest now.
            pending.wake_all();
            return Poll::Ready(());
        }
        if let Some(wait) = pending.waits.get_mut(&id) {
            wait.waker.clone_from(cx.waker());
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

/// Work held in flight in the background: futures polled only while the timer is
/// [driven](VirtualTimer::driving), and moved forward only by [`run_until`](Self::run_until).
///
/// A tiny executor of its own: the harness's lines run on whatever the page or a test runs
/// them on, and background work must not ride that executor, or it would run (and move the
/// clock) whenever the page next yields. Here it runs exactly when the harness says.
pub struct Held<T> {
    work: Mutex<Vec<Piece<T>>>,
}

/// One piece of held work, and the waker that says it can move.
struct Piece<T> {
    future: futures::future::BoxFuture<'static, T>,
    woken: Arc<Woken>,
}

impl<T> Default for Held<T> {
    fn default() -> Self {
        Held {
            work: Mutex::default(),
        }
    }
}

/// A waker that remembers it was woken: how [`Held`] knows which work can move. (Not
/// `FuturesUnordered`: it wakes its own caller whenever it has polled every future once, so a
/// caller cannot tell from it whether anything can still move.)
struct Woken(std::sync::atomic::AtomicBool);

impl futures::task::ArcWake for Woken {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.0.store(true, Ordering::SeqCst);
    }
}

impl<T> Held<T> {
    /// Hold `work` in flight. It is not polled until the harness next runs its work.
    pub fn push(&self, work: futures::future::BoxFuture<'static, T>) {
        self.lock().push(Piece {
            future: work,
            woken: Arc::new(Woken(std::sync::atomic::AtomicBool::new(true))),
        });
    }

    /// How many pieces of work are still in flight.
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// Whether nothing is in flight.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Piece<T>>> {
        self.work
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Poll the work until none of it can move at the clock as it reads: what finished, in
    /// the order it finished (and, among pieces that finish in one round, in the order they
    /// were started).
    pub fn stall(&self, timer: &VirtualTimer) -> Vec<T> {
        let mut done = Vec::new();
        let mut work = self.lock();
        timer.driving(|| {
            while work
                .iter()
                .any(|piece| piece.woken.0.load(Ordering::SeqCst))
            {
                let mut k = 0;
                while k < work.len() {
                    if !work[k].woken.0.swap(false, Ordering::SeqCst) {
                        k += 1;
                        continue;
                    }
                    let waker = futures::task::waker(Arc::clone(&work[k].woken));
                    let mut cx = Context::from_waker(&waker);
                    match work[k].future.as_mut().poll(&mut cx) {
                        Poll::Ready(out) => {
                            done.push(out);
                            work.remove(k);
                        }
                        Poll::Pending => k += 1,
                    }
                }
            }
        });
        done
    }

    /// Run the work forward to `target` (milliseconds since 1970), one event at a time: stall
    /// it, move the clock to the soonest background deadline if that is no later than
    /// `target`, and again, then leave the clock at `target`. A piece of work that finishes at
    /// an event and lets another start (a slot freed) starts it at that event's time. What
    /// finished, in order.
    pub fn run_until(&self, timer: &VirtualTimer, target: u64) -> Vec<T> {
        let mut done = self.stall(timer);
        while let Some(next) = timer.next_background().filter(|next| *next <= target) {
            timer.clock.advance_to(next);
            timer.wake_due();
            done.extend(self.stall(timer));
        }
        timer.clock.advance_to(target);
        done
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

    /// The rule across several TASKS, each with waits of its own: every wait finishes at its
    /// deadline, in deadline order, whichever task it is in, while each task registers its next
    /// wait in the poll its last one finished.
    #[test]
    fn several_tasks_with_waits_of_their_own_finish_in_deadline_order() {
        use futures::task::LocalSpawnExt;
        let (clock, timer) = timer();
        let timer = Arc::new(timer);
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut pool = futures::executor::LocalPool::new();
        for (name, waits) in [("a", vec![3]), ("b", vec![1, 4]), ("c", vec![2])] {
            let (timer, clock, seen) = (Arc::clone(&timer), Arc::clone(&clock), Arc::clone(&seen));
            pool.spawner()
                .spawn_local(async move {
                    for (k, secs) in waits.into_iter().enumerate() {
                        timer.wait(Duration::from_secs(secs)).await;
                        seen.lock()
                            .unwrap()
                            .push(format!("{name}{k}@{}", span(clock.elapsed())));
                    }
                })
                .unwrap();
        }
        pool.run();
        assert_eq!(*seen.lock().unwrap(), ["b0@1s", "c0@2s", "a0@3s", "b1@5s"]);
        assert_eq!(clock.elapsed(), Duration::from_secs(5));
    }

    /// Where the rule stops holding, pinned so it is not forgotten: a task woken but not yet
    /// polled has no wait pending, so the earliest-wait rule cannot see it. Here `a` waits 1s,
    /// yields once, and waits 1s more: on a real clock it ends at 2s. Its yield lets `b`'s 3s
    /// wait be the earliest pending, `b` finishes first and moves the clock to 3s, and `a`
    /// ends at 4s. No cell in the harness has two tasks in the foreground (a cell is one
    /// line), which is why background work runs on the clock instead ([`Held`]).
    #[test]
    fn a_task_that_yields_between_waits_lets_a_later_wait_finish_first() {
        use futures::task::LocalSpawnExt;
        let (clock, timer) = timer();
        let timer = Arc::new(timer);
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut pool = futures::executor::LocalPool::new();
        {
            let (timer, clock, seen) = (Arc::clone(&timer), Arc::clone(&clock), Arc::clone(&seen));
            pool.spawner()
                .spawn_local(async move {
                    timer.wait(Duration::from_secs(1)).await;
                    let mut once = false;
                    futures::future::poll_fn(|cx| {
                        if once {
                            Poll::Ready(())
                        } else {
                            once = true;
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                    })
                    .await;
                    timer.wait(Duration::from_secs(1)).await;
                    seen.lock()
                        .unwrap()
                        .push(format!("a@{}", span(clock.elapsed())));
                })
                .unwrap();
        }
        {
            let (timer, clock, seen) = (Arc::clone(&timer), Arc::clone(&clock), Arc::clone(&seen));
            pool.spawner()
                .spawn_local(async move {
                    timer.wait(Duration::from_secs(3)).await;
                    seen.lock()
                        .unwrap()
                        .push(format!("b@{}", span(clock.elapsed())));
                })
                .unwrap();
        }
        pool.run();
        assert_eq!(*seen.lock().unwrap(), ["b@3s", "a@4s"]);
    }

    /// Background work, run event by event: two pieces that each wait twice, one that waits
    /// once, and a foreground wait that is never held up by them. Each finishes at its true
    /// time, and a run that stops short of a deadline leaves that work in flight.
    #[test]
    fn background_work_runs_on_the_clock_event_by_event() {
        let (clock, timer) = timer();
        let timer = Arc::new(timer);
        let held: Held<String> = Held::default();
        for (name, waits) in [("a", vec![10, 10]), ("b", vec![10, 10]), ("c", vec![15])] {
            let (timer, clock) = (Arc::clone(&timer), Arc::clone(&clock));
            held.push(Box::pin(async move {
                for secs in waits {
                    timer.wait(Duration::from_secs(secs)).await;
                }
                format!("{name}@{}", span(clock.elapsed()))
            }));
        }
        // Started: every wait registered, nothing finished, the clock where it was.
        assert!(held.stall(&timer).is_empty());
        assert_eq!(clock.elapsed(), Duration::ZERO);
        // A foreground wait is not held up by the background's, and does not finish them.
        block_on(timer.wait(Duration::from_secs(1)));
        assert_eq!(clock.elapsed(), Duration::from_secs(1));
        assert_eq!(held.len(), 3);
        let at = |secs: u64| START + secs * 1_000;
        assert_eq!(held.run_until(&timer, at(16)), ["c@15s"]);
        assert_eq!(clock.elapsed(), Duration::from_secs(16));
        // `a` and `b` registered their second waits at 10s (their first deadline, measured
        // from 0s, where they started), so they finish at 20s, not at 16s + 10s.
        assert_eq!(held.run_until(&timer, at(25)), ["a@20s", "b@20s"]);
        assert!(held.is_empty());
        assert_eq!(clock.elapsed(), Duration::from_secs(25));
    }
}
