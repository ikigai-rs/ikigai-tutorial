//! Text diagrams whose boxes do not close.
//!
//! A chapter draws some of its pictures as box-drawing text in a code block, and a reader
//! with a monospace font sees exactly the columns we typed. A label one space too long
//! pushes that row's right side out of line, and nothing else catches it: the build is
//! green, the page renders, and the picture reads as a mistake (the overlays chapter of
//! the architecture book shipped one). A screen reader reads the label either way; this
//! is about the picture a sighted reader is shown being the one we meant.
//!
//! The check follows every box from its `┌`: along the top edge to its `┐` (a title may
//! sit in the edge, `┌── gonk ──┐`), then down both sides, which must stay in those two
//! columns (`│`, or a `├` `┤` `┼` where a line meets them) until the `└` and `┘` land in
//! the same columns with an unbroken bottom edge between. An arrow may leave through an
//! edge (`▲` `▼`). Trees (`├─ child`) and free arrows (`──▶`) are not boxes and are not
//! checked. A wide character (CJK, most emoji) takes two columns in a monospace font and
//! moves everything after it, so one is a fault anywhere in a block that draws a box.

use std::path::{Path, PathBuf};

/// One box that does not close, in one chapter's source.
#[derive(Debug, PartialEq, Eq)]
pub struct Fault {
    pub file: PathBuf,
    /// 1-based line in the chapter's markdown.
    pub line: usize,
    pub what: String,
}

impl Fault {
    pub fn report(&self) -> String {
        format!("{}:{}: {}", self.file.display(), self.line, self.what)
    }
}

/// What may stand in a horizontal edge.
fn horizontal(c: char) -> bool {
    matches!(c, '─' | '┬' | '┴' | '┼' | '▲' | '▼')
}

/// What may stand in a vertical side.
fn vertical(c: char) -> bool {
    matches!(c, '│' | '├' | '┤' | '┼')
}

/// Whether `c` takes two columns in a monospace font.
fn wide(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x115F | 0x2E80..=0x303E | 0x3041..=0x33FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF | 0xFE30..=0xFE4F | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6 | 0x1F300..=0x1F64F | 0x1F900..=0x1F9FF | 0x20000..=0x3FFFD)
}

/// The faults in one code block, `lines` as characters, as (row in the block, what).
pub fn check_block(lines: &[Vec<char>]) -> Vec<(usize, String)> {
    let at = |r: usize, c: usize| lines.get(r).and_then(|l| l.get(c)).copied().unwrap_or(' ');
    let mut out = Vec::new();
    for (r, line) in lines.iter().enumerate() {
        for (c, &ch) in line.iter().enumerate() {
            if wide(ch) {
                out.push((
                    r,
                    format!("column {c}: the wide character {ch:?} shifts every column after it"),
                ));
            }
            if ch != '┌' {
                continue;
            }
            // The top edge runs to the first ┐ when it starts as an edge (a title may sit
            // in it); otherwise it is only edge characters, and ends where they do.
            let titled = at(r, c + 1) == '─';
            let mut right = c + 1;
            if titled {
                match line[c + 1..].iter().position(|&x| x == '┐') {
                    Some(p) => right = c + 1 + p,
                    None => {
                        while horizontal(at(r, right)) {
                            right += 1
                        }
                    }
                }
            } else {
                while horizontal(at(r, right)) {
                    right += 1
                }
            }
            if at(r, right) != '┐' {
                out.push((
                    r,
                    format!(
                        "column {c}: the top edge from ┌ ends in {:?} at column {right}, not ┐",
                        at(r, right)
                    ),
                ));
                continue;
            }
            let mut row = r + 1;
            loop {
                let (left_side, right_side) = (at(row, c), at(row, right));
                if left_side == '└' {
                    if right_side != '┘' {
                        out.push((row, format!(
                            "the box drawn from line +{r} closes at column {c}, but its ┘ is not at column {right} (found {right_side:?})"
                        )));
                    } else if (c + 1..right).any(|x| !horizontal(at(row, x))) {
                        out.push((
                            row,
                            format!("the bottom edge of the box drawn from line +{r} is broken"),
                        ));
                    }
                    break;
                }
                if !vertical(left_side) || !vertical(right_side) {
                    let (side, column, found) = if vertical(left_side) {
                        ("right", right, right_side)
                    } else {
                        ("left", c, left_side)
                    };
                    out.push((row, format!(
                        "the {side} side of the box drawn from line +{r} breaks at column {column} (found {found:?})"
                    )));
                    break;
                }
                row += 1;
            }
        }
    }
    out
}

