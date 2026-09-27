// axe-core over every built page of the book, in jsdom, with the book's own scripts run.
//
//   node axe.mjs ../books/ikigai/book          # gate: exit 1 on any violation
//   node axe.mjs ../books/ikigai/book --survey # every page, violation or not
//
// Why a second instrument. `crates/book-a11y` measures contrast and checks the static
// structure of each page — and a static read cannot see anything JavaScript adds after
// load: mdbook's copy-to-clipboard buttons, the Run cells `js/run.js` builds (a button, a
// status region), the two repairs `js/a11y.js` makes to the search box. Loading each page
// in jsdom and running its scripts before axe looks is what covers that.
//
// What this deliberately does not do. jsdom has no layout engine, so axe's color-contrast
// rule cannot run here (`book-a11y` is the contrast gate) and target-size is not judged.
// The dynamic `import()` of the wasm kernel fails in jsdom, so what axe sees in a cell is
// the no-kernel path — Run disabled, "the in-page kernel did not load" — which is the path
// a reader without wasm gets, and worth checking on its own account. (That the path answers
// nothing is `no-kernel.mjs`'s job, not this file's.) Rules axe tags as
// "best-practice" are reported in the survey and do not gate; violations of WCAG 2.x A/AA
// rules do. `print.html` (every chapter concatenated) and `toc.html` (the sidebar as a
// fragment mdbook loads into pages, not a document) are skipped, as book-a11y skips them.
import { readFileSync } from "node:fs";
import { relative } from "node:path";
import { createRequire } from "node:module";
import { openPage, pages } from "./book-dom.mjs";

const require = createRequire(import.meta.url);
const axeSource = readFileSync(require.resolve("axe-core/axe.min.js"), "utf8");

const [, , bookArg, ...flags] = process.argv;
if (!bookArg) {
    console.error("usage: node axe.mjs <built book directory> [--survey]");
    process.exit(2);
}
const survey = flags.includes("--survey");

// The tags axe attaches to the rules this gate enforces: WCAG 2.0/2.1/2.2, levels A and
// AA. Anything else (best-practice, experimental, AAA) is survey-only.
const GATING = new Set(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"]);

async function audit(path) {
    // Loaded at the URL it is served at; see book-dom.mjs for why.
    const window = await openPage(bookArg, path);
    // Give the failed kernel load time to settle, so axe sees each cell as a reader
    // without wasm would: Run disabled and described, the caption saying why.
    await new Promise((resolve) => setTimeout(resolve, 300));

    window.eval(axeSource);
    const results = await window.axe.run(window.document, {
        // No layout in jsdom: these rules would report nothing true.
        rules: { "color-contrast": { enabled: false }, "target-size": { enabled: false } },
        resultTypes: ["violations"],
    });
    window.close();
    return results.violations;
}

let violations = 0;
let checked = 0;
for (const path of pages(bookArg)) {
    const label = relative(bookArg, path);
    const found = await audit(path);
    checked += 1;
    const gating = found.filter((v) => v.tags.some((t) => GATING.has(t)));
    const advisory = found.filter((v) => !v.tags.some((t) => GATING.has(t)));
    if (survey) {
        console.log(`${label.padEnd(48)} ${gating.length ? "FAIL" : "ok"}${advisory.length ? `  (${advisory.length} advisory)` : ""}`);
    }
    for (const v of gating) {
        violations += 1;
        const sc = v.tags.filter((t) => /^wcag\d{3,4}$/.test(t)).map((t) => t.replace(/^wcag(\d)(\d)(\d+)$/, "SC $1.$2.$3"));
        console.error(`${label}: ${v.id} — ${v.help} [${sc.join(", ") || v.tags.join(", ")}]`);
        for (const node of v.nodes.slice(0, 3)) {
            console.error(`    ${node.target.join(" ")}: ${node.failureSummary.split("\n")[1] || node.failureSummary}`);
        }
    }
    if (survey) {
        for (const v of advisory) {
            console.log(`    advisory: ${v.id} — ${v.help}`);
        }
    }
}
if (violations) {
    console.error(`\naxe: ${violations} violation${violations === 1 ? "" : "s"} across ${checked} pages`);
    process.exit(1);
}
console.log(`axe: ${checked} pages, no WCAG 2.x A/AA violations`);
