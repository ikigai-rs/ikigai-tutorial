//! The library: one page that lists every book in this repository.
//!
//! The library is a book of its own, `books/library/`, published at `/ikigai-tutorial/library/`,
//! so every gate every book passes (contrast, axe in jsdom and in a browser in each theme, the link
//! check) covers it by its being there. Its one real page, `src/index.md`, is GENERATED: written
//! by [`render`] from `books/library/library.toml` (one `[[book]]` entry per book: its
//! description, its audience, where to start and what to read first) and from each book's own
//! `book.toml` (its title and where it is served), so nothing about a book is written twice.
//!
//! A fourth book is one more directory under `books/` and one more entry. Without the entry,
//! [`expected_page`] refuses, so `cargo test` (and `./scripts/test-books.sh`) fail rather than
//! publishing a library that has lost a book.
//!
//! ```
//! let entries = book_library::parse_library(r#"
//! intro = "Every book."
//! [[book]]
//! dir = "demo"
//! description = "A demo."
//! audience = "Anyone."
//! start = "introduction.html"
//! start_title = "Introduction"
//! read_first = "Nothing."
//! "#).unwrap();
//! assert_eq!(entries.books[0].dir, "demo");
//! ```

use std::path::Path;

/// The library's own directory under `books/`.
pub const LIBRARY: &str = "library";

/// The data file, beside the library's `book.toml`.
pub const DATA_FILE: &str = "library.toml";

/// The generated page, under the library's `src/`.
pub const PAGE: &str = "index.md";

/// The library's data: an opening paragraph and one entry per book, in the order shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    /// The paragraph under the page's heading.
    pub intro: String,
    /// One entry per book.
    pub books: Vec<Entry>,
}

/// One book's entry. Its title and URL come from its own `book.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Its directory under `books/`.
    pub dir: String,
    /// One paragraph: what the book is.
    pub description: String,
    /// Who it is for.
    pub audience: String,
    /// The page to start at, relative to the book's root (`front-door/cli.html`).
    pub start: String,
    /// That page's title, as the book's SUMMARY.md gives it.
    pub start_title: String,
    /// What to have read first.
    pub read_first: String,
}

const FIELDS: [&str; 6] = [
    "dir",
    "description",
    "audience",
    "start",
    "start_title",
    "read_first",
];

/// Parse `library.toml`. Every mistake names the entry and the field.
pub fn parse_library(text: &str) -> Result<Library, String> {
    let table: toml::Table = toml::from_str(text).map_err(|e| format!("not TOML: {e}"))?;
    for key in table.keys() {
        if key != "intro" && key != "book" {
            return Err(format!(
                "unknown top-level key `{key}` (the keys are `intro` and `[[book]]`)"
            ));
        }
    }
    let intro = table
        .get("intro")
        .and_then(toml::Value::as_str)
        .ok_or("`intro` is required, a string")?
        .to_string();
    let list = table
        .get("book")
        .and_then(toml::Value::as_array)
        .ok_or("`book` must be an array of tables, written `[[book]]`")?;
    let mut books = Vec::with_capacity(list.len());
    for (i, item) in list.iter().enumerate() {
        let t = item
            .as_table()
            .ok_or_else(|| format!("entry {}: must be a table, written `[[book]]`", i + 1))?;
        let at = format!(
            "entry {} ({})",
            i + 1,
            t.get("dir")
                .and_then(toml::Value::as_str)
                .unwrap_or("no dir")
        );
        for key in t.keys() {
            if !FIELDS.contains(&key.as_str()) {
                return Err(format!(
                    "{at}: unknown field `{key}` (the fields are {})",
                    FIELDS.join(", ")
                ));
            }
        }
        let field = |key: &str| -> Result<String, String> {
            t.get(key)
                .and_then(toml::Value::as_str)
                .map(str::to_string)
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| format!("{at}: `{key}` is required, a non-empty string"))
        };
        books.push(Entry {
            dir: field("dir")?,
            description: field("description")?,
            audience: field("audience")?,
            start: field("start")?,
            start_title: field("start_title")?,
            read_first: field("read_first")?,
        });
    }
    Ok(Library { intro, books })
}

