// axe-core in a REAL browser, with layout, over the assembled site, in every theme, with the
// in-page kernels loaded and the live parts of each page started.
//
//   node axe-browser.mjs ../target/site                 # gate: exit 1 on any violation
//   node axe-browser.mjs ../target/site --survey        # every page × theme, violation or not
//   node axe-browser.mjs ../target/site --chrome <path> # a Chrome/Chromium binary to drive
//
// Why a third instrument (ledger #616). `crates/book-a11y` measures the contrast of the
// theme's VARIABLE pairs, and `axe.mjs` runs axe in jsdom, which has no layout, so axe's
// color rules are switched off there. Neither sees a color that is not a theme variable: a
// highlight.js token, an `opacity`, a link told apart from its sentence only by hue. A
// real-browser axe run found three such failures, book-wide, that both gates had passed for
// weeks (tutorial PR #56). This is that run, made a gate: everything a reader can see, in
// each of mdbook's five themes, measured by the browser that paints it.
//
// What it loads. The ASSEMBLED site (`scripts/assemble-site.sh`), the directory the deploy
// uploads, so both books are checked as published and nothing that is not published is.
// Every page is fetched at the URL it is served at, on the made-up origin `book-dom.mjs`
// uses, and each request is answered from the directory by Playwright's router: no server,
// and no request ever leaves the process. Anything off-site is refused.
//
// What it starts. On a page with a live part (a Run cell, a board, a polling view, a sheet),
// the page's kernel MUST load: CI builds it before this runs, so a page that falls back to
// its no-kernel path here is a broken deploy, not something to audit around. Then every
// cell is run once (its result pane and history are what a reader sees after Run), every
// "Expected output" is opened, and every polling view is started, and axe waits for the
// cells to settle. The no-kernel path has its own instruments (`axe.mjs`, `no-kernel.mjs`).
//
// Which browser. `playwright-core` is pinned exactly and downloads nothing. The browser is
// the machine's installed Google Chrome (Playwright's `chrome` channel; GitHub's Ubuntu
// runners carry one), or the binary `--chrome` names. Pinning a browser build would mean
// fetching one from Playwright's CDN, a network dependency beyond the npm registry this
// workflow already uses; so the version is PRINTED instead, and a color failure that
// appears with a new Chrome says which one in the log.
//
// What gates. Violations of rules axe tags WCAG 2.0/2.1/2.2 A or AA, the same set
// `axe.mjs` gates on: here that includes `color-contrast` and `link-in-text-block`, which
// only a layout can judge. Best-practice rules are reported by `--survey` and do not gate.
// `print.html` and `toc.html` are skipped, as everywhere else.
import { readFileSync, existsSync, statSync } from "node:fs";
import { join, relative, extname } from "node:path";
import { createRequire } from "node:module";
import { chromium } from "playwright-core";
import { ORIGIN, pages } from "./book-dom.mjs";

const require = createRequire(import.meta.url);
const axeSource = readFileSync(require.resolve("axe-core/axe.min.js"), "utf8");

const args = process.argv.slice(2);
const siteArg = args.find((a, i) => !a.startsWith("--") && args[i - 1] !== "--chrome");
if (!siteArg) {
    console.error("usage: node axe-browser.mjs <assembled site directory> [--survey] [--chrome <path>]");
    process.exit(2);
}
const survey = args.includes("--survey");
const chromeAt = args.includes("--chrome") ? args[args.indexOf("--chrome") + 1] : null;

// mdbook 0.5's five themes, as `localStorage['mdbook-theme']` names them.
const THEMES = ["light", "rust", "coal", "navy", "ayu"];
const GATING = new Set(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"]);

// The site is served under the root book's `site-url`; assemble-site.sh puts the root book
// (the shortest site-url) at the directory's top, and its 404 page carries that url as
// `<base href>`.
function siteUrlOf(dir) {
    const m = readFileSync(join(dir, "404.html"), "utf8").match(/<base href="([^"]+)"/);
    return m ? (m[1].endsWith("/") ? m[1] : m[1] + "/") : "/";
}
const siteUrl = siteUrlOf(siteArg);

const TYPES = {
    ".html": "text/html; charset=utf-8",
    ".js": "text/javascript; charset=utf-8",
    ".mjs": "text/javascript; charset=utf-8",
    ".css": "text/css; charset=utf-8",
    ".json": "application/json",
    ".wasm": "application/wasm",
    ".svg": "image/svg+xml",
    ".png": "image/png",
    ".woff": "font/woff",
    ".woff2": "font/woff2",
    ".ttf": "font/ttf",
    ".txt": "text/plain; charset=utf-8",
};

async function serve(route) {
    const u = new URL(route.request().url());
    if (u.origin !== ORIGIN || !u.pathname.startsWith(siteUrl)) {
        return route.abort("blockedbyclient");
    }
    let rel = decodeURIComponent(u.pathname.slice(siteUrl.length));
    if (rel === "" || rel.endsWith("/")) {
        rel += "index.html";
    }
    const path = join(siteArg, rel);
    if (!existsSync(path) || statSync(path).isDirectory()) {
        return route.fulfill({ status: 404, body: "not in the built site" });
    }
    return route.fulfill({
        status: 200,
        contentType: TYPES[extname(path)] || "application/octet-stream",
        body: readFileSync(path),
    });
}

