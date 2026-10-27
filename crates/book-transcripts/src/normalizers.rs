//! What legitimately varies between the run a page recorded and the run the check makes, by name.
//!
//! A transcript compares what a command printed with what the page says, line by line, and `…`
//! already lets a page say "anything here". That is the widest tool there is: `…` matches a
//! timestamp, and it matches a stack trace too. A **normalizer** is the narrow one. It names ONE
//! shape (a timestamp, a duration, a port after a loopback host), and on both sides of the
//! comparison it rewrites only that shape to a placeholder, so everything around it is still
//! compared exactly. A normalizer that hid more than its shape would be a gate that silently
//! covers less than it looks like it does (field guide 9c), so each one here says what it may hide
//! and what it may not, and a test pins both halves.
//!
//! A block asks for normalizers in its declaration, and a page may ask once for all its blocks:
//!
//! ```text
//! <!-- transcript: run; normalize timestamp, duration -->
//! <!-- transcripts: normalize home, workdir -->
//! ```
//!
//! None is on unless asked for. A name the runner does not know fails the block.
//!
//! | name          | rewrites                                                         | to              |
//! |---------------|------------------------------------------------------------------|-----------------|
//! | `timestamp`   | an RFC 3339 date-time, `2026-10-27T18:04:05Z` (`T` or a space)    | `<timestamp>`   |
//! | `duration`    | `12ms`, `340µs`, `340us`, `9ns`, and seconds WITH a decimal point | `<duration>`    |
//! | `version`     | the `x.y.z` after a word that starts `ikigai` (`ikigai 0.1.69`)   | `<version>`     |
//! | `fingerprint` | a run of exactly 16 or 64 lowercase hex digits                    | `<fingerprint>` |
//! | `port`        | the port after `127.0.0.1:`, `localhost:`, `[::1]:` or `0.0.0.0:` | `<port>`        |
//! | `home`        | the `HOME` the runner gave the command                            | `~`             |
//! | `workdir`     | the page's working directory                                      | `$PWD`          |
//!
//! ## Bindings: a port the server chose, used by the next command
//!
//! `port` and `fingerprint` are **binding** normalizers. A server started on port 0 prints the
//! port it bound, and the page shows the one it bound when the page was written. When the page's
//! line and the run's line match, the recorded value is BOUND to the run's: from then on, every
//! command and every `file` block on the page has the recorded value replaced by the run's before
//! it runs, so `ikigai --connect quic://127.0.0.1:55682` reaches the server this run started. And
//! a binding is held to: the recorded value must stand for the same run value every time it
//! appears, and two recorded values may not stand for one run value. So a page cannot claim that
//! two servers share a port, or that one certificate has two fingerprints, and pass.

use std::fmt;

/// One named normalizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Normalizer {
    /// An RFC 3339 date-time.
    Timestamp,
    /// A duration in nanoseconds, microseconds or milliseconds, or seconds with a decimal point.
    Duration,
    /// The version after an `ikigai…` word.
    Version,
    /// Exactly 16 or 64 lowercase hex digits (binds).
    Fingerprint,
    /// The port after a loopback or wildcard host (binds).
    Port,
    /// The `HOME` the command ran with.
    Home,
    /// The page's working directory.
    Workdir,
}

impl Normalizer {
    /// Every normalizer, in the order they are applied: the paths first, longest-lived name
    /// first, then the shapes, so that a port inside a path is a port and a path is never cut in
    /// two by a shape inside it.
    pub const ALL: [Normalizer; 7] = [
        Normalizer::Workdir,
        Normalizer::Home,
        Normalizer::Timestamp,
        Normalizer::Duration,
        Normalizer::Version,
        Normalizer::Fingerprint,
        Normalizer::Port,
    ];