/// The faults in one chapter's markdown: every fenced code block that draws a box.
pub fn check_markdown(file: &Path, text: &str) -> Vec<Fault> {
    let lines: Vec<&str> = text.lines().collect();
    let mut faults = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !lines[i].trim_start().starts_with("```") {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < lines.len() && !lines[end].trim_start().starts_with("```") {
            end += 1;
        }
        let block: Vec<Vec<char>> = lines[start..end]
            .iter()
            .map(|l| l.chars().collect())
            .collect();
        if block.iter().any(|l| l.contains(&'┌')) {
            for (row, what) in check_block(&block) {
                faults.push(Fault {
                    file: file.to_path_buf(),
                    line: start + row + 1,
                    what,
                });
            }
        }
        i = end + 1;
    }
    faults
}

/// Every chapter under a book's `src/`, checked.
pub fn check_book(book: &Path) -> Result<Vec<Fault>, String> {
    let mut faults = Vec::new();
    let mut stack = vec![book.join("src")];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                faults.extend(check_markdown(&path, &text));
            }
        }
    }
    faults.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
    Ok(faults)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn faults(block: &str) -> Vec<(usize, String)> {
        let lines: Vec<Vec<char>> = block.lines().map(|l| l.chars().collect()).collect();
        check_block(&lines)
    }

    #[test]
    fn a_closed_box_and_nested_boxes_pass() {
        assert!(faults("┌──┐\n│ a│\n└──┘").is_empty());
        assert!(faults("┌──────┐\n│ ┌──┐ │\n│ │ a│ │\n│ └──┘ │\n└──────┘").is_empty());
    }

    /// The overlays chapter's picture as it shipped: a label one space too long.
    #[test]
    fn a_right_side_out_of_line_is_a_fault() {
        let found = faults("┌────┐\n│ abc  │\n└────┘");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, 1);
        assert!(found[0].1.contains("right side"), "{}", found[0].1);
    }

    #[test]
    fn a_bottom_corner_in_the_wrong_column_is_a_fault() {
        let found = faults("┌───┐\n│ a │\n└────┘");
        assert!(found[0].1.contains("its ┘ is not at column 4"), "{found:?}");
    }

    #[test]
    fn titles_in_edges_and_arrows_through_them_pass() {
        assert!(faults("┌── gonk ──┐\n│          │\n└────▲─────┘").is_empty());
        assert!(faults("┌───┐\n│ a ├──▶ b\n└─┬─┘\n  ▼").is_empty());
    }

    #[test]
    fn trees_and_free_arrows_are_not_boxes() {
        assert!(faults("root\n├─ a\n│  └─ b\n└─ c ──▶ d").is_empty());
    }

    #[test]
    fn a_wide_character_in_a_box_block_is_a_fault() {
        let found = faults("┌──┐\n│漢│\n└──┘");
        assert!(
            found.iter().any(|(_, w)| w.contains("wide character")),
            "{found:?}"
        );
    }

    #[test]
    fn a_fault_names_its_line_in_the_chapter() {
        let text = "# A chapter\n\n```text\n┌──┐\n│ ab │\n└──┘\n```\n";
        let found = check_markdown(Path::new("c.md"), text);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].line, 5);
        assert!(
            found[0].report().starts_with("c.md:5: "),
            "{}",
            found[0].report()
        );
    }
}
