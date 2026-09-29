//! The sheet, end to end, through a real kernel: the atoms, the compiled formulas, and
//! recalculation as golden threads.

use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Kernel, Request, Verb};
use spreadsheet::{kernel, kernel_over, InputStore};

fn request(verb: Verb, name: &str) -> Request {
    Request::new(verb, Iri::parse(name).expect("a name"))
}

fn source(kernel: &Kernel, name: &str) -> Result<String, Error> {
    block_on(kernel.issue(request(Verb::Source, name), &Capability::root()))
        .map(|r| String::from_utf8_lossy(&r.bytes).into_owned())
}

fn value(kernel: &Kernel, cell: &str) -> String {
    source(kernel, &format!("urn:iki:tutorial:sheet:cell:{cell}"))
        .unwrap_or_else(|e| panic!("cell {cell}: {e}"))
}

fn type_into(kernel: &Kernel, cell: &str, typed: &str) {
    let sink = request(Verb::Sink, &format!("urn:iki:tutorial:sheet:input:{cell}"))
        .with_arg("content", ArgRef::Inline(typed.as_bytes().to_vec()));
    block_on(kernel.issue(sink, &Capability::root()))
        .unwrap_or_else(|e| panic!("typing {typed} into {cell}: {e}"));
}

fn clear(kernel: &Kernel, cell: &str) {
    let delete = request(
        Verb::Delete,
        &format!("urn:iki:tutorial:sheet:input:{cell}"),
    );
    block_on(kernel.issue(delete, &Capability::root())).expect("a clear");
}

fn cached(kernel: &Kernel, name: &str) -> bool {
    kernel.is_cached(&request(Verb::Source, name), &Capability::root())
}

fn value_cached(kernel: &Kernel, cell: &str) -> bool {
    cached(kernel, &format!("urn:iki:tutorial:sheet:cell:{cell}"))
}

// ANCHOR: atoms
#[test]
fn an_input_is_an_atom_and_a_literal_value_passes_through() {
    let kernel = kernel();
    // Nothing typed: the input is NotFound, and the value is empty — computed, then served.
    assert!(matches!(
        source(&kernel, "urn:iki:tutorial:sheet:input:A1"),
        Err(Error::NotFound(_))
    ));
    assert_eq!(value(&kernel, "A1"), "");
    assert!(value_cached(&kernel, "A1"));

    // Typing cuts the value built on the empty cell, with no code here to say so.
    type_into(&kernel, "A1", "  42 ");
    assert!(!value_cached(&kernel, "A1"));
    assert_eq!(value(&kernel, "A1"), "42");
    assert_eq!(
        source(&kernel, "urn:iki:tutorial:sheet:input:A1").unwrap(),
        "42"
    );

    type_into(&kernel, "B1", "Rent");
    assert_eq!(value(&kernel, "B1"), "Rent");
    clear(&kernel, "B1");
    assert_eq!(value(&kernel, "B1"), "");
}
// ANCHOR_END: atoms

#[test]
fn a_cell_has_one_name_and_it_is_on_the_sheet() {
    let kernel = kernel();
    for (name, detail) in [
        ("urn:iki:tutorial:sheet:cell:a1", "`a1` is not a cell"),
        ("urn:iki:tutorial:sheet:cell:A01", "`A01` is not a cell"),
        (
            "urn:iki:tutorial:sheet:cell:E1",
            "E1 is off the sheet, which is columns A–D and rows 1–6",
        ),
        ("urn:iki:tutorial:sheet:input:A7", "A7 is off the sheet"),
    ] {
        match source(&kernel, name) {
            Err(Error::InvalidArgument {
                name: arg,
                detail: said,
            }) => {
                assert_eq!(arg, "ref");
                assert!(said.starts_with(detail), "{said}");
            }
            other => panic!("{name}: {other:?}"),
        }
    }
    let empty = request(Verb::Sink, "urn:iki:tutorial:sheet:input:A1")
        .with_arg("content", ArgRef::Inline(b"   ".to_vec()));
    assert!(matches!(
        block_on(kernel.issue(empty, &Capability::root())),
        Err(Error::InvalidArgument { .. })
    ));
}

#[test]
fn a_formula_is_served_as_a_formula() {
    let kernel = kernel();
    type_into(&kernel, "A2", "=A1*2");
    let typed = block_on(kernel.issue(
        request(Verb::Source, "urn:iki:tutorial:sheet:input:A2"),
        &Capability::root(),
    ))
    .unwrap();
    assert_eq!(typed.repr_type.media_type, "text/x-formula");
    type_into(&kernel, "A1", "5");
    let literal = block_on(kernel.issue(
        request(Verb::Source, "urn:iki:tutorial:sheet:input:A1"),
        &Capability::root(),
    ))
    .unwrap();
    assert_eq!(literal.repr_type.media_type, "text/plain");
}

