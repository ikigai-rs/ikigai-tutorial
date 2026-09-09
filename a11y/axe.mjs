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
// the static-fallback path — "the in-page kernel did not load" — which is the path a
// reader without wasm gets, and worth checking on its own account. Rules axe tags as
// "best-practice" are reported in the survey and do not gate; violations of WCAG 2.x A/AA
// rules do. `print.html` (every chapter concatenated) and `toc.html` (the sidebar as a
// fragment mdbook loads into pages, not a document) are skipped, as book-a11y skips them.
import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { createRequire } from "node:module";
import { JSDOM, ResourceLoader, VirtualConsole } from "jsdom";

const require = createRequire(import.meta.url);
const axeSource = readFileSync(require.resolve("axe-core/axe.min.js"), "utf8");

const [, , bookArg, ...flags] = process.argv;
if (!bookArg) {
    console.error("usage: node axe.mjs <built book directory> [--survey]");
    process.exit(2);
}
const survey = flags.includes("--survey");

// The book is a PROJECT Pages site: `book.toml` sets `site-url = "/ikigai-tutorial/"`,
// and mdbook writes that as `<base href>` into 404.html (the page Pages serves for any
// missing path, at any depth). Loaded from a `file://` URL, that base resolves to nowhere,
// the page's scripts never run, and axe audits a 404 the reader never sees — the sidebar
// toggle's `aria-expanded` showed up exactly that way. So every page is loaded at the
// URL it is served at, on a made-up origin, and this loader maps that origin back onto
// the built directory. Same scripts, same base, same result as the live site.
const ORIGIN = "http://book.test";
const siteUrl = (() => {
    const toml = resolve(bookArg, "..", "book.toml");
    if (existsSync(toml)) {
        const m = readFileSync(toml, "utf8").match(/^site-url\s*=\s*"([^"]+)"/m);
        if (m) {
            return m[1].endsWith("/") ? m[1] : m[1] + "/";
        }
    }
    return "/";
})();

class BookLoader extends ResourceLoader {
    fetch(url, options) {
        const u = new URL(url);
        if (u.origin !== ORIGIN || !u.pathname.startsWith(siteUrl)) {
            return null; // anything off-book (a CDN font, say) is not fetched
        }
        const path = join(bookArg, decodeURIComponent(u.pathname.slice(siteUrl.length)));
        if (!existsSync(path) || statSync(path).isDirectory()) {
            return Promise.reject(new Error(`not in the built book: ${u.pathname}`));
        }
        return Promise.resolve(readFileSync(path));
    }
}

function pages(dir, out = []) {
    for (const name of readdirSync(dir)) {
        const path = join(dir, name);
        if (statSync(path).isDirectory()) {
            pages(path, out);
        } else if (name.endsWith(".html") && name !== "print.html" && name !== "toc.html") {
            out.push(path);
        }
    }
    return out.sort();
}

// The tags axe attaches to the rules this gate enforces: WCAG 2.0/2.1/2.2, levels A and
// AA. Anything else (best-practice, experimental, AAA) is survey-only.
const GATING = new Set(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"]);

async function audit(path) {
    const html = readFileSync(path, "utf8");
    // jsdom logs every failed resource fetch; the wasm's dynamic import is one on
    // purpose, so keep the console quiet and inspect results instead.
    const virtualConsole = new VirtualConsole();
    const dom = new JSDOM(html, {
        url: ORIGIN + siteUrl + relative(bookArg, path).split("\\").join("/"),
        runScripts: "dangerously",
        resources: new BookLoader(),
        pretendToBeVisual: true,
        virtualConsole,
    });
    const { window } = dom;
    // Let the page's scripts (a11y.js, run.js, mdbook's book.js) run their load handlers.
    await new Promise((resolve) => {
        if (window.document.readyState === "complete") {
            resolve();
        } else {
            window.addEventListener("load", resolve);
            setTimeout(resolve, 3000);
        }
    });
    // Press every Run button once, so the fallback path renders its status message and
    // axe sees the cell in the state a reader without wasm would.
    for (const button of window.document.querySelectorAll(".ikigai-run-button")) {
        button.click();
    }
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
