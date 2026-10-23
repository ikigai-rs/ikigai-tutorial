//! The inspector's views, held to snapshots and to the rules the UI conventions state.

mod common;

use std::path::Path;

use common::{kernel, kernel_over, pages, read, viewer, write, B};
use ikigai_core::{Capability, Error};
use ikigai_ui::testkit::{contrast, lint, snapshot};

fn snapshots() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
}

/// Every page, each on a fresh kernel in the same known state, held to its snapshot file. A
/// page that changes fails here, naming the first line that moved.
#[test]
fn every_page_matches_its_snapshot() {
    let names = pages(&kernel());
    for (path, iri) in names {
        let page = read(&kernel(), &iri, &viewer()).unwrap_or_else(|e| panic!("{iri}: {e}"));
        // The detail pages are the card's entry and the note's thread; their paths carry an id,
        // their files a name.
        let file = match path.split('/').nth(2) {
            Some("entry") => "entry-card.html".to_string(),
            Some("thread") => "thread-note.html".to_string(),
            _ => format!("{}.html", path.trim_start_matches("/inspect/")),
        };
        snapshot::assert_snapshot(&snapshots(), &file, &page);
    }
}

/// The ids in a link are a function of the entry's key, so another kernel in the same state
/// links the same entry by the same path: a snapshot is stable across processes.
#[test]
fn an_entry_has_the_same_path_in_every_kernel() {
    let card = format!("{B}demo:card");
    assert_eq!(
        common::entry_page(&kernel(), &card),
        common::entry_page(&kernel(), &card)
    );
}

/// A cut turns a row `cut` while you watch: write the note and the cached card, which hangs
/// from it, is resident but stale.
#[test]
fn writing_the_note_turns_the_cards_entry_cut() {
    let kernel = kernel();
    let row = |cache: &str| {
        let at = cache
            .find(&format!(">{B}demo:card</a>"))
            .expect("the card is listed");
        let tr = cache[..at].rfind("<tr").unwrap();
        cache[tr..at].to_string()
    };
    let before = read(&kernel, &format!("{B}fragment:cache"), &viewer()).unwrap();
    assert!(row(&before).contains("state-live"), "{}", row(&before));
    write(&kernel, &format!("{B}demo:note"), "changed", &viewer()).unwrap();
    let after = read(&kernel, &format!("{B}fragment:cache"), &viewer()).unwrap();
    assert!(row(&after).contains("state-cut"), "{}", row(&after));
    // And the threads view counts the cut.
    let threads = read(
        &kernel,
        &format!("{B}fragment:thread:{B}demo:note"),
        &viewer(),
    )
    .unwrap();
    assert!(threads.contains("Cut 1 times."), "{threads}");
}

/// The inspector shows only what its viewer may inspect: the kernel refuses the readout, and
/// the refusal reaches the viewer as the kernel's own, a denial.
#[test]
fn a_viewer_without_inspect_is_refused_by_the_kernel() {
    let kernel = kernel();
    let reader = Capability::root().attenuate(["urn:cap:iki:tutorial:inspect:note"]);
    for view in ["cache", "threads", "topology", "catalog"] {
        let err = read(&kernel, &format!("{B}fragment:{view}"), &reader).unwrap_err();
        assert!(matches!(err, Error::Denied(_)), "{view}: {err}");
    }
    // The note's card needs no inspection, and still answers.
    assert!(read(&kernel, &format!("{B}demo:card"), &reader).is_ok());
}

/// A note is data: what a viewer writes is escaped by `$h` wherever it is shown.
#[test]
fn a_note_is_escaped_where_it_is_shown() {
    let kernel = kernel();
    write(
        &kernel,
        &format!("{B}demo:note"),
        "<script>alert(1)</script>",
        &viewer(),
    )
    .unwrap();
    let card = read(&kernel, &format!("{B}demo:card"), &viewer()).unwrap();
    assert!(
        card.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "{card}"
    );
    assert!(!card.contains("<script>"), "{card}");
}

/// A detail the cache does not hold is NotFound, saying why, not an empty page.
#[test]
fn an_entry_the_cache_does_not_hold_is_not_found() {
    let err = read(
        &kernel(),
        &format!("{B}fragment:entry:0123456789abcdef"),
        &viewer(),
    )
    .unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "{err}");
    assert!(err.to_string().contains("may have been evicted"), "{err}");
}

