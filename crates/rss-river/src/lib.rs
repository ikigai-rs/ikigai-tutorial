//! The fixture "A river of feeds" runs on, in the page.
//!
//! At a shell the chapter's REPL is `examples/rss/iki`, whose file workspace is
//! `examples/rss/ws/`, so `urn:file:directory.opml` is that folder's `directory.opml`. The page
//! has no file system, so this crate binds the same files at the same names, each an atom with
//! the committed bytes compiled in, typed as the CLI's file workspace types it (by extension:
//! `.opml` `text/x-opml`, `.xml` and `.xsl` `text/xml`, `.rq` `application/sparql-query`, `.nt`
//! `application/n-triples`). So a cell and a command name one resource, and read one byte string.
//!
//! | names | what they are |
//! |---|---|
//! | `urn:file:directory.opml` | the directory of four invented feeds |
//! | `urn:file:feeds/{host}.xml` | the four feeds, under `example.com` and `example.org` |
//! | `urn:file:feeds.xsl`, `urn:file:feeds-local.xsl`, `urn:file:items.xsl` | the stylesheets that read the directory and the feeds |
//! | `urn:file:queries/*.rq` | the river's questions |
//! | `urn:file:views/*.xsl` | the four faces of an answer |
//! | `urn:file:river.nt` | the river, as the example commits it |
//!
//! `urn:file:river.nt` is the one name whose file a reader's REPL *writes* (the chapter's section
//! 4, a map and a `sink`). A page cell cannot write a file, so here it is the committed copy,
//! which `examples/rss/check.sh` holds to what the build line writes. Every atom answers
//! `Source` and `Meta` only, and never changes, so its answer is cacheable and hangs from the
//! thread of its own name.
//!
//! Nothing here is under a real person's domain, and nothing reads the network.

use ikigai_core::{
    ActionSpec, Description, EndpointSpace, Exact, FnEndpoint, Invocation, ReprType,
    Representation, Verb,
};

/// What `.opml` is at a shell (mime_guess's table, which the CLI's file workspace reads).
pub const OPML: &str = "text/x-opml";

/// What `.xml` and `.xsl` are at a shell.
pub const XML: &str = "text/xml";

/// What `.rq` is at a shell: SPARQL's registered query type.
pub const SPARQL_QUERY: &str = "application/sparql-query";

/// What `.nt` is at a shell.
pub const N_TRIPLES: &str = "application/n-triples";

/// The river's name: the file section 4's build line writes.
pub const RIVER: &str = "urn:file:river.nt";

/// The directory's name.
pub const DIRECTORY: &str = "urn:file:directory.opml";

macro_rules! file {
    ($path:literal, $media:expr) => {
        (
            concat!("urn:file:", $path),
            $media,
            include_bytes!(concat!("../../../examples/rss/ws/", $path)) as &[u8],
        )
    };
}

/// Every file the page binds: its `urn:file:` name, its media type and its committed bytes.
pub const FILES: [(&str, &str, &[u8]); 22] = [
    file!("directory.opml", OPML),
    file!("feeds.xsl", XML),
    file!("feeds-local.xsl", XML),
    file!("items.xsl", XML),
    file!("feeds/alice.example.com.xml", XML),
    file!("feeds/bob.example.org.xml", XML),
    file!("feeds/carol.example.com.xml", XML),
    file!("feeds/dave.example.org.xml", XML),
    file!("queries/from.rq", SPARQL_QUERY),
    file!("queries/in.rq", SPARQL_QUERY),
    file!("queries/lines.rq", SPARQL_QUERY),
    file!("queries/per-feed.rq", SPARQL_QUERY),
    file!("queries/river.rq", SPARQL_QUERY),
    file!("queries/search.rq", SPARQL_QUERY),
    file!("queries/search-bound.rq", SPARQL_QUERY),
    file!("queries/search-feeds.rq", SPARQL_QUERY),
    file!("queries/since.rq", SPARQL_QUERY),
    file!("views/digest.xsl", XML),
    file!("views/feed.xsl", XML),
    file!("views/opml.xsl", XML),
    file!("views/page.xsl", XML),
    file!("river.nt", N_TRIPLES),
];

/// Every file in [`FILES`], each bound at its own name.
pub fn space() -> EndpointSpace {
    FILES
        .iter()
        .fold(EndpointSpace::new(), |space, &(name, media, bytes)| {
            space.bind(Exact::new(name), file(name, media, bytes))
        })
}

/// The atom behind one of [`FILES`]: its bytes, as `media`, cacheably.
pub fn file(name: &'static str, media: &'static str, bytes: &'static [u8]) -> FnEndpoint {
    let path = name.strip_prefix("urn:file:").expect("a urn:file: name");
    let id = format!("rss-{}", path.replace('/', "-"));
    FnEndpoint::new(id.clone(), move |_: &Invocation<'_>| {
        Ok(Representation::new(ReprType::new(media), bytes.to_vec()).cacheable())
    })
    .with_description(
        Description::new(id)
            .title(format!("examples/rss/ws/{path}"))
            .summary(format!(
                "The file examples/rss/ws/{path} of \"A river of feeds\", committed and never \
                 changed: the same bytes the CLI's file workspace serves at this name."
            ))
            .action(ActionSpec::new(Verb::Source).output(media))
            .verb(Verb::Meta),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use ikigai_core::{Capability, Iri, Kernel, Request};
    use std::sync::Arc;

    /// Every file under `examples/rss/ws/` a chapter cell can read is here, and nothing else:
    /// the scripts (`*.iki`) and the written faces (`out/`) are not read by a cell.
    #[test]
    fn the_space_binds_every_file_the_chapter_reads() {
        let ws = format!("{}/../../examples/rss/ws", env!("CARGO_MANIFEST_DIR"));
        let mut on_disk = Vec::new();
        for dir in ["", "feeds/", "queries/", "views/"] {
            for entry in std::fs::read_dir(format!("{ws}/{dir}")).expect("a folder") {
                let entry = entry.expect("an entry");
                let file_name = entry.file_name().into_string().expect("UTF-8");
                if entry.file_type().expect("a type").is_file() && !file_name.ends_with(".iki") {
                    on_disk.push(format!("urn:file:{dir}{file_name}"));
                }
            }
        }
        on_disk.sort();
        let mut bound: Vec<String> = FILES.iter().map(|(n, ..)| n.to_string()).collect();
        bound.sort();
        assert_eq!(bound, on_disk);
    }

    #[test]
    fn each_file_answers_its_committed_bytes_typed_as_a_shell_types_it() {
        let kernel = Kernel::new(Arc::new(space()));
        for (name, media, bytes) in FILES {
            let request = Request::new(Verb::Source, Iri::parse(name).expect("an IRI"));
            let answer =
                block_on(kernel.issue(request, &Capability::root())).expect("the file resolves");
            assert_eq!(answer.repr_type.media_type, media, "{name}");
            assert_eq!(answer.bytes, bytes, "{name}");
        }
    }
}
