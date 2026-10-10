//! No runnable cell in either book carries a blank line (ledger #614).
//!
//! A blank line inside a cell ends mdbook's HTML block, and the rest of the cell renders as
//! markdown: later cells nest inside this one's expected output, or the output turns into a
//! paragraph, or (in `data-cmd`) the cell is dropped. mdbook builds the page without a
//! warning, the URN gate passes (every name is still there), and axe has nothing to say; the
//! spreadsheet arc found it only because `a11y/no-kernel.mjs` counted one sheet of three. See
//! `book_urns::blank_lines_in_cells` for the rule and the cure, and `book-a11y`, which checks
//! the built page for the same thing from the other end.
//!
//! Every page under `books/*/src` is scanned, drafts included: a draft's cells render in the
//! drafts build, and it is cheaper to refuse the line before the chapter is linked.

use std::path::{Path, PathBuf};

use book_urns::{blank_lines_in_cells, CellPart};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root is two levels up")
}

/// Every `.md` under `dir`, skipping vendored files and dot-directories (a drafts server's
/// scratch copy lives in `books/.drafts.*`, outside every `src`, but be explicit).
fn markdown(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
    {
        let path = entry.expect("a directory entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') || name == "vendor" {
            continue;
        }
        if path.is_dir() {
            markdown(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

#[test]
fn no_runnable_cell_has_a_blank_line_in_it() {
    let root = repo_root();
    let mut files = Vec::new();
    for book in std::fs::read_dir(root.join("books")).expect("books/") {
        let src = book.expect("a book").path().join("src");
        if src.is_dir() {
            markdown(&src, &mut files);
        }
    }
    files.sort();

    let mut cells = 0usize;
    let mut faults = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        cells += text.matches("<div class=\"ikigai-run\"").count();
        let site = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        for blank in blank_lines_in_cells(&text) {
            faults.push(match blank.part {
                CellPart::ExpectedOutput => format!(
                    "{site}:{}: a blank line inside the expected output of the cell at line {} \
                     — write `&#32;` on the line instead (it renders as a space and does not \
                     end the HTML block)",
                    blank.line, blank.cell
                ),
                CellPart::Markup => format!(
                    "{site}:{}: a blank line inside the cell at line {}, outside its expected \
                     output (its data-cmd) — delete the line; a command list needs no blank line",
                    blank.line, blank.cell
                ),
            });
        }
    }

    // A scan that saw no cells proves nothing: the books have dozens.
    assert!(
        cells >= 20,
        "found only {cells} runnable cells under books/*/src — the scan is not reading the books"
    );
    assert!(
        faults.is_empty(),
        "a blank line inside a runnable cell ends mdbook's HTML block, and the rest of the \
         cell (sometimes the rest of the page) renders as markdown, silently:\n  {}",
        faults.join("\n  ")
    );
}