// ANCHOR: compiled
#[test]
fn a_formula_is_compiled_once_and_again_only_when_it_changes() {
    let kernel = kernel();
    type_into(&kernel, "A2", "=a1 * 2");
    type_into(&kernel, "B2", "=A2+1");
    let formula = "urn:iki:tutorial:sheet:formula:A2";
    assert_eq!(source(&kernel, formula).unwrap(), "(* A1 2)");
    assert!(cached(&kernel, formula));

    // Another cell's formula changes: this one stays compiled.
    type_into(&kernel, "B2", "=A2+10");
    assert!(cached(&kernel, formula));
    // A cell it names changes: still compiled, since the formula did not change.
    type_into(&kernel, "A1", "7");
    assert!(cached(&kernel, formula));
    // Its own input changes: recompiled.
    type_into(&kernel, "A2", "=A1*3");
    assert!(!cached(&kernel, formula));
    assert_eq!(source(&kernel, formula).unwrap(), "(* A1 3)");
}
// ANCHOR_END: compiled

#[test]
fn the_compiler_is_found_by_what_it_declares() {
    let kernel = kernel();
    let plan = kernel
        .select_transreptor_in_with(
            spreadsheet::FORMULA_TYPE,
            spreadsheet::SEXPR_TYPE,
            &ikigai_core::Scope::empty(),
            &ikigai_core::TransreptionPolicy::allow_lossy(),
        )
        .expect("a plan");
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].endpoint, spreadsheet::COMPILE);
    assert!(!plan[0].lossless, "compiling forgets the spelling");
    // Without consent to a lossy step there is no plan: the compiler is a projection.
    assert!(kernel
        .select_transreptor_in(
            spreadsheet::FORMULA_TYPE,
            spreadsheet::SEXPR_TYPE,
            &ikigai_core::Scope::empty()
        )
        .is_none());
}

#[test]
fn a_cell_with_no_formula_has_no_compiled_form_and_reads_nothing() {
    let kernel = kernel();
    type_into(&kernel, "A1", "5");
    assert!(matches!(
        source(&kernel, "urn:iki:tutorial:sheet:formula:A1"),
        Err(Error::NotFound(_))
    ));
    assert!(matches!(
        source(&kernel, "urn:iki:tutorial:sheet:formula:C3"),
        Err(Error::NotFound(_))
    ));
    assert_eq!(
        source(&kernel, "urn:iki:tutorial:sheet:refs:A1").unwrap(),
        ""
    );
    assert_eq!(
        source(&kernel, "urn:iki:tutorial:sheet:refs:C3").unwrap(),
        ""
    );
}

#[test]
fn refs_list_the_cells_a_formula_reads_and_a_range_its_rectangle() {
    let kernel = kernel();
    assert_eq!(
        source(&kernel, "urn:iki:tutorial:sheet:range:A1:B2").unwrap(),
        "urn:iki:tutorial:sheet:cell:A1\nurn:iki:tutorial:sheet:cell:B1\n\
         urn:iki:tutorial:sheet:cell:A2\nurn:iki:tutorial:sheet:cell:B2"
    );
    assert!(matches!(
        source(&kernel, "urn:iki:tutorial:sheet:range:B2:A1"),
        Err(Error::InvalidArgument { detail, .. }) if detail.ends_with("A1:B2")
    ));
    type_into(&kernel, "C1", "=SUM(A1:A2) + B1 + E1 + A1");
    assert_eq!(
        source(&kernel, "urn:iki:tutorial:sheet:refs:C1").unwrap(),
        "urn:iki:tutorial:sheet:cell:A1\nurn:iki:tutorial:sheet:cell:A2\n\
         urn:iki:tutorial:sheet:cell:B1"
    );
}

