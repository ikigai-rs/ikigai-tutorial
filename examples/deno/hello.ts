/**
 * Hello, resource — in TypeScript.
 *
 * The smallest ikigai peer there is: one function, one `endpoint(...)`, one socket. Run
 * it and a Rust host can mount it; `--mount urn:ts:=<socket>` in the chapter is the
 * binding.
 *
 *     deno run -A --config examples/deno/deno.json examples/deno/hello.ts /tmp/ts-hello.sock
 *
 * Modeled on ikigai-deno's `examples/demo.ts` (read-only for this book); kept here so
 * the book builds from a clone of this repository alone. Imports ikigai-deno by path
 * until it is on JSR, from a sibling checkout at `../ikigai-deno`; edit the import if
 * yours is elsewhere. ikigai-deno's own code imports `@std/crypto` by its bare name
 * (BLAKE3, for its contract IRIs), which its own `deno.json` maps and a script importing
 * it by path never sees, so `examples/deno/deno.json` beside this file maps it here.
 */

import { endpoint, Server } from "../../../ikigai-deno/src/serve.ts";

const XSD_STRING = "http://www.w3.org/2001/XMLSchema#string";

// ANCHOR: endpoint
// The description is explicit here — `args:` is the ArgSpec list, stated the way the
// Rust chapter stated it — and the handler receives the named arguments. (ikigai-deno's
// `./zod` entry point derives the same list from a schema; this file stays
// zero-dependency on purpose.) `cacheable: true` is the same claim `.cacheable()` makes
// in Rust; whether a kernel that mounts this peer caches the answers is the kernel's
// call, and the chapter shows what ikigai-cli does with it.
export const hello = endpoint("urn:ts:hello", {
  summary: "Greet someone",
  args: [{
    name: "who",
    required: true,
    summary: "the name to greet",
    class: XSD_STRING,
  }],
  cacheable: true,
}, ({ who }) => `Hello, ${who}!`);

// A list in, a list out: one item per line, each shouted, so a pipe that feeds it a list
// and reverses what comes back shows every stage's mark on every line.
export const shout = endpoint("urn:ts:shout", {
  summary: "Shout each line of a list",
  args: [{
    name: "text",
    required: true,
    summary: "the lines to shout, one per line",
    class: XSD_STRING,
  }],
  cacheable: true,
}, ({ text }) =>
  String(text).split("\n").map((line) => `${line.toUpperCase()}!`).join("\n"));
// ANCHOR_END: endpoint

// ANCHOR: serve
if (import.meta.main) {
  const path = Deno.args[0] ?? "/tmp/ts-hello.sock";
  console.error(`serving urn:ts:hello and urn:ts:shout on ${path}`);
  const server = new Server([hello, shout], path);
  Deno.addSignalListener("SIGINT", () => server.shutdown());
  await server.serve(); // blocks; speaks the wire protocol
}
// ANCHOR_END: serve