/// A book's title and the URL it is served at, from its `book.toml`.
pub fn book_meta(root: &Path, dir: &str) -> Result<(String, String), String> {
    let path = root.join("books").join(dir).join("book.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let table: toml::Table =
        toml::from_str(&text).map_err(|e| format!("{}: not TOML: {e}", path.display()))?;
    let title = table
        .get("book")
        .and_then(|b| b.get("title"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("{}: no [book] title", path.display()))?;
    let url = table
        .get("output")
        .and_then(|o| o.get("html"))
        .and_then(|h| h.get("site-url"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("{}: no [output.html] site-url", path.display()))?;
    Ok((title.to_string(), url.to_string()))
}

/// The relative path from the page at site URL `from` (a directory, ending `/`) to the directory
/// `to`.
///
/// ```
/// use book_library::relative;
/// assert_eq!(relative("/ikigai-tutorial/library/", "/ikigai-tutorial/"), "../");
/// assert_eq!(relative("/ikigai-tutorial/library/", "/ikigai-tutorial/gonk/"), "../gonk/");
/// ```
pub fn relative(from: &str, to: &str) -> String {
    let parts = |s: &str| -> Vec<String> {
        s.split('/')
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect()
    };
    let (from, to) = (parts(from), parts(to));
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut out = "../".repeat(from.len() - common);
    for p in &to[common..] {
        out.push_str(p);
        out.push('/');
    }
    out
}

/// Every book under `root/books/` but the library itself: the directories holding a `book.toml`,
/// sorted. A dot-directory is a working copy, not a book.
pub fn books(root: &Path) -> Result<Vec<String>, String> {
    let dir = root.join("books");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with('.') && name != LIBRARY && entry.path().join("book.toml").is_file() {
            out.push(name);
        }
    }
    out.sort();
    Ok(out)
}

/// The library page, as markdown. `books` is each entry with its title and the relative link to
/// its root.
pub fn render(library: &Library, books: &[(Entry, String, String)]) -> String {
    let mut out = String::from(
        "<!-- GENERATED by crates/book-library from books/library/library.toml and each book's \
         book.toml. Edit those, then run ./scripts/test-books.sh (or `cargo run -p book-library \
         -- generate`). -->\n\n# The ikigai library\n\n",
    );
    out.push_str(library.intro.trim());
    out.push_str("\n\n");
    for (entry, title, href) in books {
        // The title is plain text: mdbook makes every heading a link to itself, and a link
        // inside it would leave that one with no text (axe's link-name).
        out.push_str(&format!("## {title}\n\n"));
        out.push_str(entry.description.trim());
        out.push_str(&format!(" [Open {title}]({href}index.html).\n\n"));
        out.push_str(&format!("- **Who it is for:** {}\n", entry.audience.trim()));
        out.push_str(&format!("- **Read first:** {}\n", entry.read_first.trim()));
        out.push_str(&format!(
            "- **Start at:** [{}]({href}{})\n\n",
            entry.start_title.trim(),
            entry.start.trim()
        ));
    }
    out.push_str(
        "The books are built, tested and published together, from one repository: \
         <https://github.com/ikigai-rs/ikigai-tutorial>.\n",
    );
    out
}

/// The page the library's files say it should have, or everything wrong with them: an entry for
/// a directory that is not a book, a book with no entry, or a book listed twice.
pub fn expected_page(root: &Path) -> Result<String, String> {
    let lib_dir = root.join("books").join(LIBRARY);
    let data = lib_dir.join(DATA_FILE);
    let text = std::fs::read_to_string(&data).map_err(|e| format!("{}: {e}", data.display()))?;
    let library = parse_library(&text).map_err(|e| format!("books/{LIBRARY}/{DATA_FILE}: {e}"))?;
    let (_, here) = book_meta(root, LIBRARY)?;
    let on_disk = books(root)?;
    let mut problems = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for entry in &library.books {
        if seen.contains(&entry.dir.as_str()) {
            problems.push(format!("books/{} is listed twice", entry.dir));
        }
        seen.push(&entry.dir);
        if !on_disk.contains(&entry.dir) {
            problems.push(format!(
                "an entry names books/{}, which is not a book (no book.toml)",
                entry.dir
            ));
        }
    }
    for book in &on_disk {
        if !seen.contains(&book.as_str()) {
            problems.push(format!(
                "books/{book} has no entry in books/{LIBRARY}/{DATA_FILE}: add a [[book]] with \
                 dir = \"{book}\", so the library lists it"
            ));
        }
    }
    if !problems.is_empty() {
        return Err(problems.join("\n"));
    }
    let mut rows = Vec::new();
    for entry in &library.books {
        let (title, url) = book_meta(root, &entry.dir)?;
        let href = relative(&here, &url);
        rows.push((entry.clone(), title, href));
    }
    Ok(render(&library, &rows))
}

/// Check an ASSEMBLED site (`scripts/assemble-site.sh`'s directory): the library page exists
/// where the library's `site-url` puts it, and it links the root page of every book.
pub fn check_site(root: &Path, site: &Path) -> Result<usize, String> {
    let (_, here) = book_meta(root, LIBRARY)?;
    let mut base = None;
    let mut urls = Vec::new();
    for book in books(root)? {
        let (title, url) = book_meta(root, &book)?;
        if base.as_ref().is_none_or(|b: &String| url.len() < b.len()) {
            base = Some(url.clone());
        }
        urls.push((book, title, url));
    }
    let base = base.ok_or("no books under books/")?;
    let rel = here
        .strip_prefix(&base)
        .ok_or_else(|| format!("the library's site-url {here} is outside the site root {base}"))?;
    let page = site.join(rel).join("index.html");
    let html = std::fs::read_to_string(&page)
        .map_err(|e| format!("the library page {} is missing: {e}", page.display()))?;
    let mut missing = Vec::new();
    for (book, title, url) in &urls {
        let href = format!("href=\"{}index.html\"", relative(&here, url));
        if !html.contains(&href) {
            missing.push(format!("{title} (books/{book}): no {href}"));
        }
    }
    if missing.is_empty() {
        Ok(urls.len())
    } else {
        Err(format!(
            "{} does not link every book:\n  {}",
            page.display(),
            missing.join("\n  ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_field_is_named() {
        let err = parse_library(
            "intro = \"x\"\n[[book]]\ndir = \"a\"\ndescription = \"d\"\naudience = \"a\"\n\
             start = \"s\"\nstart_title = \"t\"\nread_first = \"r\"\nshelf = \"x\"\n",
        )
        .unwrap_err();
        assert!(err.contains("entry 1 (a): unknown field `shelf`"), "{err}");
    }

    #[test]
    fn a_missing_field_is_named() {
        let err = parse_library("intro = \"x\"\n[[book]]\ndir = \"a\"\n").unwrap_err();
        assert!(err.contains("`description` is required"), "{err}");
    }

    #[test]
    fn relative_paths_climb_and_descend() {
        assert_eq!(relative("/a/b/", "/a/b/"), "");
        assert_eq!(relative("/a/b/", "/a/c/d/"), "../c/d/");
    }
}
