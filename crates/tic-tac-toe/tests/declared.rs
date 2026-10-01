//! The claims the seventh chapter makes about the game's space as a file, each one run.
//!
//! ```text
//! cargo test -p tic-tac-toe --test declared -- --nocapture
//! ```
//!
//! The declared game is pinned by BEHAVIOR, not by topology equality with the coded one: the
//! coded kernel's five views are unnamed (each is `composeOver`), and the declared ones are
//! named, so the two arrangements differ in exactly those five endpoint names and in nothing
//! else — which is the first thing checked here.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    build, ArgRef, Capability, DeclarationError, Iri, Kernel, Registry, Request, SpaceKind,
    Topology, Verb,
};
use ikigai_sexpr::{arrangement_to_topology, topology_to_arrangement};
use tic_tac_toe::declared::{
    self, arrangement_name, build_game, declaration_space, lines_of, registry, ARRANGEMENTS,
    TIC_TAC_TOE,
};
use tic_tac_toe::{kernel_over, lines, view, CellStore};

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

/// What `kernel` answers for one request: the bytes and whether they may be cached, or the
/// refusal in the kernel's words.
fn answer(kernel: &Kernel, verb: Verb, name: &str, content: Option<&str>) -> String {
    let mut request = Request::new(verb, iri(name));
    if let Some(content) = content {
        request = request.with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()));
    }
    match block_on(kernel.issue(request, &Capability::root())) {
        Ok(repr) => format!(
            "{} [{}]",
            String::from_utf8_lossy(&repr.bytes),
            if repr.expiry == ikigai_core::Expiry::Always {
                "uncacheable"
            } else {
                "cacheable"
            }
        ),
        Err(refused) => format!("error: {refused}"),
    }
}

const T: &str = "urn:iki:tutorial:ttt:";

/// Everything a player or a page reads, asked after every step of the script below.
const READS: [&str; 19] = [
    "board",
    "winner",
    "turn",
    "row:0",
    "row:1",
    "row:2",
    "column:1",
    "diagonal:0",
    "cell:0:0",
    "cell:1:1",
    "checkset:1:1",
    "stored:0:0",
    "view:board",
    "view:status",
    "view:square:0:0",
    "view:square:1:1",
    "view:game:root",
    "template:square",
    "cells:0.0,1.1",
];

// ANCHOR: script
/// A game played to the end, refusals included: a taken square, a square off the board, the
/// wrong mark, a play through the view (and a refused one), X winning down column 1, a move
/// after the win, and a new game.
const SCRIPT: [(Verb, &str, Option<&str>); 14] = [
    (Verb::Sink, "move:1:1", None),
    (Verb::Sink, "move:1:1", None),
    (Verb::Sink, "move:5:5", None),
    (Verb::Sink, "move:0:0", Some("X")),
    (Verb::Sink, "view:play:0:0", None),
    (Verb::Sink, "view:play:0:0", None),
    (Verb::Sink, "move:1:0", None),
    (Verb::Sink, "move:0:2", None),
    (Verb::Sink, "move:1:2", None),
    (Verb::Sink, "move:2:2", None),
    (Verb::Source, "move:2:2", None),
    (Verb::Sink, "view:reset", None),
    (Verb::Sink, "move:2:0", None),
    (Verb::Delete, "stored:2:0", None),
];
// ANCHOR_END: script

// ANCHOR: same_game
/// The game built from the file answers every step of a whole game exactly as the coded game
/// does — every write, every refusal, and every read after each — and reads its atom exactly
/// as often, so it caches and recomputes the same.
#[test]
fn the_declared_game_answers_exactly_as_the_coded_one() {
    let (coded_store, declared_store) = (
        Arc::new(CellStore::default()),
        Arc::new(CellStore::default()),
    );
    let coded = kernel_over(Arc::clone(&coded_store));
    let declared = declared::kernel_over(Arc::clone(&declared_store)).expect("the file builds");

    let mut steps = 0;
    let mut compare = |verb: Verb, name: &str, content: Option<&str>| {
        let name = format!("{T}{name}");
        let (a, b) = (
            answer(&coded, verb, &name, content),
            answer(&declared, verb, &name, content),
        );
        assert_eq!(b, a, "{verb:?} {name}");
        steps += 1;
    };
    for name in READS {
        compare(Verb::Source, name, None);
    }
    for (verb, name, content) in SCRIPT {
        compare(verb, name, content);
        for name in READS {
            compare(Verb::Source, name, None);
        }
    }
    assert_eq!(steps, READS.len() * (SCRIPT.len() + 1) + SCRIPT.len());
    assert_eq!(
        declared_store.reads.load(Ordering::SeqCst),
        coded_store.reads.load(Ordering::SeqCst),
        "the same reads of the atom: the same cache, the same golden threads"
    );
}
// ANCHOR_END: same_game

