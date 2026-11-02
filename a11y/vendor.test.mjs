// The vendored DOMPurify, held to its pins (ledger #2823's review): the bytes in every book that
// serves it are upstream's `dist/purify.min.js` for the release `run.js` names, by the SHA-256
// written here and in `run.js`, and by the `integrity=` value `run.js` loads it with (so the
// browser's own check and this one agree). Its license is kept beside it.
import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("..", import.meta.url));
const RELEASE = "3.4.17";
// npm's `dompurify@3.4.17` tarball (integrity sha512-B/pSlYVHQ58k…), its `dist/purify.min.js`.
const SHA256 = "cd37898f9782adf54820c413191b5691803f0f137c799d6edcfd38523b4c2c31";
const FILE = `vendor/purify-${RELEASE}.min.js`;
const runJs = readFileSync(`${repo}books/ikigai/js/run.js`, "utf8");

for (const book of ["ikigai", "ui"]) {
    test(`books/${book}/src/${FILE} is DOMPurify ${RELEASE}, as pinned`, () => {
        const bytes = readFileSync(`${repo}books/${book}/src/${FILE}`);
        assert.equal(createHash("sha256").update(bytes).digest("hex"), SHA256);
        assert.match(bytes.toString("utf8", 0, 120), new RegExp(`^/\\*! @license DOMPurify ${RELEASE.replace(/\./g, "\\.")} `));
        const license = readFileSync(`${repo}books/${book}/src/vendor/purify-${RELEASE}.LICENSE.txt`, "utf8");
        assert.match(license, /Apache License\s+Version 2\.0/);
    });
}

test("run.js loads that file, by that digest", () => {
    const bytes = readFileSync(`${repo}books/ikigai/src/${FILE}`);
    const sri = "sha384-" + createHash("sha384").update(bytes).digest("base64");
    assert.match(runJs, new RegExp(`var PURIFY = "${FILE.replace(/\./g, "\\.")}";`));
    assert.ok(runJs.includes(`var PURIFY_INTEGRITY = "${sri}";`), `run.js should carry integrity ${sri}`);
    assert.ok(runJs.includes(SHA256), "run.js should name the sha256 it was vendored at");
});
