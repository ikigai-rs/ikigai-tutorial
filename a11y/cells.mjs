// Every Run cell, run the way a reader who reads in order runs it, against the chapter's
// "Expected output", in a real browser with the in-page kernels loaded.
//
//   node cells.mjs ../target/site                 # on its own: exit 1 on any cell that differs
//   node cells.mjs ../target/site --chrome <path> # a Chrome/Chromium binary to drive
//
// In CI it runs inside `axe-browser.mjs` (the "axe-core in a real browser" step of `pages.yml`),
// in the same browser, after the axe audit, and its failures fail that step: `checkCells` below.
//
// Why (ledger #2109). `js/run.js` marks every run "as the chapter shows" or "differs from the
// chapter", so that a refusal the chapter means to show reads as expected and a real failure
// stands out. That mark is only worth reading if a reader who runs a chapter's cells in order
// gets "as the chapter shows" on every one; otherwise the page cries wolf. So this loads every
// page of the ASSEMBLED site that has a cell, runs its cells one at a time in page order on a
// fresh load, and requires that mark (or, on a page that runs on the browser's clock, the mark
// saying it was not compared) of each. A chapter whose expected output was written for a cell run
// on its own, or a kernel that moved under a chapter, fails here, with the difference printed.
// A cell in a stages viewer's chain must also leave its tab open with the picture it drew as After,
// below the tabs, its caption saying the reader's kernel drew it (Brian, 2026-10-25: a step whose
// picture is not beside it is a step whose picture goes unseen; 2026-10-26: Before | After).
// Once a page's cells have run, each viewer's "Compare with" is driven: picking an earlier stage
// must change Before (`compareWith`, ledger #2298).
//
// Then the two things a reader who does NOT read in order meets (Brian, 2026-10-22: step 3 of
// "A space that changes while it runs" without step 2). For every cell that offers "Run all above",
// on a fresh load: Run alone must not claim "as the chapter shows" without saying that cells above
// have not run, when it differs; and "Run all above" followed by Run must give "as the chapter
// shows".
//
// The site is served the way `axe-browser.mjs` serves it: every request answered from the
// directory by Playwright's router, on the made-up origin `book-dom.mjs` uses; nothing leaves the
// process. Run on its own, the browser is the machine's Google Chrome (or `--chrome`), as there.
import { readFileSync, existsSync, statSync } from "node:fs";
import { join, relative, extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";
import { ORIGIN, pages } from "./book-dom.mjs";

function siteUrlOf(dir) {
    const m = readFileSync(join(dir, "404.html"), "utf8").match(/<base href="([^"]+)"/);
    return m ? (m[1].endsWith("/") ? m[1] : m[1] + "/") : "/";
}

const TYPES = {
    ".html": "text/html; charset=utf-8",
    ".js": "text/javascript; charset=utf-8",
    ".css": "text/css; charset=utf-8",
    ".json": "application/json",
    ".wasm": "application/wasm",
    ".svg": "image/svg+xml",
};

function server(siteArg, siteUrl) {
    return async function serve(route) {
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
    };
}

const AS_SHOWN = "as the chapter shows";
const NOT_COMPARED = "not compared: this page runs on your clock";

// In the page: press one of cell `i`'s buttons and wait for its caption to settle. Returns the
// cell's caption, the mark of its newest run, its output and its expected output.
async function press(page, i, which) {
    return page.evaluate(async ({ i, which }) => {
        const cell = document.querySelectorAll("div.ikigai-run")[i];
        const caption = cell.querySelector(".ikigai-run-caption");
        const button = which === "run"
            ? cell.querySelector("button.ikigai-run-button")
            : [...cell.querySelectorAll("button")].find((b) => b.textContent === "Run all above");
        button.click();
        const deadline = Date.now() + 30000;
        while (/running…|that had not run…$/.test(caption.textContent) && Date.now() < deadline) {
            await new Promise((r) => setTimeout(r, 20));
        }
        const metas = cell.querySelectorAll(".ikigai-run-entry-meta");
        return {
            caption: caption.textContent,
            meta: metas.length ? metas[metas.length - 1].textContent : "",
            out: cell.querySelector(".ikigai-run-out").textContent,
            expected: cell.querySelector(".ikigai-run-expected").textContent,
        };
    }, { i, which });
}

