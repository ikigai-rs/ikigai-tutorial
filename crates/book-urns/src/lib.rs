//! Find every resource name the book prints, and say which authority can confirm it.
//!
//! `mdbook test` compiles the book's Rust blocks, which is why the code examples can be
//! trusted. It is worth being precise about how little that covers: of the resource names
//! in this book, one sits in a block `mdbook test` compiles. The rest are prose, `ignore`
//! blocks, program output, and — first command a new reader types — a **shell** line in a
//! bash fence, which nothing on earth runs.
//!
//! So the book's correctness gate was thinnest exactly where a beginner's experience is
//! most fragile. This crate closes that: [`scan_tree`] pulls every `urn:` literal out of
//! the markdown and classifies it, and `tests/book_urns.rs` checks each one against the
//! authority that can actually answer for it.
//!
//! ## The two authorities, and why one name cannot be checked like the other
//!
//! * A name in a **Rust example** resolves in a host this workspace builds
//!   (`hello_camel::space()`, `loadable_module::host_space()`). That is checkable by
//!   construction: build the space and ask it.
//! * A name in an **`ikigai …` shell example** resolves in the CLI, a different crate on
//!   a different release schedule. Nothing here can build it, and CI has no trustworthy
//!   binary. Those check against `books/ikigai/cli-vocabulary.txt`, the book's written-
//!   down belief about somebody else's host, which a separate `#[ignore]`d test probes
//!   against a real binary when a human has one.
//!
//! Getting that distinction wrong in either direction would be worse than no gate: it
//! would go red on a line that is correct, and the fix a reader would reach for is to
//! edit the correct line.
//!
//! ## Escapes
//!
//! Some names in a teaching text are *supposed* not to resolve. They are declared in the
//! prose, next to the claim they qualify, as an HTML comment:
//!
//! ```text
//! <!-- urn-gate: unbound urn:fn:toCamel — the name this endpoint used to be bound at. -->
//! <!-- urn-gate: illustration urn:something:else — a stand-in for "any other resource". -->
//! ```
//!
//! `unbound` has teeth: the test asserts the name really does not resolve, so a paragraph
//! saying "this used to be called X" stops being true out loud if somebody binds X again.
//! `illustration` has none — it means "not a name, a placeholder" — so it costs a
//! sentence of reason, in the file, where the next editor will read it.

use std::fs;
use std::path::Path;

/// Where a mention sits, which decides who can confirm it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// Inside a shell fence, on a line that runs the `ikigai` binary. The CLI answers.
    Cli,
    /// Inside a Rust fence. A host this workspace builds answers.
    Rust,
    /// Prose, program output, a config fence — anything else. Either may answer.
    Prose,
}

/// The two ways a name is excused from resolving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exemption {
    /// Asserted **not** to resolve — a retired name the prose names on purpose.
    Unbound,
    /// A placeholder standing in for "some resource"; not checked at all.
    Illustration,
}

/// One `urn:` literal, where it was found and how it should be judged.
#[derive(Debug, Clone)]
pub struct Mention {
    /// Path relative to the scanned root.
    pub file: String,
    /// 1-based line number.
    pub line: usize,
    /// The literal, with any `?query` or `#fragment` tail removed.
    pub urn: String,
    /// Who can confirm it.
    pub context: Context,
    /// The token ended in `:`, so the book is naming a **family**, not a resource.
    pub is_prefix: bool,
}

