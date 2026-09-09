/**
 * Part I, from TypeScript: resolve, describe, cache, trace, and be refused.
 *
 * Against a running kernel — `ikigai serve <socket>` — this walks the same four things
 * "What resolution buys you" ran in Rust, over the wire, from a runtime with no kernel
 * in it at all.
 *
 *     ikigai serve /tmp/ikbook-kernel.sock &
 *     deno run -A examples/deno/client.ts /tmp/ikbook-kernel.sock
 *
 * Imports ikigai-deno by path until it is on JSR.
 */

import {
  CacheStatus,
  Capability,
  connect,
  DeniedError,
} from "../../../ikigai-deno/src/mod.ts";

const path = Deno.args[0] ?? "/tmp/ikbook-kernel.sock";

// ANCHOR: resolve
// A name, resolved. `connect` speaks the wire protocol over the kernel's socket; the
// answer is a representation — bytes with a type — exactly what a Rust caller gets.
const k = await connect(path);
const rep = await k.source("urn:iki:fn:toUpper", { in: "resource oriented computing" });
console.log(rep.text); // RESOURCE ORIENTED COMPUTING
console.log(rep.mediaType); // text/plain;charset=utf-8
console.log(CacheStatus[rep.cacheStatus]); // how the kernel's cache answered: Miss, Hit, Uncacheable
// ANCHOR_END: resolve

// ANCHOR: describe
// Self-description, as data: the JSON face of Meta, parsed — the ArgSpecs an endpoint
// declared, which is what an agent's tool definition is made of.
const card = await k.describe("urn:iki:fn:toUpper");
console.log(card?.id, (card?.inputs as { name: string }[]).map((a) => a.name)); // toUpper [ "in" ]
// ...and the same card as a graph.
const turtle = await k.meta("urn:iki:fn:toUpper", "text/turtle");
console.log(turtle.text.split("\n")[0]); // @prefix ik: <https://ikigai-rs.dev/ns#> .
// ANCHOR_END: describe

// ANCHOR: cached
// Cached once: the probe says not yet; a resolution; the probe says served now.
console.log(await k.isCached("urn:iki:fn:toUpper", { in: "a b" })); // false
await k.source("urn:iki:fn:toUpper", { in: "a b" });
console.log(await k.isCached("urn:iki:fn:toUpper", { in: "a b" })); // true
// ANCHOR_END: cached

// ANCHOR: traced
// Traced: the kernel records its own events and ships them back over the wire.
const [, events] = await k.sourceTraced("urn:iki:fn:toUpper", { in: "a b" });
for (const event of events) {
  console.log(event.target, event.cacheHit ? "cache_hit" : "computed");
}
// ANCHOR_END: traced
k.close();

// ANCHOR: scoped
// Capabilities: connect under a narrowed authority and the kernel enforces it — a write
// to the file workspace under an inspect-only scope is refused, and the refusal arrives
// typed, as `DeniedError`, not as a string to parse.
const narrow = await connect(path, {
  capability: Capability.scoped(["urn:cap:kernel:inspect"]),
});
try {
  await narrow.sink("urn:file:notes.txt", "nope");
} catch (err) {
  if (err instanceof DeniedError) console.log("denied:", err.message);
  else throw err;
}
narrow.close();
// ANCHOR_END: scoped