    /// The name a declaration uses.
    pub fn name(self) -> &'static str {
        match self {
            Normalizer::Timestamp => "timestamp",
            Normalizer::Duration => "duration",
            Normalizer::Version => "version",
            Normalizer::Fingerprint => "fingerprint",
            Normalizer::Port => "port",
            Normalizer::Home => "home",
            Normalizer::Workdir => "workdir",
        }
    }

    /// The normalizer a declaration names, if there is one by that name.
    pub fn named(name: &str) -> Option<Normalizer> {
        Normalizer::ALL.into_iter().find(|n| n.name() == name)
    }

    /// What a matched shape becomes, on both sides.
    pub fn placeholder(self) -> &'static str {
        match self {
            Normalizer::Timestamp => "<timestamp>",
            Normalizer::Duration => "<duration>",
            Normalizer::Version => "<version>",
            Normalizer::Fingerprint => "<fingerprint>",
            Normalizer::Port => "<port>",
            Normalizer::Home => "~",
            Normalizer::Workdir => "$PWD",
        }
    }

    /// Does a value this normalizer finds stand for a run's value, to be substituted into the
    /// page's later commands?
    pub fn binds(self) -> bool {
        matches!(self, Normalizer::Fingerprint | Normalizer::Port)
    }
}

impl fmt::Display for Normalizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Resolve a declaration's names, in [`Normalizer::ALL`] order, or say which name is unknown.
///
/// ```
/// use book_transcripts::normalizers::{resolve, Normalizer};
///
/// assert_eq!(
///     resolve(&["port".into(), "timestamp".into()]).unwrap(),
///     vec![Normalizer::Timestamp, Normalizer::Port]
/// );
/// assert!(resolve(&["ports".into()]).unwrap_err().contains("ports"));
/// ```
pub fn resolve(names: &[String]) -> Result<Vec<Normalizer>, String> {
    let mut out = Vec::new();
    for name in names {
        match Normalizer::named(name) {
            Some(n) => out.push(n),
            None => {
                return Err(format!(
                    "no normalizer is called `{name}`; there are: {}",
                    Normalizer::ALL
                        .iter()
                        .map(|n| n.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// The paths `home` and `workdir` stand for: the exact directories the runner chose, each in
/// every spelling a command may print (as created, and canonical: on macOS the temporary
/// directory is a symlink, `/var` to `/private/var`, and a program prints whichever it was
/// handed).
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Spellings of the command's `HOME`.
    pub home: Vec<String>,
    /// Spellings of the page's working directory.
    pub workdir: Vec<String>,
}

/// `line` with every shape `normalizers` names replaced by its placeholder.
///
/// ```
/// use book_transcripts::normalizers::{apply, Context, Normalizer};
///
/// let line = "ikigai: serving on quic://127.0.0.1:55682  (connections re-minted every 600s)";
/// assert_eq!(
///     apply(line, &[Normalizer::Port], &Context::default()),
///     "ikigai: serving on quic://127.0.0.1:<port>  (connections re-minted every 600s)"
/// );
/// ```
pub fn apply(line: &str, normalizers: &[Normalizer], context: &Context) -> String {
    let mut out = line.to_string();
    for n in Normalizer::ALL {
        if !normalizers.contains(&n) {
            continue;
        }
        out = match n {
            Normalizer::Home => replace_paths(&out, &context.home, n.placeholder()),
            Normalizer::Workdir => replace_paths(&out, &context.workdir, n.placeholder()),
            _ => {
                let spans = spans(&out, n);
                splice(&out, &spans, n.placeholder())
            }
        };
    }
    out
}

/// The values a shape normalizer finds in `line`, in order (empty for the path normalizers).
pub fn tokens(line: &str, n: Normalizer) -> Vec<String> {
    spans(line, n)
        .into_iter()
        .map(|(a, b)| line[a..b].to_string())
        .collect()
}

/// Replace each `(start, end)` byte span of `line` with `with`.
fn splice(line: &str, spans: &[(usize, usize)], with: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut at = 0;
    for &(a, b) in spans {
        out.push_str(&line[at..a]);
        out.push_str(with);
        at = b;
    }
    out.push_str(&line[at..]);
    out
}

/// Replace every occurrence of any of `paths` (longest first) that ends where a path ends:
/// before a `/`, the end of the line, or a character that cannot continue a file name the
/// runner chose. `…/bt42-7` is not replaced inside `…/bt42-7-home`, which is the trap beside it.
fn replace_paths(line: &str, paths: &[String], with: &str) -> String {
    let mut paths: Vec<&String> = paths.iter().filter(|p| !p.is_empty()).collect();
    paths.sort_by_key(|p| std::cmp::Reverse(p.len()));
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    'scan: while !rest.is_empty() {
        for p in &paths {
            if let Some(after) = rest.strip_prefix(p.as_str()) {
                let continues = after
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'));
                if !continues {
                    out.push_str(with);
                    rest = after;
                    continue 'scan;
                }
            }
        }
        let c = rest.chars().next().expect("not empty");
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

/// The byte spans of `line` that the shape normalizer `n` matches, in order, never overlapping.
fn spans(line: &str, n: Normalizer) -> Vec<(usize, usize)> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        // Every shape starts with an ASCII character, never inside a multi-byte one (`…`, `µ`).
        if !line.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let found = match n {
            Normalizer::Timestamp => timestamp_at(b, i),
            Normalizer::Duration => duration_at(line, i),
            Normalizer::Version => version_at(line, i),
            Normalizer::Fingerprint => fingerprint_at(b, i),
            Normalizer::Port => port_at(line, i),
            Normalizer::Home | Normalizer::Workdir => None,
        };
        match found {
            Some((start, end)) => {
                out.push((start, end));
                i = end.max(i + 1);
            }
            None => i += 1,
        }
    }
    out
}

fn digits(b: &[u8], i: usize, n: usize) -> bool {
    b.len() >= i + n && b[i..i + n].iter().all(u8::is_ascii_digit)
}

/// `YYYY-MM-DD[T ]HH:MM:SS[.frac][Z|±HH:MM]`, starting at `i` and not inside a longer number.
fn timestamp_at(b: &[u8], i: usize) -> Option<(usize, usize)> {
    if i > 0 && b[i - 1].is_ascii_alphanumeric() {
        return None;
    }
    let shape = digits(b, i, 4)
        && b.get(i + 4) == Some(&b'-')
        && digits(b, i + 5, 2)
        && b.get(i + 7) == Some(&b'-')
        && digits(b, i + 8, 2)
        && matches!(b.get(i + 10), Some(b'T') | Some(b' '))
        && digits(b, i + 11, 2)
        && b.get(i + 13) == Some(&b':')
        && digits(b, i + 14, 2)
        && b.get(i + 16) == Some(&b':')
        && digits(b, i + 17, 2);
    if !shape {
        return None;
    }
    let mut end = i + 19;
    if b.get(end) == Some(&b'.') && digits(b, end + 1, 1) {
        end += 1;
        while end < b.len() && b[end].is_ascii_digit() {
            end += 1;
        }
    }
    match b.get(end) {
        Some(b'Z') => end += 1,
        Some(b'+') | Some(b'-')
            if digits(b, end + 1, 2) && b.get(end + 3) == Some(&b':') && digits(b, end + 4, 2) =>
        {
            end += 6
        }
        _ => {}
    }
    Some((i, end))
}

/// A number followed by `ns`, `µs`, `us` or `ms`, or a DECIMAL number followed by `s`: a
/// measured time. Whole seconds (`10s`) are left alone, because a whole number of seconds in
/// this book is a setting (a timeout, a connection age) and a setting is a claim.
fn duration_at(line: &str, i: usize) -> Option<(usize, usize)> {
    let b = line.as_bytes();
    if !b[i].is_ascii_digit() || (i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'.')) {
        return None;
    }
    let mut end = i;
    while end < b.len() && b[end].is_ascii_digit() {
        end += 1;
    }
    let mut decimal = false;
    if b.get(end) == Some(&b'.') && digits(b, end + 1, 1) {
        decimal = true;
        end += 1;
        while end < b.len() && b[end].is_ascii_digit() {
            end += 1;
        }
    }
    let rest = &line[end..];
    let unit = ["ns", "µs", "us", "ms"]
        .into_iter()
        .find(|u| rest.starts_with(u))
        .map(str::len)
        .or_else(|| (decimal && rest.starts_with('s')).then_some(1))?;
    let after = end + unit;
    let continues = line[after..]
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric());
    (!continues).then_some((i, after))
}

/// The `x.y.z[-pre]` after a word that starts `ikigai` and one space.
fn version_at(line: &str, i: usize) -> Option<(usize, usize)> {
    let b = line.as_bytes();
    if b[i] != b' ' {
        return None;
    }
    let word_start = line[..i]
        .rfind(|c: char| c.is_whitespace())
        .map_or(0, |w| w + 1);
    // `ikigai`, or `ikigai-` and a crate's name: never `ikigai:` (a log prefix) or a longer IRI.
    let word = &line[word_start..i];
    let crate_name = word == "ikigai"
        || word.strip_prefix("ikigai-").is_some_and(|rest| {
            !rest.is_empty()
                && rest
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        });
    if !crate_name {
        return None;
    }
    let start = i + 1;
    let mut end = start;
    for part in 0..3 {
        let from = end;
        while end < b.len() && b[end].is_ascii_digit() {
            end += 1;
        }
        if end == from {
            return None;
        }
        if part < 2 {
            if b.get(end) != Some(&b'.') {
                return None;
            }
            end += 1;
        }
    }
    if b.get(end) == Some(&b'-') {
        let from = end + 1;
        let mut e = from;
        while e < b.len() && (b[e].is_ascii_alphanumeric() || b[e] == b'.') {
            e += 1;
        }
        if e > from {
            end = e;
        }
    }
    if b.get(end).is_some_and(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some((start, end))
}

fn is_hex(c: u8) -> bool {
    c.is_ascii_digit() || (b'a'..=b'f').contains(&c)
}

/// Exactly 16 or 64 lowercase hex digits, not inside a longer word.
fn fingerprint_at(b: &[u8], i: usize) -> Option<(usize, usize)> {
    if i > 0 && b[i - 1].is_ascii_alphanumeric() {
        return None;
    }
    let mut end = i;
    while end < b.len() && is_hex(b[end]) {
        end += 1;
    }
    let len = end - i;
    if (len == 16 || len == 64) && !b.get(end).is_some_and(|c| c.is_ascii_alphanumeric()) {
        Some((i, end))
    } else {
        None
    }
}

/// The hosts a served port is printed after.
const PORT_HOSTS: [&str; 4] = ["127.0.0.1:", "localhost:", "[::1]:", "0.0.0.0:"];

/// The 1 to 5 digits after one of [`PORT_HOSTS`], not followed by another digit.
fn port_at(line: &str, i: usize) -> Option<(usize, usize)> {
    let host = PORT_HOSTS.iter().find(|h| line[i..].starts_with(*h))?;
    let b = line.as_bytes();
    let start = i + host.len();
    let mut end = start;
    while end < b.len() && b[end].is_ascii_digit() {
        end += 1;
    }
    (end > start && end - start <= 5).then_some((start, end))
}

/// What the recorded values a page shows stand for in this run: a port the page's server bound
/// when the page was written, and the one it bound now.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bindings {
    pairs: Vec<(Normalizer, String, String)>,
}

impl Bindings {
    /// Learn from one line the page shows (`expected`) and the line the run printed in its place
    /// (`actual`), which already match once normalized: the binding normalizers' values, paired
    /// in order. A value bound before must stand for the same run value again, and a run value
    /// may stand for one recorded value only.
    ///
    /// ```
    /// use book_transcripts::normalizers::{Bindings, Normalizer};
    ///
    /// let mut b = Bindings::default();
    /// b.learn(
    ///     "serving on quic://127.0.0.1:55682",
    ///     "serving on quic://127.0.0.1:61017",
    ///     &[Normalizer::Port],
    /// )
    /// .unwrap();
    /// assert_eq!(
    ///     b.rewrite("ikigai --connect quic://127.0.0.1:55682 -c 'source x'"),
    ///     "ikigai --connect quic://127.0.0.1:61017 -c 'source x'"
    /// );
    /// // The page cannot later say the same server is somewhere else.
    /// assert!(b
    ///     .learn("127.0.0.1:55682", "127.0.0.1:61999", &[Normalizer::Port])
    ///     .is_err());
    /// ```
    pub fn learn(
        &mut self,
        expected: &str,
        actual: &str,
        normalizers: &[Normalizer],
    ) -> Result<(), String> {
        for n in normalizers.iter().copied().filter(|n| n.binds()) {
            let recorded = tokens(expected, n);
            let run = tokens(actual, n);
            if recorded.len() != run.len() {
                continue;
            }
            for (r, a) in recorded.into_iter().zip(run) {
                self.bind(n, &r, &a)?;
                // A full fingerprint's first 16 digits are the short form a banner prints.
                if n == Normalizer::Fingerprint && r.len() == 64 {
                    self.bind(n, &r[..16], &a[..16])?;
                }
            }
        }
        Ok(())
    }

    fn bind(&mut self, n: Normalizer, recorded: &str, actual: &str) -> Result<(), String> {
        for (kind, r, a) in &self.pairs {
            if *kind != n {
                continue;
            }
            if r == recorded && a != actual {
                return Err(format!(
                    "the page's {n} {recorded} stood for {a} in this run, and here for {actual}"
                ));
            }
            if a == actual && r != recorded {
                return Err(format!(
                    "the page shows two {n}s, {r} and {recorded}, where this run had one, {actual}"
                ));
            }
            if r == recorded {
                return Ok(());
            }
        }
        self.pairs
            .push((n, recorded.to_string(), actual.to_string()));
        Ok(())
    }

    /// `text` (a command, or a line of a `file` block) with every bound recorded value replaced
    /// by the run's value, where it stands as that shape: a port after one of the hosts a port is
    /// read after, a fingerprint not inside a longer run of hex.
    pub fn rewrite(&self, text: &str) -> String {
        let mut out = text.to_string();
        for (n, recorded, actual) in &self.pairs {
            if recorded == actual {
                continue;
            }
            let spans: Vec<(usize, usize)> = spans(&out, *n)
                .into_iter()
                .filter(|&(a, b)| &out[a..b] == recorded)
                .collect();
            let mut next = String::with_capacity(out.len());
            let mut at = 0;
            for (a, b) in spans {
                next.push_str(&out[at..a]);
                next.push_str(actual);
                at = b;
            }
            next.push_str(&out[at..]);
            out = next;
        }
        out
    }

    /// Is anything bound?
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(n: Normalizer, line: &str) -> String {
        apply(line, &[n], &Context::default())
    }

    // Each normalizer has two tests: what it may hide, and what it may not.

    #[test]
    fn timestamp_hides_an_rfc3339_instant() {
        let n = Normalizer::Timestamp;
        assert_eq!(one(n, "at 2026-10-27T18:04:05Z ok"), "at <timestamp> ok");
        assert_eq!(one(n, "2026-10-27T18:04:05.123+02:00"), "<timestamp>");
        assert_eq!(one(n, "2026-10-27 18:04:05"), "<timestamp>");
    }

    #[test]
    fn timestamp_does_not_hide_a_date_alone_or_a_version() {
        let n = Normalizer::Timestamp;
        for line in [
            "released 2026-10-27",
            "ikigai 0.1.69",
            "18:04:05",
            "x2026-10-27T18:04:05Z",
        ] {
            assert_eq!(one(n, line), line);
        }
    }

    #[test]
    fn duration_hides_a_measured_time() {
        let n = Normalizer::Duration;
        assert_eq!(one(n, "toUpper · 12ms → 3b"), "toUpper · <duration> → 3b");
        assert_eq!(
            one(n, "took 340µs, then 9ns"),
            "took <duration>, then <duration>"
        );
        assert_eq!(one(n, "1.25s elapsed"), "<duration> elapsed");
        assert_eq!(one(n, "(7us)"), "(<duration>)");
    }

    #[test]
    fn duration_does_not_hide_a_setting_a_count_or_a_word() {
        let n = Normalizer::Duration;
        for line in [
            "connections re-minted every 600s",
            "default 10s",
            "3 items",
            "2ms2",
            "v1.2ms",
            "items",
            "a 1.5 second wait",
        ] {
            assert_eq!(one(n, line), line, "{line}");
        }
    }

    #[test]
    fn version_hides_the_version_of_an_ikigai_crate() {
        let n = Normalizer::Version;
        assert_eq!(
            one(n, "ikigai 0.1.69 — REPL. Type `help`"),
            "ikigai <version> — REPL. Type `help`"
        );
        assert_eq!(one(n, "ikigai-core 0.1.108"), "ikigai-core <version>");
        assert_eq!(one(n, "ikigai-gonk 0.2.0-rc.1"), "ikigai-gonk <version>");
    }

    #[test]
    fn version_does_not_hide_another_crates_version_or_any_other_number() {
        let n = Normalizer::Version;
        for line in [
            "steel-core 0.8.3",
            "version 0.1.69",
            "ikigai 0.1",
            "ikigai: 0.1.69",
            "urn:ikigai:endpoint:toUpper 1.2.3x",
        ] {
            assert_eq!(one(n, line), line, "{line}");
        }
    }

    #[test]
    fn fingerprint_hides_a_short_or_full_sha256_in_hex() {
        let n = Normalizer::Fingerprint;
        assert_eq!(
            one(n, "client  base  bc42e0a9dff48e21  /x/client.crt"),
            "client  base  <fingerprint>  /x/client.crt"
        );
        let full = "c1c8462d57dd0c178e8fec900d57b35992ba2cc625db4c6928a1a92163b4cc2b";
        assert_eq!(
            one(n, &format!("fingerprint: {full}")),
            "fingerprint: <fingerprint>"
        );
        // What it MAY hide, and why it is opt-in: a content digest has the same shape, so a block
        // whose digest is the claim must not declare it.
        assert_eq!(
            one(n, &format!("contract:toUpper:source:b3:{full}")),
            "contract:toUpper:source:b3:<fingerprint>"
        );
    }

    #[test]
    fn fingerprint_does_not_hide_other_lengths_or_capitals() {
        let n = Normalizer::Fingerprint;
        for line in [
            "bc42e0a9dff48e2",   // 15
            "bc42e0a9dff48e21a", // 17
            "BC42E0A9DFF48E21",  // capitals: not the form ikigai prints
            "item 01m495wy306p819ms8h5jyc",
            "xbc42e0a9dff48e21",
        ] {
            assert_eq!(one(n, line), line, "{line}");
        }
    }

    #[test]
    fn port_hides_the_port_after_a_loopback_or_wildcard_host() {
        let n = Normalizer::Port;
        assert_eq!(
            one(n, "serving on quic://127.0.0.1:55682  (Ctrl-C to stop)"),
            "serving on quic://127.0.0.1:<port>  (Ctrl-C to stop)"
        );
        assert_eq!(one(n, "http://localhost:8080/"), "http://localhost:<port>/");
        assert_eq!(
            one(n, "[::1]:9 and 0.0.0.0:443"),
            "[::1]:<port> and 0.0.0.0:<port>"
        );
    }

    #[test]
    fn port_does_not_hide_a_remote_host_a_bare_number_or_the_address() {
        let n = Normalizer::Port;
        for line in [
            "quic://server.local:4434",
            "port 55682",
            "127.0.0.1",
            "listening on 10.0.0.2:4434",
            "127.0.0.1:123456",
        ] {
            assert_eq!(one(n, line), line, "{line}");
        }
    }

    fn context() -> Context {
        Context {
            home: vec!["/tmp/bt42-7-home".into(), "/private/tmp/bt42-7-home".into()],
            workdir: vec!["/tmp/bt42-7".into(), "/private/tmp/bt42-7".into()],
        }
    }

    #[test]
    fn home_and_workdir_hide_exactly_the_directories_the_runner_chose() {
        let both = [Normalizer::Home, Normalizer::Workdir];
        assert_eq!(
            apply(
                "watching /private/tmp/bt42-7-home/.ikigai/workspace",
                &both,
                &context()
            ),
            "watching ~/.ikigai/workspace"
        );
        assert_eq!(
            apply("serving on /tmp/bt42-7/k.sock", &both, &context()),
            "serving on $PWD/k.sock"
        );
        assert_eq!(
            apply("at /private/tmp/bt42-7", &both, &context()),
            "at $PWD"
        );
    }

    #[test]
    fn home_and_workdir_do_not_hide_another_path_or_a_longer_name() {
        let both = [Normalizer::Home, Normalizer::Workdir];
        for line in [
            "/tmp/bt42-70/k.sock",
            "/tmp/bt42-7x",
            "/home/runner/.ikigai/workspace",
            "/tmp/other/k.sock",
        ] {
            assert_eq!(apply(line, &both, &context()), line, "{line}");
        }
        // Only the workdir asked for: the trap beside it is not mistaken for it.
        assert_eq!(
            apply("/tmp/bt42-7-home/x", &[Normalizer::Workdir], &context()),
            "/tmp/bt42-7-home/x"
        );
    }

    #[test]
    fn every_normalizer_reads_a_line_with_multibyte_characters() {
        // `…` is how a page says "anything here", and it is three bytes: no shape may start
        // inside it (a server's banner with a `…` in it once panicked the runner).
        let line = "on 127.0.0.1 port … (http://127.0.0.1:51234/) — 340µs · ikigai 0.1.69 …";
        for n in Normalizer::ALL {
            let _ = apply(line, &[n], &context());
        }
        assert_eq!(
            apply(line, &Normalizer::ALL, &context()),
            "on 127.0.0.1 port … (http://127.0.0.1:<port>/) — <duration> · ikigai <version> …"
        );
    }

    #[test]
    fn nothing_is_normalized_that_was_not_asked_for() {
        let line = "2026-10-27T18:04:05Z 12ms ikigai 0.1.69 127.0.0.1:5 bc42e0a9dff48e21";
        assert_eq!(apply(line, &[], &context()), line);
    }

    #[test]
    fn an_unknown_name_is_refused_with_the_known_ones() {
        let e = resolve(&["timestamps".into()]).unwrap_err();
        assert!(
            e.contains("`timestamps`") && e.contains("timestamp, duration"),
            "{e}"
        );
    }

    #[test]
    fn a_binding_is_one_to_one() {
        let p = [Normalizer::Port];
        let mut b = Bindings::default();
        b.learn("127.0.0.1:1111", "127.0.0.1:5000", &p).unwrap();
        b.learn("127.0.0.1:1111", "127.0.0.1:5000", &p).unwrap();
        assert!(b.learn("127.0.0.1:1111", "127.0.0.1:6000", &p).is_err());
        assert!(
            b.learn("127.0.0.1:2222", "127.0.0.1:5000", &p).is_err(),
            "two recorded ports for one bound port"
        );
        b.learn("127.0.0.1:2222", "127.0.0.1:6000", &p).unwrap();
        assert_eq!(
            b.rewrite("a 127.0.0.1:1111 b localhost:2222 c 1111 d 127.0.0.1:11112"),
            "a 127.0.0.1:5000 b localhost:6000 c 1111 d 127.0.0.1:11112",
            "only a port after a host is rewritten, and never inside a longer number"
        );
    }

    #[test]
    fn a_full_fingerprint_binds_its_short_form_too() {
        let f = [Normalizer::Fingerprint];
        let page = "c1c8462d57dd0c178e8fec900d57b35992ba2cc625db4c6928a1a92163b4cc2b";
        let run = "0000462d57dd0c178e8fec900d57b35992ba2cc625db4c6928a1a92163b40000";
        let mut b = Bindings::default();
        b.learn(
            &format!("fingerprint: {page}"),
            &format!("fingerprint: {run}"),
            &f,
        )
        .unwrap();
        assert_eq!(
            b.rewrite(&format!("{{\"{page}\": {{\"grant\": \"read\"}}}}")),
            format!("{{\"{run}\": {{\"grant\": \"read\"}}}}")
        );
        assert!(b
            .learn("client c1c8462d57dd0c17", "client 1111462d57dd0c17", &f)
            .is_err());
    }

    #[test]
    fn a_shape_normalizer_without_a_binding_rewrites_nothing() {
        let mut b = Bindings::default();
        b.learn(
            "at 2026-10-27T18:04:05Z",
            "at 2026-10-28T00:00:00Z",
            &[Normalizer::Timestamp],
        )
        .unwrap();
        assert!(b.is_empty());
    }
}