impl Mention {
    /// `file:line` — how a failure should name itself.
    pub fn site(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}

/// A `<!-- urn-gate: … -->` escape declared in the prose.
#[derive(Debug, Clone)]
pub struct Directive {
    /// Path relative to the scanned root.
    pub file: String,
    /// 1-based line number of the `urn-gate:` line.
    pub line: usize,
    /// The name being excused.
    pub urn: String,
    /// Which escape.
    pub kind: Exemption,
    /// Why — required, because an unexplained exemption is how a gate rots.
    pub reason: String,
}

/// Everything one pass over the book found.
#[derive(Debug, Default)]
pub struct Scan {
    /// Every `urn:` literal, in file then line order.
    pub mentions: Vec<Mention>,
    /// Every escape declared in the prose.
    pub directives: Vec<Directive>,
}

impl Scan {
    /// The escape covering `urn`, if one was declared.
    pub fn exemption(&self, urn: &str) -> Option<&Directive> {
        self.directives.iter().find(|d| d.urn == urn)
    }
}

/// True for the characters a URN literal may contain in this book.
///
/// `*` is excluded on purpose: markdown bold (`**`) butts straight up against a name
/// often enough that swallowing an asterisk would corrupt tokens more often than it would
/// capture a real one. `urn:cap:net:*` therefore arrives as `urn:cap:net:`, which the test
/// skips anyway as a capability scope.
fn is_urn_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, ':' | '.' | '-' | '_' | '~' | '/' | '{' | '}' | '%')
}

