//! Part VII: **the game's space is a file.**
//!
//! Parts I–VI arranged the game in Rust: [`space_with_store`](crate::space_with_store) binds
//! sixteen doors and a store, wraps them in the eight line aliases, and a host adds
//! `conditional`. This module builds the same game from `tic-tac-toe.arrangement`, an
//! s-expression beside this crate's source, and leaves only the ENDPOINTS in Rust:
//!
//! * [`registry`] registers every endpoint the game has, by the name each gives itself. A
//!   declaration binds a door to an endpoint by that name and never makes one.
//! * [`space_over`] reads the file ([`topology`]) and builds it with core's
//!   [`build`](ikigai_core::build) against that registry.
//! * [`game`] puts a declared game in a corridor of its own, so the book's page can play it
//!   beside the coded one on one kernel.
//!
//! Two resources show the kernel's side of the same act: `urn:iki:tutorial:ttt:arrangement:{name}`
//! answers a declaration's text, and `urn:iki:tutorial:ttt:build` reads one through the
//! kernel, as Turtle by ikigai-sexpr's transreptor, and builds it — answering the space it
//! built, or the refusal that says where the declaration went wrong.

use std::sync::Arc;

use ikigai_core::{
    ArgRef, ArgSpec, AsyncFnEndpoint, DeclarationError, Description, Endpoint, EndpointSpace,
    Error, Exact, Fallback, FnEndpoint, Invocation, InvokeFuture, Iri, Kernel, Registry, ReprType,
    Representation, Request, Result, RuleKind, Scope, Space, SpaceKind, Topology, UriTemplate,
    Verb,
};
use ikigai_sexpr::{arrangement_to_topology, topology_to_arrangement, MEDIA_ARRANGEMENT};
use ikigai_vocab::TurtleRenderer;

use crate::{
    board, checkset, line_of_cells, platonic_cell, play_move, reset, stored_cell, templates, turn,
    view, view_play, view_reset, winner, CellStore,
};

// ANCHOR: declared_names
/// A declaration, by name: the text of `{name}.arrangement`, as `text/x-ikigai-arrangement`.
pub const ARRANGEMENT: &str = "urn:iki:tutorial:ttt:arrangement:{name}";

/// Build the declaration `of` names, and answer the space that was built — `Source` only.
pub const BUILD: &str = "urn:iki:tutorial:ttt:build";

/// A game built from the file: the name of the corridor it is played in, as
/// [`GAME`](crate::GAME) names a coded game's.
pub const DECLARED: &str = "urn:iki:tutorial:ttt:declared:{id}";
// ANCHOR_END: declared_names

/// Every declaration this crate ships, by the name it is resolved at: the game, and one that
/// a build refuses.
pub const ARRANGEMENTS: &[(&str, &str)] = &[
    ("tic-tac-toe", include_str!("../tic-tac-toe.arrangement")),
    ("undo", include_str!("../undo.arrangement")),
];

/// The game's declaration, as written: `tic-tac-toe.arrangement`.
pub const TIC_TAC_TOE: &str = include_str!("../tic-tac-toe.arrangement");

/// The name of declaration `name`.
pub fn arrangement_name(name: &str) -> String {
    format!("urn:iki:tutorial:ttt:arrangement:{name}")
}

/// The corridor name of declared game `id`: letters, digits and `-`, as a coded game's id.
pub fn declared_name(id: &str) -> Result<Iri> {
    // The same rule as a coded game's id, so reuse its check and its words.
    crate::game_name(id)?;
    Iri::parse(format!("urn:iki:tutorial:ttt:declared:{id}"))
        .map_err(|e| Error::Endpoint(format!("a declared game's name: {e}")))
}

// ANCHOR: registry
/// Every endpoint the game has, registered under the name each one gives itself — the host's
/// half of a declaration. The file says which door binds which name; this says what each
/// name IS. Nothing else in the game is Rust any more.
///
/// The five views are one kind of endpoint, `composeOver`, over five templates, so each is
/// [`named`](ikigai_fn::ComposeOver::named) here: a registry holds one endpoint per name.
/// `lines` is the table the rules read (the CheckSet, the winner and the reset), taken from
/// the declaration by [`lines_of`] so the board's geometry is written once.
pub fn registry(
    store: Arc<CellStore>,
    lines: Arc<ikigai_core::AliasTable>,
) -> std::result::Result<Registry, DeclarationError> {
    let endpoints: [Arc<dyn Endpoint>; 18] = [
        Arc::new(stored_cell(store)),
        Arc::new(platonic_cell()),
        Arc::new(line_of_cells()),
        Arc::new(board()),
        Arc::new(checkset(Arc::clone(&lines))),
        Arc::new(winner(Arc::clone(&lines))),
        Arc::new(turn()),
        Arc::new(play_move()),
        Arc::new(reset(lines)),
        Arc::new(templates()),
        Arc::new(view("board").named("ttt-view-board")),
        Arc::new(view("square").named("ttt-view-square")),
        Arc::new(view("status").named("ttt-view-status")),
        Arc::new(view("reply").named("ttt-view-reply")),
        Arc::new(view("game").named("ttt-view-game")),
        Arc::new(view_play()),
        Arc::new(view_reset()),
        Arc::new(ikigai_fn::conditional()),
    ];
    let mut registry = Registry::new();
    for endpoint in endpoints {
        registry.register(endpoint)?;
    }
    Ok(registry)
}
// ANCHOR_END: registry

