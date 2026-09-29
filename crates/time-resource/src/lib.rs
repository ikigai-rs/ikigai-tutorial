//! The code taught by the book's chapter **Time is a resource**.
//!
//! Until this chapter every answer in the book changed only when something wrote: a `Sink`
//! cut a golden thread and whatever hung from it was computed again. Time changes with
//! nobody writing, so the kernel needs a second way to let an answer go, and it has one:
//! **expiry**. A representation can say *cache me until this instant*, and the kernel, which
//! has a clock only if its host gave it one, lets the answer go at that instant.
//!
//! The resources, designed as names before code:
//!
//! * `urn:iki:tutorial:time:now` — the time, `HH:MM`, UTC. A tiny function over the
//!   kernel's clock, cacheable until the next minute boundary.
//! * `urn:iki:tutorial:time:today` — the date, `YYYY-MM-DD`, UTC. The same, cacheable until
//!   the next midnight.
//! * `urn:iki:tutorial:time:instant` — the current minute as one instant,
//!   `YYYY-MM-DDTHH:MMZ`, read from `today` and `now` with the date read on both sides of the
//!   time, so it is never a torn reading of the two. Cacheable until the next minute, which
//!   it inherits from `now`. The spreadsheet's `NOW()` reads it.
//! * `urn:iki:tutorial:time:event:{name}` — a named instant, `YYYY-MM-DDTHH:MMZ`. The one
//!   **atom**: the only state in the chapter, set by `Sink`, cleared by `Delete`.
//! * `urn:iki:tutorial:time:until:{name}` — minutes from now until the event. A composite
//!   over `today`, `now` and the event. It declares itself cacheable with no deadline, and
//!   the kernel gives it `now`'s deadline anyway, because a composite is never fresher than
//!   the least fresh thing it read.
//! * `urn:iki:tutorial:time:template:{name}` — the chapter's HTML: `clock`, which names the
//!   resource it shows, and `face`, the markup that polls it.
//! * `urn:iki:tutorial:time:view:clock` — the `clock` template expanded by the generic
//!   composer `urn:iki:fn:compose`.
//!
//! The idea of a clock that is polled every second and computed once a minute is the one
//! the web demo's navigation clock (`ikigai-web-demo`, `urn:time:now`) has shown since
//! 2026-06-30; this is the book's own version of it, over the kernel's clock rather than
//! the browser's `Date`.
//!
//! Read the book: <https://ikigai-rs.github.io/ikigai-tutorial/applied/time.html>.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, ArgSpec, AsyncFnEndpoint, Clock, Description, EndpointSpace, Error, Fallback,
    FnEndpoint, Invocation, InvokeFuture, Iri, Kernel, ReprType, Representation, Result, Space,
    Time, UriTemplate, Verb,
};
use ikigai_fn::ComposeOver;
use ikigai_vocab::TurtleRenderer;

/// `text/plain; charset=utf-8` as a [`ReprType`].
fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

/// The same media type as a string, for a [`Description`].
const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";

/// `text/html; charset=utf-8` as a [`ReprType`].
fn text_html_utf8() -> ReprType {
    ReprType::new("text/html").with_param("charset", "utf-8")
}

/// The same media type as a string, for a [`Description`].
const TEXT_HTML_UTF8: &str = "text/html;charset=utf-8";

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

// ANCHOR: names
/// The time of day, `HH:MM`, UTC — until the next minute.
pub const NOW: &str = "urn:iki:tutorial:time:now";

/// The date, `YYYY-MM-DD`, UTC — until the next midnight.
pub const TODAY: &str = "urn:iki:tutorial:time:today";

/// The current minute as one instant, `YYYY-MM-DDTHH:MMZ`, never a torn reading of `today`
/// and `now` — until the next minute.
pub const INSTANT: &str = "urn:iki:tutorial:time:instant";

/// A named instant, `YYYY-MM-DDTHH:MMZ`: the chapter's one piece of state.
pub const EVENT: &str = "urn:iki:tutorial:time:event:{name}";

/// Whole minutes from now until the event named `name`; negative once it has passed.
pub const UNTIL: &str = "urn:iki:tutorial:time:until:{name}";

/// A piece of the chapter's HTML.
pub const TEMPLATE: &str = "urn:iki:tutorial:time:template:{name}";

/// The clock as HTML: the `clock` template, composed.
pub const VIEW_CLOCK: &str = "urn:iki:tutorial:time:view:clock";
// ANCHOR_END: names