// ANCHOR: values
#[test]
fn formulas_evaluate_and_errors_are_values() {
    let kernel = kernel();
    type_into(&kernel, "A1", "10");
    type_into(&kernel, "A2", "4");
    type_into(&kernel, "A3", "Rent");
    for (formula, expected) in [
        ("=A1*2+1", "21"),
        ("=A1/A2", "2.5"),
        ("=-(A1-A2)", "-6"),
        ("=A4+1", "1"), // an empty cell is 0
        ("=A4", "0"),
        ("=A3", "Rent"),       // a formula may show text
        ("=SUM(A1:A4)", "14"), // SUM skips text and empty cells
        ("=SUM(A1:A2, 6, A1*2)", "40"),
        ("=A1/(A2-4)", "#DIV/0"),
        ("=A3*2", "#VALUE"), // text in arithmetic
        ("=E1+1", "#REF"),   // off the sheet
        ("=SUM(A1:E1)", "#REF"),
        ("=A1+", "#SYNTAX"),
        ("=MAX(A1)", "#SYNTAX"),
    ] {
        type_into(&kernel, "B1", formula);
        assert_eq!(value(&kernel, "B1"), expected, "{formula}");
    }
    // An error propagates to what reads it.
    type_into(&kernel, "B1", "=A1/0");
    type_into(&kernel, "C1", "=B1+1");
    type_into(&kernel, "D1", "=SUM(B1:C1)");
    assert_eq!(value(&kernel, "C1"), "#DIV/0");
    assert_eq!(value(&kernel, "D1"), "#DIV/0");
}
// ANCHOR_END: values

#[test]
fn a_syntax_error_is_a_value_the_kernel_can_cache() {
    let kernel = kernel();
    type_into(&kernel, "A1", "=1+");
    assert_eq!(
        source(&kernel, "urn:iki:tutorial:sheet:formula:A1").unwrap(),
        "(syntax-error \"expected a number, a cell or (, and the formula ended\")"
    );
    assert_eq!(value(&kernel, "A1"), "#SYNTAX");
    assert!(value_cached(&kernel, "A1"));
}

// ANCHOR: cycle
#[test]
fn a_cycle_is_named_not_chased() {
    let kernel = kernel();
    type_into(&kernel, "A1", "=B1+1");
    type_into(&kernel, "B1", "=C1+1");
    type_into(&kernel, "C1", "=A1+1");
    type_into(&kernel, "D1", "=C1*2");
    for cell in ["A1", "B1", "C1"] {
        assert_eq!(value(&kernel, cell), "#CYCLE", "{cell}");
    }
    // D1 is not in the cycle; it reads a cell that is, and says so.
    assert_eq!(value(&kernel, "D1"), "#CYCLE");
    assert_eq!(
        source(&kernel, "urn:iki:tutorial:sheet:precedents:A1").unwrap(),
        "urn:iki:tutorial:sheet:cell:B1\nurn:iki:tutorial:sheet:cell:C1\n\
         urn:iki:tutorial:sheet:cell:A1"
    );
    // Break the cycle, and everything round it recomputes from the new input.
    type_into(&kernel, "C1", "1");
    assert_eq!(value(&kernel, "A1"), "3");
    assert_eq!(value(&kernel, "D1"), "2");
    // A cell that names itself is the shortest cycle.
    type_into(&kernel, "A2", "=A2");
    assert_eq!(value(&kernel, "A2"), "#CYCLE");
}
// ANCHOR_END: cycle

// ANCHOR: recalc
#[test]
fn an_edit_recomputes_exactly_its_dependents() {
    let kernel = kernel();
    for (cell, typed) in [
        ("A1", "5"),
        ("A2", "=A1*2"),
        ("A3", "=A2+1"),
        ("B1", "100"),
        ("B2", "=SUM(A1:A3)"),
        ("C1", "=B1/4"),
    ] {
        type_into(&kernel, cell, typed);
    }
    let every = ["A1", "A2", "A3", "B1", "B2", "C1", "D6"];
    let values: Vec<String> = every.iter().map(|c| value(&kernel, c)).collect();
    assert_eq!(values, ["5", "10", "11", "100", "26", "25", ""]);
    assert!(every.iter().all(|c| value_cached(&kernel, c)));

    // Edit A1. Its value and every value that read it, directly or not, is cut; the rest
    // stay cached. Nothing listed the dependents: they are the threads.
    type_into(&kernel, "A1", "7");
    let cut: Vec<&str> = every
        .iter()
        .copied()
        .filter(|c| !value_cached(&kernel, c))
        .collect();
    assert_eq!(cut, ["A1", "A2", "A3", "B2"]);
    // Read again, the sum is 7 + 14 + 15, and the four are cached again.
    assert_eq!(value(&kernel, "B2"), "36");
    assert!(every.iter().all(|c| value_cached(&kernel, c)));

    // Edit B1: only B1 and C1, which reads it.
    type_into(&kernel, "B1", "8");
    let cut: Vec<&str> = every
        .iter()
        .copied()
        .filter(|c| !value_cached(&kernel, c))
        .collect();
    assert_eq!(cut, ["B1", "C1"]);
    assert_eq!(value(&kernel, "C1"), "2");
}
// ANCHOR_END: recalc

