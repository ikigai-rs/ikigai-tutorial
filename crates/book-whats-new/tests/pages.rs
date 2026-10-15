//! Every book's committed What's new page is the one its `changes.toml` makes, and every entry is
//! valid against its `SUMMARY.md` (a page it names exists, dates are real and newest first).
//! `./scripts/test-books.sh` writes the pages; this is what fails when someone edits a
//! `changes.toml` and commits without running it, or edits the generated page by hand.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/book-whats-new sits two levels below the repository root")
        .to_path_buf()
}

#[test]
fn every_books_whats_new_page_matches_its_entries() {
    let root = repo_root();
    let books = book_whats_new::books(&root).expect("books/");
    assert!(
        books.len() >= 2,
        "found {books:?}: the scan is not reading books/"
    );
    let mut problems = Vec::new();
    for book in &books {
        match book_whats_new::expected_page(&root, book) {
            Err(e) => problems.push(e),
            Ok(page) => {
                let path = root
                    .join("books")
                    .join(book)
                    .join("src")
                    .join(book_whats_new::PAGE);
                let committed = std::fs::read_to_string(&path).unwrap_or_default();
                if committed != page {
                    problems.push(format!(
                        "books/{book}/src/{} does not match books/{book}/{}: run \
                         `cargo run -p book-whats-new -- generate` (./scripts/test-books.sh does) \
                         and commit the result",
                        book_whats_new::PAGE,
                        book_whats_new::ENTRIES_FILE
                    ));
                }
            }
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}
