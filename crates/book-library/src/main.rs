//! `book-library`: write the library page, or check an assembled site links every book.
//!
//! ```text
//! book-library generate [--check] [ROOT]     write books/library/src/index.md (or compare)
//! book-library check-site SITE [ROOT]        the assembled site's library page links every book
//! ```
//!
//! ROOT is the repository (default `.`). Every directory under `ROOT/books/` with a `book.toml`,
//! the library aside, is a book, and each needs an entry in `books/library/library.toml`.

use std::path::PathBuf;
use std::process::ExitCode;

use book_library::{check_site, expected_page, DATA_FILE, LIBRARY, PAGE};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("generate") => generate(&args[1..]),
        Some("check-site") => site(&args[1..]),
        _ => Err("usage: book-library generate [--check] [ROOT]\n       \
                  book-library check-site SITE [ROOT]"
            .to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn generate(args: &[String]) -> Result<(), String> {
    let check = args.iter().any(|a| a == "--check");
    let root = args
        .iter()
        .find(|a| *a != "--check")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let page = expected_page(&root)?;
    let path = root.join("books").join(LIBRARY).join("src").join(PAGE);
    if check {
        if std::fs::read_to_string(&path).ok().as_deref() != Some(page.as_str()) {
            return Err(format!(
                "books/{LIBRARY}/src/{PAGE} does not match books/{LIBRARY}/{DATA_FILE}: run \
                 `cargo run -p book-library -- generate` and commit the result"
            ));
        }
        return Ok(());
    }
    std::fs::write(&path, page).map_err(|e| format!("{}: {e}", path.display()))
}

fn site(args: &[String]) -> Result<(), String> {
    let site = args
        .first()
        .ok_or("check-site needs the assembled site's directory")?;
    let root = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let n = check_site(&root, &PathBuf::from(site))?;
    println!("the library links all {n} books");
    Ok(())
}