/// The refusals the chapter shows are the ones both games give, word for word.
#[test]
fn a_taken_square_is_refused_in_the_same_words() {
    let coded = kernel_over(Arc::default());
    let declared = declared::kernel().expect("the file builds");
    for kernel in [&coded, &declared] {
        answer(kernel, Verb::Sink, &format!("{T}move:1:1"), None);
        assert_eq!(
            answer(kernel, Verb::Sink, &format!("{T}move:1:1"), None),
            "error: invalid argument `x, y`: 1,1 is taken — X played there"
        );
    }
}

/// The file declares the coded game's arrangement, with the five views named: the same
/// tree once those five names are put back to the one every unnamed view answers.
#[test]
fn the_file_is_the_coded_arrangement_with_its_views_named() {
    // The root space's own arrangement, the tree a declaration states (not the chain a
    // request with no corridor carries, which holds it).
    let coded = kernel_over(Arc::default()).root_topology();
    let mut declared = declared::topology().expect("the file reads");
    let mut renamed = Vec::new();
    fn unname(node: &mut Topology, renamed: &mut Vec<String>) {
        if let SpaceKind::EndpointSpace { doors } = &mut node.kind {
            for door in doors {
                if door.endpoint.starts_with("ttt-view-")
                    && !["ttt-view-play", "ttt-view-reset"].contains(&door.endpoint.as_str())
                {
                    renamed.push(std::mem::replace(
                        &mut door.endpoint,
                        ikigai_fn::COMPOSE_OVER.to_string(),
                    ));
                }
            }
        }
        for child in &mut node.children {
            unname(child, renamed);
        }
    }
    unname(&mut declared, &mut renamed);
    assert_eq!(
        renamed,
        [
            "ttt-view-board",
            "ttt-view-square",
            "ttt-view-status",
            "ttt-view-reply",
            "ttt-view-game"
        ]
    );
    assert_eq!(declared, coded);
}

/// Building is a fixpoint: the built space describes itself as the file does, and the file
/// goes around through Turtle and comes back the same arrangement. Only its comments are lost.
#[test]
fn the_round_trip_is_a_fixpoint() {
    let tree = declared::topology().expect("the file reads");
    let built = declared::space_over(Arc::default()).expect("the file builds");
    assert_eq!(built.topology(), tree);
    let printed = topology_to_arrangement(&tree).expect("it prints");
    assert_eq!(
        arrangement_to_topology(&printed).expect("the print reads"),
        tree
    );
    assert_eq!(
        Topology::from_turtle(&tree.to_turtle()).expect("the Turtle reads"),
        tree
    );
    assert!(TIC_TAC_TOE.contains(';') && !printed.contains(';'));
}

/// Eight doors have no variable, and the game binds them as templates, so the file says
/// `:match template` on exactly those eight. Left out, each would read as an exact name —
/// the same answers, a different arrangement.
#[test]
fn eight_doors_say_match_template() {
    let marked: Vec<&str> = TIC_TAC_TOE
        .lines()
        .filter(|line| line.contains(":match template"))
        .filter_map(|line| line.split('"').nth(1))
        .collect();
    assert_eq!(
        marked,
        [
            "urn:iki:tutorial:ttt:board",
            "urn:iki:tutorial:ttt:winner",
            "urn:iki:tutorial:ttt:turn",
            "urn:iki:tutorial:ttt:reset",
            "urn:iki:tutorial:ttt:view:board",
            "urn:iki:tutorial:ttt:view:status",
            "urn:iki:tutorial:ttt:view:reply",
            "urn:iki:tutorial:ttt:view:reset",
        ]
    );
    let bare = TIC_TAC_TOE.replace(" :match template", "");
    assert_ne!(
        arrangement_to_topology(&bare).expect("still an arrangement"),
        declared::topology().expect("the file reads")
    );
}