// In the page: when cell `i` runs in a stages viewer's chain, wait for its tab to be open and the
// After picture below the tabs to be one the page's kernel drew (a `blob:` image, never the
// chapter's committed file), captioned as the kernel's. Returns a problem, or null (also for a
// cell in no viewer's chain).
async function stagePicture(page, i) {
    return page.evaluate(async (i) => {
        const cell = document.querySelectorAll("div.ikigai-run")[i];
        const inChain = [...document.querySelectorAll(".ikigai-stages[data-chain]")].some((viewer) => {
            const [kind, name] = viewer.getAttribute("data-chain").split(":");
            return cell.getAttribute(`data-${kind}`) === name;
        });
        if (!inChain) {
            return null;
        }
        // The cell's tab is the open one, and After (`figure.ikigai-stages-after`) is what this
        // step drew: a picture the kernel drew, in the copy the theme shows, said to be the kernel's.
        const viewer = cell.closest(".ikigai-stages");
        const section = cell.closest("section.ikigai-stage");
        const deadline = Date.now() + 15000;
        while (Date.now() < deadline) {
            const compare = viewer && viewer.querySelector(".ikigai-stages-compare");
            const after = compare && compare.querySelector("figure.ikigai-stages-after");
            const caption = after && after.querySelector("figcaption");
            const drawn = after && [...after.querySelectorAll('.ikigai-stage-picture img[src^="blob:"]')]
                .some((img) => img.getClientRects().length > 0);
            // A viewer with a picture above the tabs (`data-overview`) redraws that one too.
            const overview = viewer && viewer.getAttribute("data-overview")
                ? document.getElementById(viewer.getAttribute("data-overview-target") || "")
                : null;
            const overviewDrawn = !viewer || !viewer.getAttribute("data-overview") ||
                (overview && overview.querySelector('img[src^="blob:"]'));
            if (compare && !compare.hidden && (!section || !section.hidden) && drawn &&
                caption && /^After:.*as your kernel drew it/.test(caption.textContent) && overviewDrawn) {
                return null;
            }
            await new Promise((r) => setTimeout(r, 50));
        }
        const status = [...document.querySelectorAll(".ikigai-stages-status")].map((s) => s.textContent).join(" | ");
        const after = viewer && viewer.querySelector("figure.ikigai-stages-after figcaption");
        return `After, below the tabs, is not a picture this step drew (After says ${JSON.stringify(after ? after.textContent : "(no After)")}; the viewer said: ${JSON.stringify(status)})`;
    }, i);
}

// In the page, after its cells ran in order: drive each stages viewer's "Compare with" (a native
// <select>, js/stages.js). On the last tab that has two stages before it, the select must offer
// exactly the stages before the tab, default to the one just before, and, set to the first
// stage, put that stage in Before, change "What changed" to start from it, and say so in the
// status region; switching tabs puts it back to the stage just before. Returns problems.
async function compareWith(page) {
    return page.evaluate(async () => {
        const problems = [];
        const settle = () => new Promise((r) => setTimeout(r, 50));
        for (const viewer of document.querySelectorAll(".ikigai-stages[data-picture]")) {
            const compare = viewer.querySelector(".ikigai-stages-compare");
            if (!compare || compare.hidden) {
                continue;
            }
            const name = viewer.querySelector('[role="tablist"]').getAttribute("aria-label");
            const tabs = [...viewer.querySelectorAll('[role="tab"]')];
            const select = compare.querySelector("select");
            const label = select && select.labels && select.labels[0];
            if (!select || !label || label.textContent.trim() !== "Compare with" || label.hidden) {
                problems.push(`${name}: "Compare with" is not a <select> with its visible label`);
                continue;
            }
            const k = tabs.length - 1;
            if (k < 2) {
                continue;
            }
            const beforeCaption = () => compare.querySelector("figure.ikigai-stages-before figcaption").textContent;
            const lead = () => compare.querySelector(".ikigai-stages-diff strong").textContent;
            const status = viewer.querySelector(".ikigai-stages-status");
            tabs[k].click();
            await settle();
            const options = [...select.options].map((o) => o.textContent);
            const want = tabs.slice(0, k).map((t) => t.textContent);
            if (JSON.stringify(options) !== JSON.stringify(want)) {
                problems.push(`${name}, tab ${k + 1}: "Compare with" offers ${JSON.stringify(options)}, not the stages before it ${JSON.stringify(want)}`);
                continue;
            }
            if (select.selectedIndex !== k - 1) {
                problems.push(`${name}, tab ${k + 1}: "Compare with" starts at option ${select.selectedIndex + 1}, not the stage just before`);
            }
            const was = { caption: beforeCaption(), lead: lead() };
            select.value = select.options[0].value;
            select.dispatchEvent(new Event("change", { bubbles: true }));
            await settle();
            const first = tabs[0].textContent.replace(/^Stage /, "");
            if (beforeCaption() === was.caption || !beforeCaption().includes(`stage ${first}`)) {
                problems.push(`${name}, tab ${k + 1}: picking stage 1 in "Compare with" left Before as ${JSON.stringify(beforeCaption())}`);
            }
            if (lead() === was.lead || !lead().startsWith(`What changed from stage ${first} `)) {
                problems.push(`${name}, tab ${k + 1}: picking stage 1 left "What changed" as ${JSON.stringify(lead())}`);
            }
            if (!status.textContent.includes(`stage ${first}`)) {
                problems.push(`${name}, tab ${k + 1}: picking stage 1 was not said in the status region (${JSON.stringify(status.textContent)})`);
            }
            tabs[k - 1].click();
            await settle();
            if (select.selectedIndex !== k - 2) {
                problems.push(`${name}: opening tab ${k} left "Compare with" at option ${select.selectedIndex + 1}, not the stage just before`);
            }
            tabs[0].click();
            await settle();
            if (!select.closest(".ikigai-stages-against").hidden) {
                problems.push(`${name}: on the first tab "Compare with" is still shown, with nothing before it`);
            }
        }
        return problems;
    });
}