// ANCHOR: lines_of
/// The line table a declaration holds: the rules of its one alias, in the order the table
/// tries them. The rules need the lines as a table (the CheckSet reads which lines pass
/// through a cell, the winner reads every line), and the file is where the lines are.
///
/// A declaration with no alias declares no lines, so no square is on its board: the rules
/// then refuse every move as off the board, which is what such a file says. One with two is
/// refused, since which of them holds the board's lines would be a guess.
pub fn lines_of(tree: &Topology) -> std::result::Result<ikigai_core::AliasTable, DeclarationError> {
    fn aliases<'t>(node: &'t Topology, found: &mut Vec<&'t Topology>) {
        if matches!(node.kind, SpaceKind::Alias { .. }) {
            found.push(node);
        }
        for child in &node.children {
            aliases(child, found);
        }
    }
    let mut found = Vec::new();
    aliases(tree, &mut found);
    let alias = match found.as_slice() {
        [] => return Ok(ikigai_core::AliasTable::new()),
        [alias] => alias,
        more => {
            return Err(DeclarationError::Malformed {
                node: None,
                reason: format!(
                    "a game's declaration holds at most one alias, the board's lines; this \
                     one holds {}",
                    more.len()
                ),
            })
        }
    };
    let SpaceKind::Alias { rules, .. } = &alias.kind else {
        unreachable!("filtered on the kind above")
    };
    Ok(rules
        .iter()
        .fold(ikigai_core::AliasTable::new(), |table, rule| {
            match rule.kind {
                RuleKind::Exact => table.exact(rule.from.as_str(), rule.to.as_str()),
                RuleKind::Prefix => table.prefix(rule.from.as_str(), rule.to.as_str()),
            }
        }))
}
// ANCHOR_END: lines_of

// ANCHOR: build_game
/// The game `tree` declares, over `store`: its line table read from the declaration, every
/// endpoint registered, and core's [`build`](ikigai_core::build). Refused, naming where, if
/// the declaration binds a name the registry does not hold, or anything else core refuses.
pub fn build_game(
    tree: &Topology,
    store: Arc<CellStore>,
) -> std::result::Result<Arc<dyn Space>, DeclarationError> {
    let lines = Arc::new(lines_of(tree)?);
    ikigai_core::build(tree, &registry(store, lines)?)
}
// ANCHOR_END: build_game

/// The game's declaration, read: `tic-tac-toe.arrangement` through ikigai-sexpr's reader,
/// the one its transreptor wraps.
pub fn topology() -> std::result::Result<Topology, DeclarationError> {
    arrangement_to_topology(TIC_TAC_TOE).map_err(|e| DeclarationError::Malformed {
        node: None,
        reason: e.to_string(),
    })
}

/// The game built from its file, over `store`.
pub fn space_over(store: Arc<CellStore>) -> std::result::Result<Arc<dyn Space>, DeclarationError> {
    build_game(&topology()?, store)
}

// ANCHOR: declared_kernel
/// A host for the declared game alone: a kernel over the space the file declares, with the
/// Meta renderer [`kernel`](crate::kernel) uses. The coded kernel's twin, with no arrangement
/// in Rust.
pub fn kernel_over(store: Arc<CellStore>) -> std::result::Result<Kernel, DeclarationError> {
    Ok(Kernel::with_meta_renderer(
        space_over(store)?,
        Arc::new(TurtleRenderer),
    ))
}
// ANCHOR_END: declared_kernel

/// [`kernel_over`] with a store of its own.
pub fn kernel() -> std::result::Result<Kernel, DeclarationError> {
    kernel_over(Arc::default())
}

// ANCHOR: declared_game
/// Declared game `id`: a corridor, named [`DECLARED`], holding the whole game the file
/// declares over a store of its own. Put ahead of a root, every name the file binds is
/// answered in the corridor, and nothing reaches the root's game.
pub fn game(id: &str) -> Result<Scope> {
    let name = declared_name(id)?;
    let space = space_over(Arc::default()).map_err(|e| Error::Endpoint(e.to_string()))?;
    Ok(Scope::empty().with_named(name, space))
}
// ANCHOR_END: declared_game