/// The rules read the lines the file declares: the table the declared game's rules are given
/// is the table the coded game writes as `LINES`.
#[test]
fn the_rules_read_the_lines_from_the_file() {
    let from_file = lines_of(&declared::topology().expect("the file reads")).expect("one alias");
    let spelled = |table: &ikigai_core::AliasTable| -> Vec<(String, String, String)> {
        table
            .rules()
            .iter()
            .map(|rule| {
                let kind = rule.kind().keyword().to_string();
                (kind, rule.from().to_string(), rule.to().to_string())
            })
            .collect()
    };
    assert_eq!(spelled(&from_file), spelled(&lines()));
}

// ANCHOR: refusals
/// A declaration arranges registered endpoints and never mints one: a door naming an endpoint
/// nobody registered is refused at build, naming the door and the name.
#[test]
fn a_declaration_cannot_mint_an_endpoint() {
    let undo = ARRANGEMENTS
        .iter()
        .find(|(name, _)| *name == "undo")
        .map(|(_, text)| *text)
        .expect("the undo arrangement");
    let tree = arrangement_to_topology(undo).expect("it reads");
    let refused = build(
        &tree,
        &registry(Arc::default(), Arc::new(lines())).expect("registers"),
    )
    .err()
    .expect("refused");
    assert!(
        matches!(&refused, DeclarationError::UnknownEndpoint { id, .. } if id == "ttt-undo"),
        "{refused}"
    );
}

/// A registry holds one endpoint per name, so two views left unnamed — both `composeOver` —
/// cannot both be registered. That is why each view is named.
#[test]
fn two_unnamed_views_are_one_name_too_many() {
    let mut registry = Registry::new();
    registry
        .register(Arc::new(view("board")))
        .expect("the first");
    let refused = registry
        .register(Arc::new(view("square")))
        .expect_err("refused");
    assert_eq!(
        refused,
        DeclarationError::DuplicateEndpoint {
            id: "composeOver".to_string()
        }
    );
}
// ANCHOR_END: refusals

/// What core cannot rebuild is refused, never skipped: an opaque space (a peer, a hand-written
/// resolver) and a closure rewrite, each named by the IRI the declaration's Turtle gives it.
#[test]
fn an_opaque_space_and_a_closure_rewrite_are_refused_naming_where() {
    let game = declared::topology().expect("the file reads");
    for (kind, word) in [
        (SpaceKind::Opaque, "opaque"),
        (SpaceKind::Rewrite, "closure"),
    ] {
        let mut tree = game.clone();
        let mut node = Topology::new(kind);
        if matches!(node.kind, SpaceKind::Rewrite) {
            node.children.push(Topology::opaque(None));
        }
        tree.children.push(node);
        let refused = build_game(&tree, Arc::default()).err().expect("refused");
        let said = refused.to_string();
        assert!(
            said.contains("urn:ikigai:space:_:") && said.contains(word),
            "{said}"
        );
    }
}

/// Seals are checked at build: a declaration cannot seal a name the host sealed.
#[test]
fn a_declaration_cannot_seal_what_the_host_sealed() {
    let tree = arrangement_to_topology(
        r#"(level "urn:iki:tutorial:ttt:level:rules" :seals ("urn:iki:tutorial:ttt:")
             (endpoints (door "urn:iki:tutorial:ttt:board" ttt-board :match template)))"#,
    )
    .expect("it reads");
    let host = registry(Arc::default(), Arc::new(lines()))
        .expect("registers")
        .sealing(["urn:iki:tutorial:ttt:"]);
    let refused = build(&tree, &host).err().expect("refused");
    assert!(matches!(refused, DeclarationError::Sealing(_)), "{refused}");
}

