//! The committed library page is the one `library.toml` and the books' own `book.toml` files
//! make, and every book under `books/` has an entry. `./scripts/test-books.sh` writes the page;
//! this is what fails when a fourth book arrives without an entry, or someone edits the page by
//! hand.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/book-library sits two levels below the repository root")
        .to_path_buf()
}

#[test]
fn the_library_lists_every_book_and_its_page_is_current() {
    let root = repo_root();
    let books = book_library::books(&root).expect("books/");
    assert!(
        books.len() >= 3,
        "found {books:?}: the scan is not reading books/"
    );
    let page = book_library::expected_page(&root).unwrap_or_else(|e| panic!("\n{e}"));
    for book in &books {
        let (title, _) = book_library::book_meta(&root, book).unwrap();
        assert!(
            page.contains(&format!("## {title}\n")),
            "{title} is not on the page"
        );
        assert!(
            page.contains(&format!("[Open {title}](")),
            "{title} is not linked"
        );
    }
    let committed =
        std::fs::read_to_string(root.join("books/library/src").join(book_library::PAGE))
            .unwrap_or_default();
    assert!(
        committed == page,
        "books/library/src/index.md does not match books/library/library.toml: run \
         `cargo run -p book-library -- generate` (./scripts/test-books.sh does) and commit it"
    );
}

/// A book with no entry is refused, by name: the check the brief asks of the gate.
#[test]
fn a_book_without_an_entry_is_refused() {
    let dir = std::env::temp_dir().join(format!("book-library-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (book, url) in [
        ("library", "/x/library/"),
        ("one", "/x/"),
        ("two", "/x/two/"),
    ] {
        std::fs::create_dir_all(dir.join("books").join(book).join("src")).unwrap();
        std::fs::write(
            dir.join("books").join(book).join("book.toml"),
            format!("[book]\ntitle = \"{book}\"\n[output.html]\nsite-url = \"{url}\"\n"),
        )
        .unwrap();
    }
    std::fs::write(
        dir.join("books/library/library.toml"),
        "intro = \"x\"\n[[book]]\ndir = \"one\"\ndescription = \"d\"\naudience = \"a\"\n\
         start = \"s.html\"\nstart_title = \"S\"\nread_first = \"r\"\n",
    )
    .unwrap();
    let err = book_library::expected_page(&dir).unwrap_err();
    assert!(err.contains("books/two has no entry"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}