/// Pull the `urn:` literals out of one line.
fn urns_in_line(line: &str) -> Vec<String> {
    let bytes: Vec<char> = line.chars().collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(&['u', 'r', 'n', ':']) {
            // Not a URN if it is the tail of a longer word (`return:` would not match, but
            // be explicit rather than lucky).
            let preceded_by_word = i > 0 && (bytes[i - 1].is_ascii_alphanumeric());
            let mut j = i;
            while j < bytes.len() && is_urn_char(bytes[j]) {
                j += 1;
            }
            if !preceded_by_word {
                let token: String = bytes[i..j].iter().collect();
                let token = token.trim_end_matches(['.', ',', ';']).to_string();
                if token.len() > "urn:".len() {
                    found.push(token);
                }
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    found
}

/// Blank out `<!-- … -->` regions, keeping line structure, and return the raw comment
/// text with the line each one started on.
fn split_comments(text: &str) -> (String, Vec<(usize, String)>) {
    let mut visible = String::with_capacity(text.len());
    let mut comments = Vec::new();
    let mut rest = text;
    let mut line = 1usize;
    while let Some(start) = rest.find("<!--") {
        let (before, from_marker) = rest.split_at(start);
        visible.push_str(before);
        line += before.matches('\n').count();
        let comment_line = line;
        let end = from_marker
            .find("-->")
            .map(|e| e + 3)
            .unwrap_or(from_marker.len());
        let (comment, after) = from_marker.split_at(end);
        comments.push((comment_line, comment.to_string()));
        // Preserve line numbering for everything that follows.
        for _ in 0..comment.matches('\n').count() {
            visible.push('\n');
        }
        line += comment.matches('\n').count();
        rest = after;
    }
    visible.push_str(rest);
    (visible, comments)
}

fn parse_directives(file: &str, comments: &[(usize, String)]) -> Result<Vec<Directive>, String> {
    let mut out = Vec::new();
    for (start_line, comment) in comments {
        for (offset, raw) in comment.lines().enumerate() {
            // The comment delimiters are punctuation, not part of anything being declared.
            let raw = raw.replace("<!--", " ").replace("-->", " ");
            let Some(rest) = raw.split_once("urn-gate:").map(|(_, r)| r.trim()) else {
                continue;
            };
            let line = start_line + offset;
            let mut words = rest.split_whitespace();
            let kind = match words.next() {
                Some("unbound") => Exemption::Unbound,
                Some("illustration") => Exemption::Illustration,
                other => {
                    return Err(format!(
                        "{file}:{line}: urn-gate directive needs `unbound` or `illustration`, \
                         got {other:?}"
                    ))
                }
            };
            let urn = match words.next() {
                Some(u) if u.starts_with("urn:") => u.to_string(),
                other => {
                    return Err(format!(
                        "{file}:{line}: urn-gate directive needs a `urn:…` name, got {other:?}"
                    ))
                }
            };
            let reason = words.collect::<Vec<_>>().join(" ");
            let reason = reason
                .trim_start_matches(['—', '-', ':'])
                .trim()
                .to_string();
            if reason.is_empty() {
                return Err(format!(
                    "{file}:{line}: urn-gate directive for {urn} states no reason. An \
                     exemption nobody explained is how a gate rots."
                ));
            }
            out.push(Directive {
                file: file.to_string(),
                line,
                urn,
                kind,
                reason,
            });
        }
    }
    Ok(out)
}

/// Scan one markdown document.
///
/// `file` is the label used in failure messages; it is not opened.
pub fn scan_markdown(file: &str, text: &str) -> Result<Scan, String> {
    let (visible, comments) = split_comments(text);
    let directives = parse_directives(file, &comments)?;

    let mut mentions = Vec::new();
    let mut fence: Option<String> = None;
    for (index, line) in visible.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(info) = trimmed.strip_prefix("```") {
            fence = match fence {
                Some(_) => None,
                None => Some(info.trim().to_ascii_lowercase()),
            };
            continue;
        }

        let context = match fence.as_deref() {
            Some(info) if info.starts_with("rust") => Context::Rust,
            Some(info)
                if info.starts_with("bash")
                    || info.starts_with("sh")
                    || info.starts_with("shell")
                    || info.starts_with("console") =>
            {
                // A shell fence can hold `cargo run` as easily as `ikigai`; only a line
                // that actually drives the CLI answers to the CLI.
                if line.contains("ikigai ") {
                    Context::Cli
                } else {
                    Context::Prose
                }
            }
            _ => Context::Prose,
        };

        for token in urns_in_line(line) {
            // `urn:iki:fn:toUpper?in=x` names `urn:iki:fn:toUpper`; the tail is arguments.
            let head = token.split(['?', '#']).next().unwrap_or(&token).to_string();
            mentions.push(Mention {
                file: file.to_string(),
                line: index + 1,
                is_prefix: head.ends_with(':'),
                urn: head,
                context,
            });
        }
    }

    Ok(Scan {
        mentions,
        directives,
    })
}

/// Scan every `.md` file under `root`, recursively, in a stable order.
pub fn scan_tree(root: &Path) -> Result<Scan, String> {
    let mut files = Vec::new();
    collect_markdown(root, root, &mut files)?;
    files.sort();

    let mut scan = Scan::default();
    for relative in files {
        let text = fs::read_to_string(root.join(&relative))
            .map_err(|e| format!("reading {relative}: {e}"))?;
        let one = scan_markdown(&relative, &text)?;
        scan.mentions.extend(one.mentions);
        scan.directives.extend(one.directives);
    }
    Ok(scan)
}

fn collect_markdown(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("reading {}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(root, &path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            out.push(relative.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

/// The book's written-down belief about what the `ikigai` CLI resolves.
///
/// This is not evidence — it is a claim, in one place, that one command can check. See
/// the header of `books/ikigai/cli-vocabulary.txt`.
#[derive(Debug, Default)]
pub struct Vocabulary {
    exact: Vec<String>,
    prefixes: Vec<String>,
}

impl Vocabulary {
    /// Parse the manifest format: one entry per line, `#` comments, trailing `*` for a
    /// prefix entry.
    pub fn parse(text: &str) -> Self {
        let mut vocabulary = Vocabulary::default();
        for line in text.lines() {
            let entry = line.split('#').next().unwrap_or("").trim();
            if entry.is_empty() {
                continue;
            }
            match entry.strip_suffix('*') {
                Some(prefix) => vocabulary.prefixes.push(prefix.to_string()),
                None => vocabulary.exact.push(entry.to_string()),
            }
        }
        vocabulary
    }

    /// Does the manifest claim the CLI resolves this name?
    pub fn covers(&self, urn: &str) -> bool {
        self.exact.iter().any(|e| e == urn) || self.prefixes.iter().any(|p| urn.starts_with(p))
    }

    /// Does the manifest claim anything under this family?
    pub fn covers_family(&self, prefix: &str) -> bool {
        self.prefixes
            .iter()
            .any(|p| p == prefix || prefix.starts_with(p))
            || self.exact.iter().any(|e| e.starts_with(prefix))
    }

    /// The exact entries, in file order — what the `#[ignore]`d probe test walks.
    pub fn exact_entries(&self) -> &[String] {
        &self.exact
    }

    /// The prefix entries, in file order.
    pub fn prefix_entries(&self) -> &[String] {
        &self.prefixes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shell_line_that_runs_the_cli_answers_to_the_cli() {
        let text = "```bash\nikigai --plain -c 'source urn:iki:fn:toUpper in=\"hi\"'\n```\n";
        let scan = scan_markdown("x.md", text).expect("scans");
        assert_eq!(scan.mentions.len(), 1);
        assert_eq!(scan.mentions[0].context, Context::Cli);
        assert_eq!(scan.mentions[0].urn, "urn:iki:fn:toUpper");
    }

    #[test]
    fn a_shell_line_that_does_not_run_the_cli_is_not_the_clis_problem() {
        let text = "```bash\ncargo run -p hello-camel -- urn:iki:tutorial:camel-case\n```\n";
        let scan = scan_markdown("x.md", text).expect("scans");
        assert_eq!(scan.mentions[0].context, Context::Prose);
    }

    #[test]
    fn a_rust_fence_answers_to_this_workspace() {
        let text = "```rust,no_run\nIri::parse(\"urn:iki:tutorial:camel-case\")\n```\n";
        let scan = scan_markdown("x.md", text).expect("scans");
        assert_eq!(scan.mentions[0].context, Context::Rust);
    }

    #[test]
    fn a_query_tail_is_not_part_of_the_name() {
        let scan = scan_markdown("x.md", "call it with `in=urn:iki:fn:toUpper?in=x` and\n")
            .expect("scans");
        assert_eq!(scan.mentions[0].urn, "urn:iki:fn:toUpper");
    }

    #[test]
    fn sentence_punctuation_is_not_part_of_the_name() {
        let scan = scan_markdown("x.md", "resolves urn:host:name.\n").expect("scans");
        assert_eq!(scan.mentions[0].urn, "urn:host:name");
        assert!(!scan.mentions[0].is_prefix);
    }

    #[test]
    fn a_trailing_colon_names_a_family_not_a_resource() {
        let scan =
            scan_markdown("x.md", "everything under `urn:greet:` is routed\n").expect("scans");
        assert!(scan.mentions[0].is_prefix);
        assert_eq!(scan.mentions[0].urn, "urn:greet:");
    }

    #[test]
    fn a_directive_is_not_itself_a_mention() {
        let text = "<!-- urn-gate: unbound urn:fn:toCamel — the old name -->\nprose\n";
        let scan = scan_markdown("x.md", text).expect("scans");
        assert!(scan.mentions.is_empty());
        assert_eq!(scan.directives.len(), 1);
        assert_eq!(scan.directives[0].kind, Exemption::Unbound);
        assert_eq!(scan.directives[0].urn, "urn:fn:toCamel");
        assert_eq!(scan.directives[0].reason, "the old name");
    }

    #[test]
    fn a_comment_does_not_shift_the_line_numbers_after_it() {
        let text = "one\n<!-- urn-gate: illustration urn:x:y — a stand-in\n     over two lines -->\nurn:host:name\n";
        let scan = scan_markdown("x.md", text).expect("scans");
        assert_eq!(scan.mentions.len(), 1);
        assert_eq!(scan.mentions[0].line, 4);
    }

    #[test]
    fn an_exemption_without_a_reason_is_refused() {
        let text = "<!-- urn-gate: unbound urn:fn:toCamel -->\n";
        assert!(scan_markdown("x.md", text).is_err());
    }

    #[test]
    fn the_manifest_understands_exact_and_prefix_entries() {
        let vocabulary = Vocabulary::parse("# a comment\nurn:kernel:catalog\nurn:file:*\n");
        assert!(vocabulary.covers("urn:kernel:catalog"));
        assert!(vocabulary.covers("urn:file:notes.txt"));
        assert!(!vocabulary.covers("urn:kernel:cache"));
        assert!(vocabulary.covers_family("urn:file:"));
        assert!(vocabulary.covers_family("urn:kernel:"));
        assert!(!vocabulary.covers_family("urn:greet:"));
    }
}