/// `ttt-arrangement`: declaration `name`, as `text/x-ikigai-arrangement`, computed once.
pub fn arrangements() -> FnEndpoint {
    FnEndpoint::new("ttt-arrangement", |inv: &Invocation<'_>| {
        let name = inv
            .bindings
            .get("name")
            .ok_or_else(|| Error::MissingArgument("name".to_string()))?;
        let (_, text) = ARRANGEMENTS
            .iter()
            .find(|(known, _)| *known == name)
            .ok_or_else(|| Error::NotFound(format!("there is no arrangement named `{name}`")))?;
        Ok(Representation::new(arrangement_type(), text.as_bytes().to_vec()).cacheable())
    })
    .with_description(
        Description::new("ttt-arrangement")
            .title("Arrangement")
            .summary(
                "A declaration of a space, as an s-expression: which doors bind which endpoints.",
            )
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("name")
                    .summary("which declaration")
                    .class(XSD_STRING)
                    .one_of(ARRANGEMENTS.iter().map(|(name, _)| *name))
                    .binding(),
            )
            .output(MEDIA_ARRANGEMENT),
    )
}

// ANCHOR: build_endpoint
/// `ttt-build`: read the declaration `of` names through the kernel — as Turtle, by whatever
/// transreptor the kernel has for its type (ikigai-sexpr's, for an `.arrangement`) — build
/// it against the game's registry over a store of its own, and answer the space that was
/// built, printed as an arrangement.
///
/// The answer is the built space describing itself, not the file: comments gone, the alias
/// rules in the order the table tries them. A declaration the build refuses is refused here,
/// in core's words, as an invalid `of`. Cacheable: a pure function of what `of` answered,
/// so it hangs from `of`'s thread, and an edit to the file rebuilds it.
pub fn build_endpoint() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("ttt-build", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let raw = inv.inline_str("of")?.trim();
            let of = Iri::parse(raw).map_err(|e| Error::InvalidArgument {
                name: "of".to_string(),
                detail: format!("not a resource name ({e}): `of` names the declaration"),
            })?;
            let turtle = turtle_of(inv, &of).await?;
            let refused = |detail: String| Error::InvalidArgument {
                name: "of".to_string(),
                detail,
            };
            let tree = Topology::from_turtle(&turtle).map_err(|e| refused(e.to_string()))?;
            let built = build_game(&tree, Arc::default()).map_err(|e| refused(e.to_string()))?;
            let text = topology_to_arrangement(&built.topology())
                .map_err(|e| Error::Endpoint(format!("printing the built space: {e}")))?;
            Ok(Representation::new(arrangement_type(), text.into_bytes()).cacheable())
        })
    })
    .with_description(
        Description::new("ttt-build")
            .title("Build a declaration")
            .summary(
                "Read the declaration `of` names, build it against the game's endpoints, and \
                 answer the space that was built as an arrangement; refused, naming where, if \
                 it binds an endpoint the game does not have.",
            )
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("of")
                    .summary(
                        "the name of a declaration, e.g. \
                         urn:iki:tutorial:ttt:arrangement:tic-tac-toe; the one input, so a \
                         positional value lands here",
                    )
                    .class(XSD_ANY_URI),
            )
            .output(MEDIA_ARRANGEMENT),
    )
}

/// Source `of` and bring it to Turtle through the kernel's own transreptors: a plan from the
/// type it answered as, each step a request, so every step is a sub-request like any other.
async fn turtle_of(inv: &Invocation<'_>, of: &Iri) -> Result<String> {
    let answered = inv.source(of).await?;
    let media = answered.repr_type.media_type.clone();
    let mut bytes = answered.bytes;
    if media != TURTLE {
        let plan =
            inv.select_transreptor(&media, TURTLE)
                .ok_or_else(|| Error::InvalidArgument {
                    name: "of".to_string(),
                    detail: format!(
                        "{} answers {media}, and nothing here reads that as Turtle",
                        of.as_str()
                    ),
                })?;
        for step in plan {
            let transreptor = Iri::parse(&step.endpoint)
                .map_err(|e| Error::Endpoint(format!("a transreptor's name: {e}")))?;
            let request = Request::new(Verb::Source, transreptor)
                .with_arg("content", ArgRef::Inline(bytes))
                .with_arg("as", ArgRef::Inline(step.to.into_bytes()));
            bytes = inv.issue(request).await?.bytes;
        }
    }
    String::from_utf8(bytes).map_err(|_| Error::InvalidArgument {
        name: "of".to_string(),
        detail: format!("{} is not UTF-8 text", of.as_str()),
    })
}
// ANCHOR_END: build_endpoint

/// What a host binds to read and build declarations: the declarations, the build, and
/// ikigai-sexpr's arrangement transreptors (an arrangement's Turtle face, and back) — that
/// surface alone, so a page that mounts this pays for nothing else in the crate.
pub fn declaration_space() -> Fallback {
    Fallback::new(vec![
        Arc::new(
            EndpointSpace::new()
                .bind(template(ARRANGEMENT), arrangements())
                .bind(Exact::new(BUILD), build_endpoint()),
        ) as Arc<dyn Space>,
        Arc::new(ikigai_sexpr::arrangement_space()),
    ])
}

const TURTLE: &str = "text/turtle";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const XSD_ANY_URI: &str = "http://www.w3.org/2001/XMLSchema#anyURI";

fn arrangement_type() -> ReprType {
    ReprType::new(MEDIA_ARRANGEMENT).with_param("charset", "utf-8")
}

fn template(source: &str) -> UriTemplate {
    UriTemplate::parse(source).expect("a constant template parses")
}
