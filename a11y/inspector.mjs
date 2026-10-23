// The kernel inspector in a real browser: what axe does not check (Building ikigai UIs, the demo).
//
//   node inspector.mjs <site directory> [--chrome <path>]
//
// The site is the one `cargo run -p inspector --example axe_site -- <dir>` writes (axe runs over
// the same directory, through ikigai-ui's axe.yml). Requests are answered from the directory by
// Playwright's router, with the media types its `_types.json` records and the HTML door's CSP on
// every answer, so an inline script or style would fail here as it would at the door. Nothing
// leaves the machine. Checks, each printing ok or FAIL (exit 1 on any failure):
//
// 1. REFLOW (WCAG 2.2 SC 1.4.10): every page at 320 CSS pixels wide, in every theme, scrolls no
//    wider than the viewport.
// 2. KEYBOARD: the first Tab reaches the skip link, Enter on it moves focus into <main>, and the
//    next Tab from there reaches the view's first link; every nav link is reachable by Tab.
// 3. FOCUS AFTER A SWAP (SC 2.4.3, UI conventions D13): Enter on a nav link swaps <main> (htmx),
//    pushes the page's URL, and moves focus to the new view's heading; Back restores the view.
// 4. A LIVE REFETCH IS QUIET: an `ikigai:cut` (what live.js announces for a cut) refetches the
//    view in place, and neither moves focus nor writes the status region.
//
// The browser is the machine's installed Google Chrome (Playwright's `chrome` channel), as
// axe-browser.mjs runs it, or the one `--chrome` names.
import { chromium } from "playwright-core";
import { readFileSync, existsSync, statSync, readdirSync } from "node:fs";
import { join } from "node:path";

const args = process.argv.slice(2);
const dir = args[0];
if (!dir) {
    console.error("usage: node inspector.mjs <site directory> [--chrome <path>]");
    process.exit(2);
}
const chromeAt = args.includes("--chrome") ? args[args.indexOf("--chrome") + 1] : null;
const ORIGIN = "http://inspector.test";
// The HTML door's page CSP (ikigai-web's `HTML_DOOR_CSP`, which inspector-htmx serves).
const CSP = "default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'; " +
    "base-uri 'none'; frame-ancestors 'none'; form-action 'self'";
const types = JSON.parse(readFileSync(join(dir, "_types.json"), "utf8"));

async function serve(route) {
    const u = new URL(route.request().url());
    if (u.origin !== ORIGIN) return route.abort("blockedbyclient");
    let path = decodeURIComponent(u.pathname);
    // A page URL the inspector pushes (/inspect/threads) is the site's auto-theme page.
    const file = join(dir, path);
    if (!existsSync(file) || statSync(file).isDirectory()) {
        const page = join(dir, `${path}-auto.html`);
        if (!existsSync(page)) return route.fulfill({ status: 404, body: "not in the site" });
        path = `${path}-auto.html`;
    }
    return route.fulfill({
        status: 200,
        contentType: types[path] || "text/html; charset=utf-8",
        headers: { "Content-Security-Policy": CSP },
        body: readFileSync(join(dir, path)),
    });
}

const pages = readdirSync(join(dir, "inspect")).filter((f) => f.endsWith(".html")).sort();
if (pages.length === 0) {
    console.error(`no pages under ${dir}/inspect`);
    process.exit(1);
}

const browser = await chromium.launch(chromeAt ? { executablePath: chromeAt } : { channel: "chrome" });
console.log(`browser: ${chromeAt || "Google Chrome (the installed one)"} ${browser.version()}`);
let failures = 0;

async function check(name, viewport, run) {
    const context = await browser.newContext({ viewport });
    await context.route("**/*", serve);
    const page = await context.newPage();
    const errors = [];
    page.on("pageerror", (e) => errors.push(`page error: ${e.message}`));
    page.on("console", (m) => { if (m.type() === "error") errors.push(`console error: ${m.text()}`); });
    try {
        const why = await run(page);
        if (why) throw new Error(why);
        if (errors.length) throw new Error(errors.join("; "));
        console.log(`ok   ${name}`);
    } catch (e) {
        failures += 1;
        console.log(`FAIL ${name}: ${e.message}`);
    }
    await context.close();
}