/// The name of the event `name`.
pub fn event_name(name: &str) -> String {
    format!("urn:iki:tutorial:time:event:{name}")
}

/// The name of the countdown to the event `name`.
pub fn until_name(name: &str) -> String {
    format!("urn:iki:tutorial:time:until:{name}")
}

/// The name of the template `name`.
pub fn template_name(name: &str) -> String {
    format!("urn:iki:tutorial:time:template:{name}")
}

const MINUTE_MS: u64 = 60_000;
const MINUTES_PER_DAY: i64 = 24 * 60;
const DAY_MS: u64 = 24 * 60 * MINUTE_MS;

// ANCHOR: clock
/// The kernel's time, or a refusal that says why there is none.
///
/// [`Invocation::now`] is the kernel's clock as this request should see it, and a kernel
/// has one only when its host injected one with `Kernel::with_clock`. Without one there is
/// no honest answer to "what time is it", so there is none: the endpoint refuses rather
/// than read the wall clock behind the host's back.
fn clock(inv: &Invocation<'_>) -> Result<Time> {
    inv.now().ok_or_else(|| {
        Error::Endpoint(
            "this kernel has no clock: its host did not give it one (Kernel::with_clock)"
                .to_string(),
        )
    })
}
// ANCHOR_END: clock

// ANCHOR: now
/// `time-now`: the time of day, `HH:MM`, UTC.
///
/// Cacheable until the next minute boundary, and not a millisecond longer: the answer is
/// right for exactly that long. A caller may ask every second; the kernel answers from its
/// cache until the deadline and runs this again once it has passed.
pub fn now() -> FnEndpoint {
    FnEndpoint::new("time-now", |inv: &Invocation<'_>| {
        let minute = clock(inv)?.as_millis() / MINUTE_MS;
        let text = format!("{:02}:{:02}", (minute / 60) % 24, minute % 60);
        let next_minute = Time::from_millis((minute + 1) * MINUTE_MS);
        Ok(Representation::new(text_plain_utf8(), text.into_bytes()).cacheable_until(next_minute))
    })
    .with_description(
        Description::new("time-now")
            .title("Now")
            .summary("The time of day, HH:MM, UTC, by the kernel's clock. Cacheable until the next minute.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: now

// ANCHOR: today
/// `time-today`: the date, `YYYY-MM-DD`, UTC, cacheable until the next midnight.
pub fn today() -> FnEndpoint {
    FnEndpoint::new("time-today", |inv: &Invocation<'_>| {
        let day = clock(inv)?.as_millis() / DAY_MS;
        let next_midnight = Time::from_millis((day + 1) * DAY_MS);
        Ok(
            Representation::new(text_plain_utf8(), date_text(day as i64).into_bytes())
                .cacheable_until(next_midnight),
        )
    })
    .with_description(
        Description::new("time-today")
            .title("Today")
            .summary("The date, YYYY-MM-DD, UTC, by the kernel's clock. Cacheable until the next midnight.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: today

// ANCHOR: as_of_doors
/// The time chapter's doors for a temporal corridor: `now` and `today` at the instant the
/// corridor is pinned to, and cacheable for as long as the corridor, since a "then" does not
/// change. (The live `now` and `today` would answer the same text, from the same pinned clock,
/// but with the next minute and the next midnight as deadlines, and those have already passed
/// by the kernel's own clock, so nothing built on them would ever be cached.)
///
/// ⚠ Bind these only in a corridor whose name is its instant (`Scope::with_named_at`). At the
/// root they would cache the first minute they were asked forever.
pub fn as_of_doors() -> EndpointSpace {
    EndpointSpace::new()
        .bind(
            template(NOW),
            pinned("time-now-then", |minute| {
                format!("{:02}:{:02}", (minute / 60) % 24, minute % 60)
            }),
        )
        .bind(
            template(TODAY),
            pinned("time-today-then", |minute| {
                date_text(minute.div_euclid(MINUTES_PER_DAY))
            }),
        )
}

/// A door that answers `text` of the corridor's minute, cacheable with no deadline.
fn pinned(id: &'static str, text: fn(i64) -> String) -> FnEndpoint {
    FnEndpoint::new(id, move |inv: &Invocation<'_>| {
        let minute = (clock(inv)?.as_millis() / MINUTE_MS) as i64;
        Ok(Representation::new(text_plain_utf8(), text(minute).into_bytes()).cacheable())
    })
    .with_description(
        Description::new(id)
            .title("The time, pinned")
            .summary("The time at the instant this corridor is pinned to; cacheable, since a then does not change.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: as_of_doors

// ---------------------------------------------------------------------------------------
// The calendar: days since 1970-01-01 to a civil date and back, proleptic Gregorian.
// Howard Hinnant's `civil_from_days` / `days_from_civil`, which are exact for every date
// a four-digit year can spell. No time zones: this chapter's time is UTC, and a zone is
// context, which is a later chapter's subject.
// ---------------------------------------------------------------------------------------

/// The civil date of day `z` (days since 1970-01-01).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// The day (days since 1970-01-01) of a civil date.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `YYYY-MM-DD` for day `day`.
fn date_text(day: i64) -> String {
    let (y, m, d) = civil_from_days(day);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `YYYY-MM-DDTHH:MMZ` for the minute `minute` (minutes since 1970-01-01T00:00Z).
pub fn instant_text(minute: i64) -> String {
    let (day, of_day) = (
        minute.div_euclid(MINUTES_PER_DAY),
        minute.rem_euclid(MINUTES_PER_DAY),
    );
    format!("{}T{:02}:{:02}Z", date_text(day), of_day / 60, of_day % 60)
}

/// Every character of `text` a digit, and its value.
fn digits(text: &str) -> Option<i64> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// The day of `YYYY-MM-DD`, if that is a real date spelled that way.
fn parse_date(text: &str) -> Option<i64> {
    let (y, m, d) = match text.split('-').collect::<Vec<_>>().as_slice() {
        [y, m, d] if y.len() == 4 && m.len() == 2 && d.len() == 2 => {
            (digits(y)?, digits(m)?, digits(d)?)
        }
        _ => return None,
    };
    if !(1..=12).contains(&m) || d < 1 {
        return None;
    }
    let day = days_from_civil(y, m, d);
    // A day past the end of its month (`02-30`) lands in the next month: refuse it by
    // asking whether the date comes back spelled the same.
    (date_text(day) == text).then_some(day)
}

/// The minute of the day of `HH:MM`, if it is one spelled that way.
fn parse_time(text: &str) -> Option<i64> {
    match text.split(':').collect::<Vec<_>>().as_slice() {
        [h, m] if h.len() == 2 && m.len() == 2 => {
            let (h, m) = (digits(h)?, digits(m)?);
            (h < 24 && m < 60).then_some(h * 60 + m)
        }
        _ => None,
    }
}

/// The minute of `YYYY-MM-DDTHH:MMZ`, if it is one spelled exactly that way.
pub fn parse_instant(text: &str) -> Option<i64> {
    let (date, time) = text.strip_suffix('Z')?.split_once('T')?;
    Some(parse_date(date)? * MINUTES_PER_DAY + parse_time(time)?)
}

/// `text` as an `i64`, reading it from a resource this crate itself serves.
fn read_back(name: &str, text: &str, parsed: Option<i64>) -> Result<i64> {
    parsed.ok_or_else(|| {
        Error::Endpoint(format!(
            "{name} answered `{text}`, which is not its own spelling"
        ))
    })
}

/// The text of the resource `name`, read through the kernel.
async fn text_of(inv: &Invocation<'_>, name: &str) -> Result<String> {
    let iri = Iri::parse(name).map_err(|e| Error::Endpoint(format!("the name {name}: {e}")))?;
    Ok(String::from_utf8_lossy(&inv.source(&iri).await?.bytes).into_owned())
}

// ANCHOR: this_minute
/// The current minute as one instant, read from `today` and `now` through the kernel.
///
/// Two resources, two deadlines, two reads: `today` can be read on one side of midnight and
/// `now` on the other, and the pair would name a minute a day away from the real one. So
/// the date is read on both sides of the time. If it is the same both times, the time was
/// read on that date; if it is not, midnight fell between the reads, and one more try
/// cannot meet another midnight.
async fn this_minute(inv: &Invocation<'_>) -> Result<i64> {
    for _ in 0..2 {
        let date = text_of(inv, TODAY).await?;
        let time = text_of(inv, NOW).await?;
        if text_of(inv, TODAY).await? == date {
            let day = read_back(TODAY, &date, parse_date(&date))?;
            let of_day = read_back(NOW, &time, parse_time(&time))?;
            return Ok(day * MINUTES_PER_DAY + of_day);
        }
    }
    Err(Error::Endpoint(
        "the date changed twice while one minute was read".to_string(),
    ))
}
// ANCHOR_END: this_minute

// ANCHOR: instant
/// `time-instant`: the current minute as one instant, `YYYY-MM-DDTHH:MMZ`.
///
/// The guarded reading of `this_minute`, given a name, so that anything wanting "the date
/// and the time" reads one resource rather than two and cannot tear them. Declared
/// `.cacheable()` with no deadline; it read `now`, so the kernel caches it until the next
/// minute, and not a millisecond longer.
pub fn instant() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("time-instant", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let minute = this_minute(inv).await?;
            Ok(
                Representation::new(text_plain_utf8(), instant_text(minute).into_bytes())
                    .cacheable(),
            )
        })
    })
    .with_description(
        Description::new("time-instant")
            .title("Instant")
            .summary(
                "The current minute as one instant, YYYY-MM-DDTHH:MMZ, UTC: the date and the \
                 time read so that they agree. Cacheable until the next minute.",
            )
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: instant

// ANCHOR: store
/// The events that have been set, by name, each at its minute.
#[derive(Debug, Default)]
pub struct EventStore {
    events: Mutex<BTreeMap<String, i64>>,
}
// ANCHOR_END: store

/// An event's name: a lower-case letter, then lower-case letters, digits and `-`. One
/// spelling per event, so one golden thread per event.
fn event_binding(inv: &Invocation<'_>) -> Result<String> {
    let name = inv
        .bindings
        .get("name")
        .ok_or_else(|| Error::MissingArgument("name".to_string()))?;
    let spelled = name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !spelled {
        return Err(Error::InvalidArgument {
            name: "name".to_string(),
            detail: format!(
                "`{name}` is not an event name: a lower-case letter, then lower-case letters, digits and - (e.g. tea, launch-2)"
            ),
        });
    }
    Ok(name.to_string())
}

fn name_input() -> ArgSpec {
    ArgSpec::new("name")
        .summary("the event's name: lower-case letters, digits and -")
        .class(XSD_STRING)
        .binding()
}

// ANCHOR: event
/// `time-event`: a named instant, held in memory.
///
/// `Source` answers the instant, `YYYY-MM-DDTHH:MMZ`, `.cacheable()` with no deadline: an
/// event does not move until somebody moves it, and the kernel cuts the event's thread
/// when somebody does. An event nobody has set is `NotFound`.
///
/// `Sink` sets it, from `content`: an instant in that one spelling, or `+N`, meaning N
/// minutes from now, read through the kernel from `today` and `now`. Either way the event
/// holds an instant, never "ninety minutes from whenever you ask", and the answer is the
/// instant it now holds. `Delete` clears it.
pub fn event(store: Arc<EventStore>) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("time-event", move |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        let store = Arc::clone(&store);
        Box::pin(async move {
            let name = event_binding(inv)?;
            match inv.request.verb {
                Verb::Sink => {
                    let content = inv.inline_str("content")?.trim();
                    let minute = match content.strip_prefix('+') {
                        Some(ahead) => this_minute(inv).await? + minutes_ahead(ahead)?,
                        None => parse_instant(content).ok_or_else(|| Error::InvalidArgument {
                            name: "content".to_string(),
                            detail: format!(
                                "`{content}` is not an instant: YYYY-MM-DDTHH:MMZ (e.g. 2026-09-29T16:00Z), or +N minutes from now"
                            ),
                        })?,
                    };
                    store.events.lock().expect("the event store's lock").insert(name, minute);
                    Ok(Representation::new(text_plain_utf8(), instant_text(minute).into_bytes()))
                }
                Verb::Delete => {
                    store.events.lock().expect("the event store's lock").remove(&name);
                    Ok(Representation::new(text_plain_utf8(), b"ok".to_vec()))
                }
                _ => {
                    let minute = store.events.lock().expect("the event store's lock").get(&name).copied();
                    match minute {
                        Some(minute) => Ok(Representation::new(
                            text_plain_utf8(),
                            instant_text(minute).into_bytes(),
                        )
                        .cacheable()),
                        None => Err(Error::NotFound(format!("no event named `{name}` has been set"))),
                    }
                }
            }
        })
    })
    .with_description(
        Description::new("time-event")
            .title("Event")
            .summary("A named instant: read it, set it, or clear it.")
            .verb(Verb::Meta)
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("the event's instant, YYYY-MM-DDTHH:MMZ; NotFound if it has not been set")
                    .input(name_input())
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary("set the event, and answer the instant it now holds")
                    .input(name_input())
                    .input(
                        ArgSpec::new("content")
                            .summary("YYYY-MM-DDTHH:MMZ, or +N for N minutes from now")
                            .class(XSD_STRING),
                    )
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Delete)
                    .summary("clear the event")
                    .input(name_input())
                    .output(TEXT_PLAIN_UTF8),
            ),
    )
}
// ANCHOR_END: event

/// The `N` of `+N`: a whole number of minutes, in its one plain spelling, at most a year.
fn minutes_ahead(text: &str) -> Result<i64> {
    let refuse = || Error::InvalidArgument {
        name: "content".to_string(),
        detail: format!(
            "`+{text}` is not +N minutes: N is a whole number, 0 to 525600, with no leading zeros"
        ),
    };
    let n = digits(text).ok_or_else(refuse)?;
    if (text.len() > 1 && text.starts_with('0')) || n > 525_600 {
        return Err(refuse());
    }
    Ok(n)
}

// ANCHOR: until
/// `time-until`: whole minutes from now until the event, negative once it has passed.
///
/// A composite over three names, all read through the kernel: the event, `today` and
/// `now`. It declares itself `.cacheable()`, with no deadline, and says nothing else about
/// time. The kernel does the rest: a result is never fresher than its least fresh
/// dependency, so this one is cached until `now`'s deadline, the next minute, and cut
/// early by the event's golden thread when the event is moved.
pub fn until() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("time-until", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let name = event_binding(inv)?;
            let at = text_of(inv, &event_name(&name)).await?;
            let at = read_back(&event_name(&name), &at, parse_instant(&at))?;
            let minutes = at - this_minute(inv).await?;
            Ok(
                Representation::new(text_plain_utf8(), minutes.to_string().into_bytes())
                    .cacheable(),
            )
        })
    })
    .with_description(
        Description::new("time-until")
            .title("Until")
            .summary("Whole minutes from now until the event; negative once it has passed.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(name_input())
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: until

/// Every template, by the name it is resolved at: `template:{name}`. A file's final
/// newline is not part of the template.
pub const TEMPLATES: &[(&str, &str)] = &[
    ("clock", include_str!("../templates/clock.html")),
    ("face", include_str!("../templates/face.html")),
];

/// `time-template`: the template `name`, as `text/html`, computed once.
pub fn templates() -> FnEndpoint {
    FnEndpoint::new("time-template", |inv: &Invocation<'_>| {
        let name = inv
            .bindings
            .get("name")
            .ok_or_else(|| Error::MissingArgument("name".to_string()))?;
        let (_, text) = TEMPLATES
            .iter()
            .find(|(known, _)| *known == name)
            .ok_or_else(|| Error::NotFound(format!("there is no template named `{name}`")))?;
        let text = text.strip_suffix('\n').unwrap_or(text);
        Ok(Representation::new(text_html_utf8(), text.as_bytes().to_vec()).cacheable())
    })
    .with_description(
        Description::new("time-template")
            .title("Template")
            .summary("A piece of the chapter's HTML.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("name")
                    .summary("which template")
                    .class(XSD_STRING)
                    .one_of(TEMPLATES.iter().map(|(name, _)| *name))
                    .binding(),
            )
            .output(TEXT_HTML_UTF8),
    )
}

// ANCHOR: view_clock
/// `urn:iki:tutorial:time:view:clock`: the `clock` template, bound at a name of its own.
///
/// The template names the resource it shows (`$h{urn:iki:tutorial:time:now}`), and compose
/// resolves every name it finds and splices the answer in. `compose_over` is compose with
/// the template fixed, bound at the view's name, so the view has no code of its own.
pub fn view_clock() -> ComposeOver {
    ikigai_fn::compose_over(
        Iri::parse(template_name("clock")).expect("a template's name is a name"),
    )
}
// ANCHOR_END: view_clock

fn template(source: &str) -> UriTemplate {
    UriTemplate::parse(source).expect("a constant template parses")
}

// ANCHOR: space
/// Every name in the chapter, with the events in a store of their own.
pub fn space() -> EndpointSpace {
    space_over(Arc::default())
}

/// [`space`], with the events in an [`EventStore`] the caller keeps.
pub fn space_over(store: Arc<EventStore>) -> EndpointSpace {
    EndpointSpace::new()
        .bind(template(NOW), now())
        .bind(template(TODAY), today())
        .bind(template(INSTANT), instant())
        .bind(template(EVENT), event(store))
        .bind(template(UNTIL), until())
        .bind(template(TEMPLATE), templates())
        .bind(template(VIEW_CLOCK), view_clock())
}

/// A host for the chapter alone: its names and `ikigai-fn`'s, in a kernel that reads `clock`.
pub fn kernel_with_clock(clock: Arc<dyn Clock>) -> Kernel {
    let space = Fallback::new(vec![
        Arc::new(space()) as Arc<dyn Space>,
        Arc::new(ikigai_fn::space()),
    ]);
    Kernel::with_meta_renderer(Arc::new(space), Arc::new(TurtleRenderer)).with_clock(clock)
}
// ANCHOR_END: space

// ANCHOR: manual_clock
/// A clock that stands still until it is told to move: what a test injects, so a minute
/// boundary happens exactly when the test says and never while it is not looking.
#[derive(Debug, Default)]
pub struct ManualClock(AtomicU64);

impl ManualClock {
    /// A clock stopped at `millis` since 1970-01-01T00:00Z.
    pub fn at(millis: u64) -> Self {
        ManualClock(AtomicU64::new(millis))
    }

    /// Stop the clock at `millis`.
    pub fn set(&self, millis: u64) {
        self.0.store(millis, Ordering::SeqCst);
    }

    /// Move the clock on by `millis`.
    pub fn advance(&self, millis: u64) {
        self.0.fetch_add(millis, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Time {
        Time::from_millis(self.0.load(Ordering::SeqCst))
    }
}
// ANCHOR_END: manual_clock

/// Milliseconds since 1970-01-01T00:00Z of an instant spelled `YYYY-MM-DDTHH:MMZ`, plus
/// `seconds` — how a test names the moment it stops a [`ManualClock`] at.
///
/// ```
/// use time_resource::millis_at;
/// assert_eq!(millis_at("1970-01-02T00:01Z", 30), 86_400_000 + 60_000 + 30_000);
/// ```
pub fn millis_at(instant: &str, seconds: u64) -> u64 {
    let minute = parse_instant(instant).expect("an instant spelled YYYY-MM-DDTHH:MMZ");
    u64::try_from(minute).expect("an instant after 1970") * MINUTE_MS + seconds * 1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_calendar_round_trips_every_day_of_four_centuries() {
        // 1600-03-01 .. 2400-03-01 covers two leap-century rules (1600, 2000, 2400 are
        // leap; 1700, 1800, 1900, 2100 are not) in both directions.
        let (first, last) = (days_from_civil(1600, 3, 1), days_from_civil(2400, 3, 1));
        for day in first..last {
            let (y, m, d) = civil_from_days(day);
            assert_eq!(days_from_civil(y, m, d), day, "{y}-{m}-{d}");
            assert_eq!(parse_date(&date_text(day)), Some(day));
        }
        assert_eq!(date_text(0), "1970-01-01");
        assert_eq!(date_text(days_from_civil(2000, 2, 29)), "2000-02-29");
    }

    #[test]
    fn an_instant_has_one_spelling() {
        assert_eq!(parse_instant("1970-01-01T00:00Z"), Some(0));
        assert_eq!(instant_text(0), "1970-01-01T00:00Z");
        for refused in [
            "2026-09-29T16:00",
            "2026-09-29 16:00Z",
            "2026-9-29T16:00Z",
            "2026-09-29T16:0Z",
            "2026-09-29T24:00Z",
            "2026-09-29T16:60Z",
            "2026-02-29T16:00Z",
            "2026-02-30T16:00Z",
            "2026-13-01T16:00Z",
            "2026-00-01T16:00Z",
            "2026-09-00T16:00Z",
            "+026-09-29T16:00Z",
            "2026-09-29T16:00:00Z",
            "",
        ] {
            assert_eq!(parse_instant(refused), None, "{refused}");
        }
        let at = parse_instant("2028-02-29T23:59Z").expect("a leap day");
        assert_eq!(instant_text(at), "2028-02-29T23:59Z");
        assert_eq!(instant_text(at + 1), "2028-03-01T00:00Z");
    }

    #[test]
    fn minutes_ahead_are_plain_whole_numbers() {
        assert_eq!(minutes_ahead("0").ok(), Some(0));
        assert_eq!(minutes_ahead("90").ok(), Some(90));
        for refused in ["", "090", "-5", "1.5", "525601", "ninety"] {
            assert!(minutes_ahead(refused).is_err(), "{refused}");
        }
    }
}
