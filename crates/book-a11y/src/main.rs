//! Measure the built book's palette against the WCAG contrast floor.
//!
//!     cargo run -p book-a11y -- books/ikigai            # gate: exits non-zero on a fault
//!     cargo run -p book-a11y -- books/ikigai --survey   # every pair, pass or fail
//!
//! Run after `mdbook build`, because what it reads is what mdbook wrote:
//! `<book>/book/css/variables-*.css`, then the `additional-css` files `book.toml` names,
//! in that order. `scripts/test-books.sh` does both, so CI does too.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let survey = args.iter().any(|a| a == "--survey");
    let Some(book) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: book-a11y <book directory> [--survey]");
        return ExitCode::from(2);
    };
    let book = PathBuf::from(book);

    let sheets = match load_sheets(&book) {
        Ok(sheets) => sheets,
        Err(e) => {
            eprintln!("book-a11y: {e}");
            return ExitCode::from(2);
        }
    };
    let palette = book_a11y::merge(&sheets);

    if survey {
        println!(
            "{:<8} {:<26} {:<24} {:>7}",
            "theme", "foreground", "background", "ratio"
        );
        for (theme, pair, measured) in book_a11y::measure_all(&palette) {
            let ratio = match measured {
                Some(r) => format!("{r:.2}"),
                None => "—".to_string(),
            };
            let verdict = match measured {
                Some(r) if r >= pair.floor => "ok",
                Some(_) => "FAIL",
                None => "not declared",
            };
            println!(
                "{theme:<8} {:<26} {:<24} {ratio:>7}  {verdict:<4}  needs {:.1}",
                pair.foreground, pair.background, pair.floor
            );
        }
        println!();
    }

    // The skip link is JavaScript reaching into markup this repository does not own, so
    // the one thing that can rot silently is the id it aims at. mdbook 0.5 renamed that
    // element (`#content` became `#mdbook-content`), and a skip link pointing at nothing
    // still looks exactly like a working one.
    if let Err(e) = check_skip_link(&book) {
        eprintln!("book-a11y: {e}");
        return ExitCode::FAILURE;
    }

    let findings = book_a11y::check(&palette);
    if findings.is_empty() {
        println!(
            "book-a11y: {} pairs across {} themes clear the floor",
            book_a11y::PAIRS.len(),
            book_a11y::THEMES.len()
        );
        return ExitCode::SUCCESS;
    }

    eprintln!(
        "book-a11y: {} colour pair{} below the WCAG floor:",
        findings.len(),
        if findings.len() == 1 { "" } else { "s" }
    );
    for finding in &findings {
        eprintln!("  {}", finding.report());
        // Either end can be the right one to move, and sometimes only one end can move
        // at all — a ground already at white cannot get further from grey.
        let (foreground, background) = book_a11y::suggest(finding);
        let mut options = Vec::new();
        if let Some(fg) = foreground {
            options.push(format!("{}: {fg};", finding.foreground));
        }
        if let Some(bg) = background {
            options.push(format!("{}: {bg};", finding.background));
        }
        if !options.is_empty() {
            eprintln!(
                "      would clear at {} {{ {} }}",
                finding.theme,
                options.join("  or  ")
            );
        }
    }
    eprintln!(
        "\nThe repair goes in the book's own `additional-css`, which is applied after \
         mdbook's palette — never by editing the theme, which is not ours."
    );
    ExitCode::FAILURE
}

/// Confirm the built pages still carry the skip link and the element it targets.
fn check_skip_link(book: &Path) -> Result<(), String> {
    let page = book.join("book/index.html");
    let html = read(&page)?;
    book_a11y::skip_link_faults(&html)
        .map_err(|faults| format!("{} is missing:\n  {}", page.display(), faults.join("\n  ")))
}

/// mdbook's generated palette, then the book's overrides, in cascade order.
fn load_sheets(book: &Path) -> Result<Vec<String>, String> {
    let generated = book.join("book/css");
    let mut variables: Vec<PathBuf> = std::fs::read_dir(&generated)
        .map_err(|e| {
            format!(
                "{}: {e} — run `mdbook build` first; this reads the palette mdbook \
                 generated, not a copy of it",
                generated.display()
            )
        })?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("variables") && n.ends_with(".css"))
        })
        .collect();
    variables.sort();
    if variables.is_empty() {
        return Err(format!(
            "no variables*.css in {} — mdbook's palette is what this measures, and it is \
             not there",
            generated.display()
        ));
    }

    let mut sheets = Vec::new();
    for path in &variables {
        sheets.push(read(path)?);
    }

    let toml = read(&book.join("book.toml"))?;
    for entry in book_a11y::additional_css(&toml) {
        sheets.push(read(&book.join(&entry))?);
    }
    Ok(sheets)
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))
}