const focused = (page) => page.evaluate(() => {
    const a = document.activeElement;
    return a ? `${a.tagName.toLowerCase()}${a.id ? "#" + a.id : ""}${a.className ? "." + a.className : ""}:${(a.textContent || "").trim().slice(0, 40)}` : "none";
});

// 1. Reflow at 320 px.
for (const file of pages) {
    await check(`320 px wide: ${file}`, { width: 320, height: 640 }, async (page) => {
        await page.goto(`${ORIGIN}/inspect/${file}`, { waitUntil: "load" });
        const wide = await page.evaluate(() => document.documentElement.scrollWidth);
        return wide > 320 ? `the page scrolls ${wide} px wide` : null;
    });
}

// 2. The keyboard path.
await check("keyboard: skip link, then into the view; every nav link by Tab", { width: 1024, height: 768 }, async (page) => {
    await page.goto(`${ORIGIN}/inspect/cache-auto.html`, { waitUntil: "load" });
    await page.keyboard.press("Tab");
    const first = await focused(page);
    if (!first.startsWith("a.ui-skip")) return `the first Tab reached ${first}, not the skip link`;
    await page.keyboard.press("Enter");
    await page.keyboard.press("Tab");
    const inMain = await page.evaluate(() => document.getElementById("main").contains(document.activeElement));
    if (!inMain) return `after the skip link, Tab reached ${await focused(page)}, outside <main>`;
    // From the top again, every nav link in order.
    await page.goto(`${ORIGIN}/inspect/cache-auto.html`, { waitUntil: "load" });
    const navs = await page.locator("header nav a").allTextContents();
    await page.keyboard.press("Tab");
    for (const name of navs) {
        await page.keyboard.press("Tab");
        const at = await focused(page);
        if (!at.endsWith(`:${name}`)) return `expected the nav link ${name}, Tab reached ${at}`;
    }
    return null;
});

// 3. Focus after a swap, the pushed URL, and Back.
await check("a nav link swaps <main>, pushes its URL, focuses the heading; Back restores", { width: 1024, height: 768 }, async (page) => {
    await page.goto(`${ORIGIN}/inspect/cache-auto.html`, { waitUntil: "load" });
    await page.waitForFunction(() => window.htmx);
    await page.focus("header nav a[href='/inspect/threads']");
    await page.keyboard.press("Enter");
    await page.waitForFunction(() => document.querySelector("#main h1") &&
        document.querySelector("#main h1").textContent === "Golden threads");
    await page.waitForFunction(() => document.activeElement && document.activeElement.tagName === "H1");
    const url = new URL(page.url()).pathname;
    if (url !== "/inspect/threads") return `the URL is ${url}, not /inspect/threads`;
    await page.goBack();
    await page.waitForFunction(() => document.querySelector("#main h1") &&
        document.querySelector("#main h1").textContent === "Cache", null, { timeout: 5000 });
    return null;
});

// 4. A live refetch is quiet.
await check("a cut refetches the view in place, quietly", { width: 1024, height: 768 }, async (page) => {
    await page.goto(`${ORIGIN}/inspect/cache-auto.html`, { waitUntil: "load" });
    await page.waitForFunction(() => window.htmx);
    await page.focus("header nav a[href='/inspect/threads']");
    const before = await focused(page);
    await page.evaluate(() => {
        document.querySelector("#main h1").setAttribute("data-old", "yes");
        document.body.dispatchEvent(new CustomEvent("ikigai:cut"));
    });
    await page.waitForFunction(() => document.querySelector("#main h1") &&
        !document.querySelector("#main h1").hasAttribute("data-old"));
    const after = await focused(page);
    if (after !== before) return `focus moved from ${before} to ${after}`;
    const status = await page.evaluate(() => document.getElementById("status").textContent);
    return status ? `the status region says "${status}"` : null;
});

await browser.close();
console.log(`${pages.length} page(s) at 320 px and 3 behavior checks, ${failures} failure(s)`);
process.exit(failures ? 1 : 0);
