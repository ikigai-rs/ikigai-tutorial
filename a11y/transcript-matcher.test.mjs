// Exercise the functions used by runnable cells, without loading a kernel or a DOM.
import { readFileSync } from "node:fs";
import vm from "node:vm";
import assert from "node:assert/strict";
import { test } from "node:test";

const source = readFileSync(new URL("../books/ikigai/js/run.js", import.meta.url), "utf8");
const start = source.indexOf("    function sameLines(");
const end = source.indexOf("    // One status line", start);
assert.ok(start >= 0 && end > start, "the browser's comparison functions exist");
const compare = vm.runInNewContext(source.slice(start, end) + "\nasTheChapterShows;");
const cases = readFileSync(new URL("../crates/book-transcripts/tests/fixtures/matches.tsv", import.meta.url), "utf8");
const decode = text => text.replaceAll("\\n", "\n").replaceAll("\\t", "\t");
for (const line of cases.trimEnd().split("\n")) {
    if (line.startsWith("#")) continue;
    const [name, verdict, expected, actual] = line.split("\t");
    test(name, () => assert.equal(compare(decode(actual), decode(expected)), verdict === "1"));
}
