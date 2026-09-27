// Loading a built page of the book into jsdom, with the book's own scripts run — shared by
// the two jsdom instruments: `axe.mjs` (accessibility) and `no-kernel.mjs` (a page whose
// in-page kernel is missing answers nothing).
import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { JSDOM, ResourceLoader, VirtualConsole } from "jsdom";

// The book is a PROJECT Pages site: `book.toml` sets `site-url = "/ikigai-tutorial/"`,
// and mdbook writes that as `<base href>` into 404.html (the page Pages serves for any
// missing path, at any depth). Loaded from a `file://` URL, that base resolves to nowhere,
// the page's scripts never run, and a tool audits a 404 the reader never sees — the sidebar
// toggle's `aria-expanded` showed up exactly that way. So every page is loaded at the
// URL it is served at, on a made-up origin, and this loader maps that origin back onto
// the built directory. Same scripts, same base, same result as the live site.
//
// A build with no `book.toml` beside it (the drafts build, which `serve-with-drafts.sh
// build <dir>` can put anywhere) is served from the origin's root.
export const ORIGIN = "http://book.test";

export function siteUrlOf(bookDir) {
    const toml = resolve(bookDir, "..", "book.toml");
    if (existsSync(toml)) {
        const m = readFileSync(toml, "utf8").match(/^site-url\s*=\s*"([^"]+)"/m);
        if (m) {
            return m[1].endsWith("/") ? m[1] : m[1] + "/";
        }
    }
    return "/";
}

// `refuse(path)` — a path relative to the book — lets a caller make part of the built book
// absent, the way a server that lost it would: the fetch fails as a 404 would.
class BookLoader extends ResourceLoader {
    constructor(bookDir, siteUrl, refuse) {
        super();
        this.bookDir = bookDir;
        this.siteUrl = siteUrl;
        this.refuse = refuse;
    }

    fetch(url, options) {
        const u = new URL(url);
        if (u.origin !== ORIGIN || !u.pathname.startsWith(this.siteUrl)) {
            return null; // anything off-book (a CDN font, say) is not fetched
        }
        const rel = decodeURIComponent(u.pathname.slice(this.siteUrl.length));
        if (this.refuse && this.refuse(rel)) {
            return Promise.reject(new Error(`refused on purpose: ${u.pathname}`));
        }
        const path = join(this.bookDir, rel);
        if (!existsSync(path) || statSync(path).isDirectory()) {
            return Promise.reject(new Error(`not in the built book: ${u.pathname}`));
        }
        return Promise.resolve(readFileSync(path));
    }
}

// Every page a reader can land on. `print.html` (every chapter concatenated) and `toc.html`
// (the sidebar as a fragment mdbook loads into pages, not a document) are skipped, as
// book-a11y skips them.
export function pages(dir, out = []) {
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

// Load one built page and wait for its load handlers (a11y.js, run.js, ttt.js, mdbook's
// book.js) to have run. The caller closes the window.
export async function openPage(bookDir, path, { refuse } = {}) {
    const siteUrl = siteUrlOf(bookDir);
    const html = readFileSync(path, "utf8");
    // jsdom logs every failed resource fetch; the wasm's dynamic import is one on
    // purpose, so keep the console quiet and inspect results instead.
    const virtualConsole = new VirtualConsole();
    const dom = new JSDOM(html, {
        url: ORIGIN + siteUrl + relative(bookDir, path).split("\\").join("/"),
        runScripts: "dangerously",
        resources: new BookLoader(bookDir, siteUrl, refuse),
        pretendToBeVisual: true,
        virtualConsole,
    });
    const { window } = dom;
    await new Promise((done) => {
        if (window.document.readyState === "complete") {
            done();
        } else {
            window.addEventListener("load", done);
            setTimeout(done, 3000);
        }
    });
    return window;
}
