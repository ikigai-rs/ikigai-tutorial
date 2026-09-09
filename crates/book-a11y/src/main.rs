//! Measure the built book against the WCAG 2.2 AA floor — colour and structure.
//!
//!     cargo run -p book-a11y -- books/ikigai            # gate: exits non-zero on a fault
//!     cargo run -p book-a11y -- books/ikigai --survey   # every pair and every page
//!
//! Run after `mdbook build`, because what it reads is what mdbook wrote: the palette
//! (`<book>/book/css/variables-*.css`, then the `additional-css` files `book.toml` names,
//! in that order) and every built page under `<book>/book/`. `scripts/test-books.sh` does
//! both, so CI does too. The structural half is `book_a11y::structure`; what it cannot
//! see — anything JavaScript adds after load — is the second instrument's job,
//! `a11y/axe.mjs`.

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

    // Structure: every built page, against the checks a parser can decide.
    let structural = match check_structure(&book, survey) {
        Ok(faults) => faults,
        Err(e) => {
            eprintln!("book-a11y: {e}");
            return ExitCode::from(2);
        }
    };
    if !structural.is_empty() {
        eprintln!(
            "book-a11y: {} structural fault{} (WCAG 2.2):",
            structural.len(),
            if structural.len() == 1 { "" } else { "s" }
        );
        for fault in &structural {
            eprintln!("  {}", fault.report());
        }
        eprintln!(
            "\nA fault in mdbook's own markup is repaired at load by js/a11y.js (see \
             book_a11y::structure::A11Y_JS_REPAIRS); one in a chapter is fixed in the \
             chapter. Neither is fixed by forking the theme."
        );
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

/// Every built page, checked; the faults `js/a11y.js` repairs at load are credited only
/// while the script still contains the repair.
///
/// Two files are skipped. `print.html` is every chapter concatenated, so it carries one
/// `<h1>` per chapter by design; `toc.html` is the sidebar as a fragment mdbook loads
/// into the page, not a document — no `<html>`, no `<main>`, no title. `404.html` is
/// checked like any page: a reader lands on it.
fn check_structure(book: &Path, survey: bool) -> Result<Vec<book_a11y::structure::Fault>, String> {
    let root = book.join("book");
    let mut pages = Vec::new();
    collect_html(&root, &root, &mut pages)?;
    pages.sort();
    if pages.len() < 5 {
        return Err(format!(
            "only {} pages under {} — run `mdbook build` first",
            pages.len(),
            root.display()
        ));
    }
    let a11y_js = read(&book.join("js/a11y.js"))?;

    let mut faults = Vec::new();
    let mut titles = Vec::new();
    let mut checked = 0usize;
    for relative in &pages {
        if relative == "print.html" || relative == "toc.html" {
            continue;
        }
        let html = read(&root.join(relative))?;
        let page_faults = book_a11y::structure::check_page(relative, &html);
        let page_faults = book_a11y::structure::without_scripted_repairs(page_faults, &a11y_js);
        if survey {
            println!(
                "{:<48} {}",
                relative,
                if page_faults.is_empty() {
                    "ok"
                } else {
                    "FAULT"
                }
            );
        }
        // `index.html` is the first chapter served again at the book's root — the same
        // document at two URLs, so the same title is right, not a collision.
        if relative != "index.html" {
            titles.push((relative.clone(), book_a11y::structure::page_title(&html)));
        }
        faults.extend(page_faults);
        checked += 1;
    }
    faults.extend(book_a11y::structure::check_titles(&titles));
    if faults.is_empty() {
        println!("book-a11y: {checked} pages pass the structural checks");
    }
    Ok(faults)
}

fn collect_html(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))? {
        let path = entry
            .map_err(|e| format!("reading {}: {e}", dir.display()))?
            .path();
        if path.is_dir() {
            collect_html(root, &path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("html") {
            out.push(
                path.strip_prefix(root)
                    .map_err(|e| format!("{}: {e}", path.display()))?
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    Ok(())
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