const LIVE = "div.ikigai-run, div.ttt-play, div.ikigai-view, div.sheet-play";

// Load the kernel and start everything live on the page. Returns a problem, or null.
async function start(page) {
    if ((await page.locator(LIVE).count()) === 0) {
        return null;
    }
    const loaded = await page.evaluate(() =>
        window.ikigaiBookKernel().then(() => null, (e) => String(e && e.message ? e.message : e)));
    if (loaded !== null) {
        return `the in-page kernel did not load (${loaded}); build it first (scripts/build-kernels.sh)`;
    }
    // One snapshot of each set, clicked in the page: a toggle stops matching its selector the
    // moment it is pressed, so a re-evaluating locator would wait on it forever.
    await page.evaluate(() => {
        for (const run of document.querySelectorAll("button.ikigai-run-button:enabled")) {
            run.click();
        }
        for (const toggle of document.querySelectorAll('button.ikigai-view-toggle[aria-pressed="false"]')) {
            toggle.click();
        }
        for (const d of document.querySelectorAll("details.ikigai-run-expected-wrap")) {
            d.open = true;
        }
    });
    // Settled: no cell still running, and a polling view has had time to answer at least once.
    await page.waitForFunction(() =>
        [...document.querySelectorAll(".ikigai-run-caption")].every((c) => c.textContent !== "running…"),
        null, { timeout: 30000 });
    await page.waitForTimeout(1200);
    return null;
}

const browser = await chromium.launch(chromeAt ? { executablePath: chromeAt } : { channel: "chrome" });
const which = chromeAt ? `${chromeAt}` : "Google Chrome (the installed one)";
console.log(`browser: ${which} ${browser.version()} · axe-core ${JSON.parse(readFileSync(require.resolve("axe-core/package.json"), "utf8")).version}`);

const all = pages(siteArg);

// One theme: every page, in a context of its own (its own storage, so its own theme). The
// themes run side by side, since most of a page's time is spent waiting on its kernel and
// its views; what each one says is kept and printed in theme order, so the log reads the
// same however they interleaved.
async function auditTheme(theme) {
    const out = [];
    const err = [];
    let failures = 0;
    const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
    await context.addInitScript((t) => {
        try { localStorage.setItem("mdbook-theme", t); } catch (e) { /* storage refused */ }
    }, theme);
    await context.route("**/*", serve);
    for (const path of all) {
        const rel = relative(siteArg, path).split("\\").join("/");
        const label = `${theme.padEnd(5)} ${rel}`;
        const page = await context.newPage();
        await page.goto(`${ORIGIN}${siteUrl}${rel}`, { waitUntil: "load" });
        const problem = await start(page);
        if (problem) {
            failures += 1;
            err.push(`${label}: ${problem}`);
            await page.close();
            continue;
        }
        const applied = await page.evaluate(() => document.documentElement.className);
        if (!applied.split(/\s+/).includes(theme)) {
            failures += 1;
            err.push(`${label}: the page did not take the ${theme} theme (html class "${applied}")`);
            await page.close();
            continue;
        }
        await page.evaluate(axeSource);
        const found = await page.evaluate(() => window.axe.run(document, { resultTypes: ["violations"] })
            .then((r) => r.violations.map((v) => ({
                id: v.id, help: v.help, tags: v.tags,
                nodes: v.nodes.map((n) => ({ target: n.target, summary: n.failureSummary })),
            }))));
        await page.close();
        const gating = found.filter((v) => v.tags.some((t) => GATING.has(t)));
        const advisory = found.filter((v) => !v.tags.some((t) => GATING.has(t)));
        if (survey) {
            out.push(`${label.padEnd(56)} ${gating.length ? "FAIL" : "ok"}${advisory.length ? `  (${advisory.length} advisory)` : ""}`);
            for (const v of advisory) {
                out.push(`    advisory: ${v.id} — ${v.help}`);
            }
        }
        for (const v of gating) {
            failures += 1;
            const sc = v.tags.filter((t) => /^wcag\d{3,4}$/.test(t)).map((t) => t.replace(/^wcag(\d)(\d)(\d+)$/, "SC $1.$2.$3"));
            err.push(`${label}: ${v.id} — ${v.help} [${sc.join(", ") || v.tags.join(", ")}]`);
            for (const node of v.nodes.slice(0, 3)) {
                err.push(`    ${node.target.join(" ")}: ${node.summary.split("\n").slice(1).join(" ").trim() || node.summary}`);
            }
        }
    }
    await context.close();
    return { out, err, failures };
}

let violations = 0;
for (const { out, err, failures } of await Promise.all(THEMES.map(auditTheme))) {
    out.forEach((line) => console.log(line));
    err.forEach((line) => console.error(line));
    violations += failures;
}
await browser.close();

console.log(`axe-core in a real browser: ${all.length * THEMES.length} page loads (${all.length} pages × ${THEMES.length} themes), ${violations} gating violation(s)`);
process.exit(violations ? 1 : 0);
