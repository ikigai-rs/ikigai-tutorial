/**
 * A family of resources, in TypeScript: one template, four verbs.
 *
 * `hello.ts` served two exact names. This serves a FAMILY — every IRI one template
 * matches — and answers more than one verb on it: read a note, write it, delete it, ask
 * whether it is there. Run it and mount it the way the chapter mounts `hello.ts`:
 *
 *     deno run -A examples/deno/notes.ts /tmp/ts-notes.sock
 *     ikigai --plain --mount urn:ts:=/tmp/ts-notes.sock -c 'sink urn:ts:note:milk "2 liters"'
 *
 * Modeled on ikigai-deno's `examples/tictactoe_store.ts` (read-only for this book); kept
 * here so the book builds from a clone of this repository alone. Imports ikigai-deno by
 * path until it is on JSR.
 */

import { endpoint, family, Server } from "../../../ikigai-deno/src/serve.ts";
import { NotFoundError } from "../../../ikigai-deno/src/wire.ts";

const XSD_STRING = "http://www.w3.org/2001/XMLSchema#string";
const notes = new Map<string, string>();

// ANCHOR: index
// An exact name, declared FIRST: the family below would match `urn:ts:note:index` too
// (its `{name}` would capture `index`), and the first declared door that matches wins.
// Not cacheable: it reads every note, and a write cuts only the note it wrote.
export const index = endpoint("urn:ts:note:index", {
  summary: "The notes there are, one name per line",
}, () => [...notes.keys()].sort().join("\n"));
// ANCHOR_END: index

// ANCHOR: family
// One door for every `urn:ts:note:{name}`: `{name}` is a Level 1 variable, matched the
// way ikigai_core::UriTemplate matches it, and it reaches each handler by name.
export const note = family("urn:ts:note:{name}", {
  id: "ts-note",
  summary: "A note, by name",
  bindings: { name: { summary: "the note's name", class: XSD_STRING } },
})
  .source({
    summary: "the note's text; NotFound if there is none",
    cacheable: true,
  }, ({ name }) => {
    const text = notes.get(String(name));
    if (text === undefined) throw new NotFoundError(`no note called ${name}`);
    return text;
  })
  .sink({ summary: "write the note" }, ({ name, content }) => {
    notes.set(String(name), String(content));
    return `wrote ${name}`;
  })
  .delete({ summary: "forget the note" }, ({ name }) => {
    notes.delete(String(name));
    return `forgot ${name}`;
  });
// No `.exists(...)`: Exists defaults to "would Source succeed", so a NotFoundError is
// `false` — cacheable exactly when Source is.
// ANCHOR_END: family

if (import.meta.main) {
  const path = Deno.args[0] ?? "/tmp/ts-notes.sock";
  console.error(`serving urn:ts:note:index and urn:ts:note:{name} on ${path}`);
  const server = new Server([index, note], path); // the order is the lookup order
  Deno.addSignalListener("SIGINT", () => server.shutdown());
  await server.serve();
}
