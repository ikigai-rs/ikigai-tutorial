//! The recorded window the ikigai Book's part "Exploring without Rust" runs on.
//!
//! | name | kind | what it is |
//! |---|---|---|
//! | `urn:file:quakes-2024-01-01.geojson` | atom | one week of USGS earthquakes, M4.5 and up, as GeoJSON |
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

// ANCHOR: space
/// The window, bound at [`WINDOW`]: an atom that answers its bytes, cacheably.
pub fn space() -> EndpointSpace {
    EndpointSpace::new().bind(Exact::new(WINDOW), window())
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