// ANCHOR: build_resource
/// `urn:iki:tutorial:ttt:build` reads a declaration through the kernel, as Turtle by
/// ikigai-sexpr's transreptor, builds it, and answers the space it built: the file's own
/// arrangement, printed without its comments. The undo declaration is refused in core's words.
#[test]
fn the_build_resource_reads_through_the_kernel() {
    let kernel = Kernel::new(Arc::new(declaration_space()));
    let build_of = |name: &str| {
        let request = Request::new(Verb::Source, iri(declared::BUILD))
            .with_arg("of", ArgRef::Inline(arrangement_name(name).into_bytes()));
        block_on(kernel.issue(request, &Capability::root()))
    };
    let built = build_of("tic-tac-toe").expect("it builds");
    let tree = declared::topology().expect("the file reads");
    assert_eq!(
        String::from_utf8(built.bytes).expect("text"),
        topology_to_arrangement(&tree).expect("it prints")
    );
    assert!(kernel.is_cached(
        &Request::new(Verb::Source, iri(declared::BUILD)).with_arg(
            "of",
            ArgRef::Inline(arrangement_name("tic-tac-toe").into_bytes())
        ),
        &Capability::root()
    ));
    let refused = build_of("undo").expect_err("refused").to_string();
    assert!(refused.starts_with("invalid argument `of`: "), "{refused}");
    assert!(
        refused.contains("binds `ttt-undo`, which the host did not register"),
        "{refused}"
    );
}
// ANCHOR_END: build_resource

/// Every declaration the crate ships is a resource, answered as its file.
#[test]
fn every_arrangement_is_a_resource() {
    let kernel = Kernel::new(Arc::new(declaration_space()));
    for (name, text) in ARRANGEMENTS {
        let answer = block_on(kernel.issue(
            Request::new(Verb::Source, iri(&arrangement_name(name))),
            &Capability::root(),
        ))
        .expect("answered");
        assert_eq!(answer.bytes, text.as_bytes(), "{name}");
        assert_eq!(answer.repr_type.media_type, ikigai_sexpr::MEDIA_ARRANGEMENT);
    }
}

/// A declared game is a corridor of its own: ahead of the coded game's root, every name the
/// game answers is answered in it, and its board is its own.
#[test]
fn a_declared_game_is_a_corridor_ahead_of_the_root() {
    let root = kernel_over(Arc::default());
    let scope = declared::game("a").expect("a corridor");
    let play = Request::new(Verb::Sink, iri(&format!("{T}move:1:1")));
    block_on(root.issue_in(play, &Capability::root(), scope.clone())).expect("plays");
    let board = |scope: ikigai_core::Scope| {
        let request = Request::new(Verb::Source, iri(&format!("{T}board")));
        let answer = block_on(root.issue_in(request, &Capability::root(), scope)).expect("reads");
        String::from_utf8(answer.bytes).expect("text")
    };
    assert_eq!(board(scope), "---\n-X-\n---");
    assert_eq!(board(ikigai_core::Scope::empty()), "---\n---\n---");
    assert!(declared::game("not an id").is_err());
}

// ANCHOR: picture
/// The picture the chapter shows is `ikigai_diagram::render` of the file's arrangement, byte
/// for byte. On a mismatch the picture the code draws is written beside the build and the
/// failure names the command that commits it; nothing rewrites it from inside a test.
#[test]
fn the_committed_picture_is_the_files_arrangement() {
    let committed = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../books/ikigai/src/applied/tic-tac-toe-arrangement.svg"
    );
    let drawn = ikigai_diagram::render(&declared::topology().expect("the file reads"));
    if std::fs::read_to_string(committed).ok().as_deref() == Some(drawn.as_str()) {
        return;
    }
    let out = format!(
        "{}/tic-tac-toe-arrangement.svg",
        env!("CARGO_TARGET_TMPDIR")
    );
    std::fs::write(&out, &drawn).expect("write the picture the code draws");
    panic!(
        "the committed picture no longer matches tic-tac-toe.arrangement.\n\
         If the change is intended:  cp {out} {committed}"
    );
}
// ANCHOR_END: picture