async function open(context, siteUrl, rel) {
    const page = await context.newPage();
    await page.goto(`${ORIGIN}${siteUrl}${rel}`, { waitUntil: "load" });
    const loaded = await page.evaluate(() =>
        window.ikigaiBookKernel().then(() => null, (e) => String(e && e.message ? e.message : e)));
    return { page, loaded };
}

function difference(got, want) {
    const g = got.split("\n").filter((l) => l.trim());
    const w = want.split("\n").filter((l) => l.trim());
    for (let k = 0; k < Math.max(g.length, w.length); k++) {
        if (g[k] !== w[k]) {
            return `line ${k + 1}: got ${JSON.stringify(g[k] ?? "(nothing)")}, the chapter shows ${JSON.stringify(w[k] ?? "(nothing)")}`;
        }
    }
    return "only in a `…` the chapter elides";
}

export async function checkCells(browser, siteArg) {
    const siteUrl = siteUrlOf(siteArg);
    const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
    await context.route("**/*", server(siteArg, siteUrl));

    const failures = [];
    let checked = 0;
    let shortcuts = 0;
    for (const path of pages(siteArg)) {
        if (!readFileSync(path, "utf8").includes('class="ikigai-run"')) {
            continue;
        }
        const rel = relative(siteArg, path).split("\\").join("/");
        const { page, loaded } = await open(context, siteUrl, rel);
        if (loaded !== null) {
            failures.push(`${rel}: the in-page kernel did not load (${loaded}); build it first (scripts/build-kernels.sh)`);
            await page.close();
            continue;
        }
        const count = await page.locator("div.ikigai-run").count();
        const withAbove = [];
        for (let i = 0; i < count; i++) {
            const r = await press(page, i, "run");
            checked += 1;
            if (!r.meta.includes(AS_SHOWN) && !r.meta.includes(NOT_COMPARED)) {
                failures.push(`${rel} cell ${i + 1}, run in page order: "${r.meta}" — ${difference(r.out, r.expected)}`);
            }
            // A cell in a stages viewer's chain leaves its tab open with the picture it drew as
            // After, below the tabs (`figure.ikigai-stages-after`, js/stages.js).
            const picture = await stagePicture(page, i);
            if (picture !== null) {
                failures.push(`${rel} cell ${i + 1}, run in page order: ${picture}`);
            }
            if (await page.locator("div.ikigai-run").nth(i).getByRole("button", { name: "Run all above" }).count()) {
                withAbove.push(i);
            }
        }
        for (const problem of await compareWith(page)) {
            failures.push(`${rel}, after its cells ran in page order: ${problem}`);
        }
        await page.close();

        for (const i of withAbove) {
            // Alone, on a fresh load: a run that differs says the cells above have not run.
            let fresh = await open(context, siteUrl, rel);
            const alone = await press(fresh.page, i, "run");
            if (!alone.meta.includes(AS_SHOWN) && !/cells? above (has|have) not run in this page yet: press “Run all above”/.test(alone.caption)) {
                failures.push(`${rel} cell ${i + 1}, run alone: differs and does not point at "Run all above": ${JSON.stringify(alone.caption)}`);
            }
            await fresh.page.close();
            // "Run all above", then Run, on a fresh load: what the chapter shows.
            fresh = await open(context, siteUrl, rel);
            await press(fresh.page, i, "above");
            const after = await press(fresh.page, i, "run");
            shortcuts += 1;
            if (!after.meta.includes(AS_SHOWN) && !after.meta.includes(NOT_COMPARED)) {
                failures.push(`${rel} cell ${i + 1}, after "Run all above": "${after.meta}" — ${difference(after.out, after.expected)}`);
            }
            await fresh.page.close();
        }
    }
    await context.close();
    return { failures, summary: `cells against the chapter: ${checked} cell(s) run in page order, ${shortcuts} run after "Run all above", ${failures.length} failure(s)` };
}

// On its own, from the command line.
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
    const args = process.argv.slice(2);
    const siteArg = args.find((a, i) => !a.startsWith("--") && args[i - 1] !== "--chrome");
    if (!siteArg) {
        console.error("usage: node cells.mjs <assembled site directory> [--chrome <path>]");
        process.exit(2);
    }
    const chromeAt = args.includes("--chrome") ? args[args.indexOf("--chrome") + 1] : null;
    const browser = await chromium.launch(chromeAt ? { executablePath: chromeAt } : { channel: "chrome" });
    console.log(`browser: ${chromeAt || "Google Chrome (the installed one)"} ${browser.version()}`);
    const { failures, summary } = await checkCells(browser, siteArg);
    await browser.close();
    failures.forEach((line) => console.error(line));
    console.log(summary);
    process.exit(failures.length ? 1 : 0);
}
