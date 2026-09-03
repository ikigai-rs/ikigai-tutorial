//! # 4 · Why an endpoint describes itself
//!
//! In most systems, documentation is a courtesy to humans. Here it is load-bearing
//! machinery, and skipping it produces a system that *works* and is *unusable by
//! anything but a person who already knows*.
//!
//! ## The `Description` does four jobs
//!
//! 1. **The engine routes named arguments by it.** `in="hello"` finds the right slot
//!    because the endpoint declared a slot called `in`.
//! 2. **Selection matches on it.** Finding a transreptor, finding an endpoint for a task,
//!    and inferring what actions a set of things affords are all the same query against
//!    declared types.
//! 3. **The catalog is built from it.** `urn:kernel:catalog` and `urn:kernel:actions` are
//!    assembled out of these descriptions, not maintained separately.
//! 4. **The agent tool list *is* it.** Projected over MCP, an endpoint's description
//!    becomes a tool definition. Nobody writes that by hand.
//!
//! An endpoint with a thin description still runs. It is simply invisible to everything
//! above — like a library function with no signature.
//!
//! ## ArgSpecs, from day one
//!
//! ```
//! # use ikigai_core::ArgSpec;
//! let spec = ArgSpec::new("in")
//!     .summary("the text to camel-case")
//!     .class("http://www.w3.org/2001/XMLSchema#string");
//! # let _ = spec;
//! ```
//!
//! - `optional()` — an argument is **required by default**, and this marks the exception.
//!   The default is the strict one on purpose: forgetting to say "required" should not
//!   quietly widen what the endpoint accepts.
//! - `class(…)` — an **XSD datatype IRI** for a scalar, or an **`rdfs:Class`** for an
//!   entity. This is the one people leave off, and it is the one that matters most.
//! - `one_of(…)` for enums, `default(…)` where there is one.
//!
//! ## Why `class` is the interesting field
//!
//! Because it turns "what can I do with these things?" into a query.
//!
//! If endpoints declare the *entity types* they consume — this one takes three
//! `schema:Person`, a `schema:Place` and a `schema:Date` — then given a set of things,
//! the actions they afford are the endpoints whose required input types are a subset of
//! the types present. Set containment over the catalog. A SPARQL query.
//!
//! That is the same mechanism as finding a transreptor from one media type to another,
//! one level up. It is why the description is not paperwork.
//!
//! ## The invariant that keeps it safe
//!
//! Type-driven affordance sounds alarming: whoever asserts types influences what gets
//! offered. A hostile assertion could surface an action that should not be there.
//!
//! The defense is already in the model. **Type intersection only *offers* an action;
//! executing it still requires the actor's capability.** Affordance is type-driven,
//! authorization is capability-driven, and they are separate gates. Adversarial data can
//! make the menu wrong. It cannot make the kitchen cook.
//!
//! ## The one that bites
//!
//! ⚠ **Declared capabilities must be enforced capabilities.** If your endpoint checks
//! authority it never declared, the catalog over-offers — it advertises something that
//! will fail. If it declares authority it never checks, the catalog under-protects, which
//! is worse. Parameterized authority (network hosts, filesystem paths) declares the
//! wildcard form: `urn:cap:net:*` means "holds some grant under this prefix".
//!
//! Next: [`super::ch05_binding`].