#[test]
fn typing_into_an_empty_cell_cuts_what_was_built_on_its_emptiness() {
    let kernel = kernel();
    type_into(&kernel, "C3", "=D1+1");
    assert_eq!(value(&kernel, "C3"), "1");
    type_into(&kernel, "D1", "5");
    assert!(!value_cached(&kernel, "C3"));
    assert_eq!(value(&kernel, "C3"), "6");
    // …and a range over it, too.
    type_into(&kernel, "C4", "=SUM(D1:D6)");
    assert_eq!(value(&kernel, "C4"), "5");
    type_into(&kernel, "D6", "10");
    assert_eq!(value(&kernel, "C4"), "15");
}

#[test]
fn the_grid_escapes_what_was_typed_and_resolves_none_of_it() {
    let kernel = kernel();
    type_into(&kernel, "A1", "<b>bold</b> & \"quoted\"");
    type_into(&kernel, "B1", "$a{urn:iki:tutorial:sheet:input:A1}");
    type_into(&kernel, "C1", "=2*3");
    let grid = source(&kernel, spreadsheet::VIEW_GRID).unwrap();
    assert!(
        grid.contains("<td>&lt;b&gt;bold&lt;/b&gt; &amp; &quot;quoted&quot;</td>"),
        "{grid}"
    );
    assert!(
        grid.contains("<td>&#36;a{urn:iki:tutorial:sheet:input:A1}</td>"),
        "{grid}"
    );
    assert!(grid.contains("<th scope=\"row\">1</th><td>"), "{grid}");
    assert!(grid.contains("<td>6</td>"), "{grid}");
    // Twenty-four cells, and the empty corner above the row numbers.
    assert_eq!(grid.matches("<td>").count(), 25, "{grid}");
    assert!(!grid.contains("$a{"), "{grid}");
}

fn edit(kernel: &Kernel, cell: &str, typed: &str) -> String {
    let sink = request(Verb::Sink, spreadsheet::VIEW_EDIT)
        .with_arg("ref", ArgRef::Inline(cell.as_bytes().to_vec()))
        .with_arg("content", ArgRef::Inline(typed.as_bytes().to_vec()));
    let reply = block_on(kernel.issue(sink, &Capability::root())).expect("an edit is answered");
    String::from_utf8_lossy(&reply.bytes).into_owned()
}

#[test]
fn the_formula_bar_types_clears_and_refuses_in_words() {
    let store = Arc::new(InputStore::default());
    let kernel = kernel_over(Arc::clone(&store));
    assert_eq!(edit(&kernel, " b3 ", "=1<2"), "B3 is now =1&lt;2.");
    assert_eq!(value(&kernel, "B3"), "#SYNTAX");
    assert_eq!(edit(&kernel, "B3", ""), "B3 is clear.");
    assert_eq!(value(&kernel, "B3"), "");
    assert_eq!(
        edit(&kernel, "e9", "1"),
        "invalid argument `ref`: E9 is off the sheet, which is columns A–D and rows 1–6"
    );
    assert!(edit(&kernel, "3B", "1").starts_with("invalid argument `ref`: `3B` is not a cell"));
    // Reading the edit is refused: it is a write.
    assert!(source(&kernel, spreadsheet::VIEW_EDIT).is_err());
}

/// Why the cache viewer beside the page's grid is the page's own code and not a composition
/// over `urn:kernel:cache`: the readout lists what the cache HOLDS, and a cut is lazy. An
/// entry whose thread was cut stays resident until the next read of it finds it stale, so
/// right after an edit the readout still lists every value the edit invalidated, exactly
/// as it did before. The kernel's probe (`Kernel::is_cached`, the REPL's `cache`) is what
/// knows; no endpoint can ask it. When this test fails, the readout has learned to tell
/// the two apart, and the viewer can become a resource.
#[test]
fn the_cache_readout_still_lists_what_an_edit_has_cut() {
    let kernel = kernel();
    type_into(&kernel, "A1", "5");
    type_into(&kernel, "A2", "=A1*2");
    assert_eq!(value(&kernel, "A2"), "10");
    type_into(&kernel, "A1", "6");
    assert!(!value_cached(&kernel, "A2"), "the probe knows A2 was cut");
    let readout = source(&kernel, "urn:kernel:cache").unwrap();
    assert!(
        readout.contains("urn:iki:tutorial:sheet:cell:A2 "),
        "the readout still lists it:\n{readout}"
    );
}
