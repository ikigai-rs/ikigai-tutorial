//! The recorded window the ikigai Book's part "Exploring without Rust" runs on.
//!
//! | name | kind | what it is |
//! |---|---|---|
//! | `urn:file:quakes-2024-01-01.geojson` | atom | one week of USGS earthquakes, M4.5 and up, as GeoJSON |
//! | `urn:quakes:template:headline` | atom | a one-line template whose `$j{…}` markers ask the week two questions |
//!
//! The part works at a shell, where the CLI's file workspace (`ikigai --files <dir>`) serves a
//! copy of the same file under the same name, and in the page, where this crate serves it. So a
//! cell and a command name one resource. The page has no file system: the window is compiled in
//! with `include_bytes!`, it answers only `Source` and `Meta`, and it never changes, so its
//! answer is cacheable and hangs from the thread of its own name.
//!
//! Everything a chapter asks of it (a count, the largest, a grouping) is a composition of
//! published resources over this one atom: `urn:json:query` (ikigai-json) for jq, ikigai-fn's
//! `$j{…}` for a template. Our own terms, when a chapter mints any, live under `urn:quakes:`,
//! never under USGS's domain. USGS's own names (each event's page) stay theirs.

use ikigai_core::{
    ActionSpec, Description, EndpointSpace, Exact, FnEndpoint, Invocation, ReprType,
    Representation, Verb,
};

// ANCHOR: names
/// The recorded window's name: the same `urn:file:` name the CLI's file workspace gives the
/// copy a reader saves there.
pub const WINDOW: &str = "urn:file:quakes-2024-01-01.geojson";

/// GeoJSON's registered media type (RFC 7946). A `+json` type, so `urn:json:query` reads it as
/// JSON without a transreptor.
pub const GEOJSON: &str = "application/geo+json";
// ANCHOR_END: names

/// The window's bytes, exactly as USGS served them (see this crate's README for the source URL,
/// the recording date and the digest).
pub const WINDOW_BYTES: &[u8] = include_bytes!("../quakes-2024-01-01.geojson");

/// How many events the window holds: what `.features|length` answers.
pub const EVENTS: usize = 178;

/// The headline template "Reshaping JSON with jq" fills with `urn:iki:fn:compose`. A name of our
/// own, under `urn:quakes:`: at a shell, the reader writes the same text to a file of their own.
pub const HEADLINE: &str = "urn:quakes:template:headline";

/// The headline's text, as `templates/headline.txt` holds it.
pub const HEADLINE_TEXT: &str = include_str!("../templates/headline.txt");

// ANCHOR: space
/// The window, bound at [`WINDOW`]: an atom that answers its bytes, cacheably; and the headline
/// template, at [`HEADLINE`].
pub fn space() -> EndpointSpace {
    EndpointSpace::new()
        .bind(Exact::new(WINDOW), window())
        .bind(Exact::new(HEADLINE), headline())
}
// ANCHOR_END: space

/// The atom behind [`WINDOW`].
pub fn window() -> FnEndpoint {
    FnEndpoint::new("quakes-window", |_: &Invocation<'_>| {
        Ok(Representation::new(ReprType::new(GEOJSON), WINDOW_BYTES.to_vec()).cacheable())
    })
    .with_description(
        Description::new("quakes-window")
            .title("A recorded week of earthquakes")
            .summary(
                "USGS earthquakes of M4.5 and up from 2024-01-01 to 2024-01-08, 178 events, as \
                 GeoJSON, recorded once and never changed: the window \"Exploring without \
                 Rust\" runs on.",
            )
            .action(ActionSpec::new(Verb::Source).output(GEOJSON))
            .verb(Verb::Meta),
    )
}

/// The atom behind [`HEADLINE`]: the template as it is written, markers and all.
pub fn headline() -> FnEndpoint {
    FnEndpoint::new("quakes-headline", |_: &Invocation<'_>| {
        Ok(Representation::new(
            ReprType::new("text/plain"),
            HEADLINE_TEXT.as_bytes().to_vec(),
        )
        .cacheable())
    })
    .with_description(
        Description::new("quakes-headline")
            .title("The week's headline, as a template")
            .summary(
                "One line with two `$j{…}` markers over the recorded week: how many \
                 earthquakes, and the largest. Fill it with urn:iki:fn:compose.",
            )
            .action(ActionSpec::new(Verb::Source).output("text/plain"))
            .verb(Verb::Meta),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use ikigai_core::{Capability, Iri, Kernel, Request};
    use std::sync::Arc;

    fn read() -> Representation {
        let kernel = Kernel::new(Arc::new(space()));
        let request = Request::new(Verb::Source, Iri::parse(WINDOW).expect("an IRI"));
        block_on(kernel.issue(request, &Capability::root())).expect("the window resolves")
    }

    #[test]
    fn the_window_answers_its_committed_bytes_as_geojson() {
        let answer = read();
        assert_eq!(answer.repr_type.media_type, GEOJSON);
        assert_eq!(answer.bytes, WINDOW_BYTES);
    }

    #[test]
    fn the_window_is_the_week_the_readme_says() {
        let doc: serde_json::Value = serde_json::from_slice(WINDOW_BYTES).expect("JSON");
        assert_eq!(doc["features"].as_array().expect("features").len(), EVENTS);
        assert_eq!(doc["metadata"]["count"], EVENTS);
        assert_eq!(
            doc["metadata"]["url"],
            "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&starttime=2024-01-01&endtime=2024-01-08&minmagnitude=4.5"
        );
        let biggest = doc["features"]
            .as_array()
            .expect("features")
            .iter()
            .max_by(|a, b| {
                let mag = |f: &serde_json::Value| f["properties"]["mag"].as_f64().expect("a mag");
                mag(a).total_cmp(&mag(b))
            })
            .expect("one event at least");
        assert_eq!(biggest["id"], "us6000m0xl");
        assert_eq!(biggest["properties"]["mag"], 7.5);
    }
}