/// Two deliveries, one set of names (D17): every artifact read from the committed directory
/// is byte for byte the compiled-in one, and so are the pages that read no kernel readout.
#[test]
fn the_directory_and_the_compiled_in_artifacts_are_the_same_bytes() {
    let compiled = kernel();
    let (from_dir, cap) = kernel_over(&common::artifacts(), false);
    for a in &inspector::ARTIFACTS {
        let one = read(&compiled, a.iri, &viewer()).unwrap();
        let two = read(&from_dir, a.iri, &cap).unwrap_or_else(|e| panic!("{}: {e}", a.iri));
        assert_eq!(one, two, "{}", a.iri);
        assert_eq!(one.as_bytes(), a.bytes, "{}", a.iri);
    }
    read(&from_dir, &format!("{B}demo:card"), &cap).unwrap();
    for page in ["scheduler", "note"] {
        let iri = format!("{B}page:{page}");
        assert_eq!(
            read(&compiled, &iri, &viewer()).unwrap(),
            read(&from_dir, &iri, &cap).unwrap(),
            "{iri}"
        );
    }
}

/// A file in the directory replaces its artifact, and only that one; a write to it through the
/// kernel cuts its thread, so the next page is recomputed over the new file (hot reload).
#[test]
fn a_file_in_the_directory_replaces_one_artifact_and_a_write_reloads_it() {
    let dir = std::env::temp_dir().join(format!("inspector-dir-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("header.html"),
        "<nav aria-label=\"Inspector\">one</nav>",
    )
    .unwrap();
    let (kernel, cap) = kernel_over(&dir, true);
    let page = read(&kernel, &format!("{B}page:scheduler"), &cap).unwrap();
    assert!(
        page.contains("<nav aria-label=\"Inspector\">one</nav>"),
        "{page}"
    );
    // Everything else is still compiled in.
    let compiled = inspector::artifact(&format!("{B}template:scheduler")).unwrap();
    assert!(page.contains(
        std::str::from_utf8(compiled.bytes)
            .unwrap()
            .lines()
            .nth(1)
            .unwrap()
    ));
    write(
        &kernel,
        &format!("{}header.html", inspector::DIR),
        "<nav aria-label=\"Inspector\">two</nav>",
        &cap,
    )
    .unwrap();
    let page = read(&kernel, &format!("{B}page:scheduler"), &cap).unwrap();
    assert!(page.contains(">two</nav>"), "{page}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The contrast floor and the token lints (D12, D18): the palettes clear their floors, the
/// inspector's stylesheet names no color, and every custom property it uses is defined.
#[test]
fn the_stylesheets_name_no_color_and_use_only_defined_tokens() {
    assert!(contrast::failures(&ikigai_ui::palette::PALETTES).is_empty());
    let kernel = kernel();
    let tokens = read(&kernel, ikigai_ui::TOKENS_CSS, &viewer()).unwrap();
    let ours = read(&kernel, &format!("{B}inspector.css"), &viewer()).unwrap();
    assert_eq!(lint::hex_colors(&ours), Vec::<String>::new());
    assert!(
        lint::undefined_vars(&[&tokens, &ours]).is_empty(),
        "{:?}",
        lint::undefined_vars(&[&tokens, &ours])
    );
}

/// Every stylesheet stays inside the XSLT xrust evaluates right (ikigai-xslt refuses the rest
/// at run time; this says so before anyone reads a page).
#[test]
fn every_stylesheet_uses_only_what_xrust_gets_right() {
    for a in inspector::ARTIFACTS
        .iter()
        .filter(|a| a.file.ends_with(".xsl"))
    {
        let text = std::str::from_utf8(a.bytes).unwrap();
        let silent = ikigai_xslt::subset::silent_constructs(text).unwrap();
        assert!(silent.is_empty(), "{}: {silent:?}", a.file);
        assert!(!text.contains("disable-output-escaping"), "{}", a.file);
    }
}

/// Fragment and page are separate resources (D8): the page carries the fragment, byte for byte,
/// inside its <main>.
#[test]
fn the_page_composes_the_fragment() {
    for view in ["threads", "scheduler", "note"] {
        let fragment = read(&kernel(), &format!("{B}fragment:{view}"), &viewer()).unwrap();
        let page = read(&kernel(), &format!("{B}page:{view}"), &viewer()).unwrap();
        assert!(
            page.contains(&format!(
                "<main id=\"main\" tabindex=\"-1\">\n{fragment}\n</main>"
            )),
            "{view}"
        );
    }
}
