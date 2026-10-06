// Every relative link in the ASSEMBLED site lands on a file that is in it.
//
//   node links.mjs ../target/site          # exit 1 on any link to nothing
//
// Why, and why over the assembled site. mdbook checks a link to another chapter's `.md` while
// it builds one book, and nothing else: a link to a built `.html`, or out of the book entirely,
// is passed through unread. The two books link to each other that way — The ikigai Book's
// "Where to go next" to `gonk/`, The gonk Book to `../index.html` and to the ikigai book's
// chapters — and those links mean something only once both books sit where Pages serves them.
// So this runs on what `scripts/assemble-site.sh` produced: the one directory that is uploaded.
//
// What counts: every `href` in every `.html` page that has no scheme and is not a bare
// fragment, resolved the way a browser resolves it — against the page's own URL, or its
// `<base href>` where it has one (mdbook's 404 page does). A target ending in `/` means its
// `index.html`. Fragments and queries are not checked: a fragment names an id inside a page,
// and this check is about pages.
import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, relative } from "node:path";

const [, , siteArg] = process.argv;
if (!siteArg) {
    console.error("usage: node links.mjs <assembled site directory>");
    process.exit(2);
}

// The site is served under one path prefix; every page's URL is that prefix plus its path.
// The prefix itself does not matter to resolution as long as it is applied consistently, and
// a `<base href>` naming the real one (`/ikigai-tutorial/`) is mapped back onto the directory.
const PREFIX = "/ikigai-tutorial/";
const ORIGIN = "http://site.test";

function pages(dir, out = []) {
    for (const name of readdirSync(dir)) {
        const path = join(dir, name);
        if (statSync(path).isDirectory()) {
            pages(path, out);
        } else if (name.endsWith(".html")) {
            out.push(path);
        }
    }
    return out.sort();
}

const scheme = /^[a-z][a-z0-9+.-]*:/i;
let broken = 0;
let checked = 0;
for (const page of pages(siteArg)) {
    const rel = relative(siteArg, page).split("\\").join("/");
    const html = readFileSync(page, "utf8");
    const base = html.match(/<base\s+href="([^"]+)"/i);
    const pageUrl = new URL(PREFIX + rel, ORIGIN);
    const against = base ? new URL(base[1], pageUrl) : pageUrl;
    // Inside a real tag only: a `href="…"` in a code sample is text (`&lt;base href=…`), and
    // its `<` is escaped, so it does not match here.
    for (const m of html.matchAll(/<(?:a|link|area)\b[^>]*?\shref="([^"]*)"/gi)) {
        const href = m[1].replace(/&amp;/g, "&");
        if (!href || href.startsWith("#") || scheme.test(href) || href.startsWith("//")) {
            continue;
        }
        const target = new URL(href, against);
        if (!target.pathname.startsWith(PREFIX)) {
            broken += 1;
            console.error(`${rel}: ${href} leaves the site (${target.pathname})`);
            continue;
        }
        let path = decodeURIComponent(target.pathname.slice(PREFIX.length));
        if (path === "" || path.endsWith("/")) {
            path += "index.html";
        }
        checked += 1;
        const file = join(siteArg, path);
        if (!existsSync(file) || statSync(file).isDirectory()) {
            broken += 1;
            console.error(`${rel}: ${href} → /${path}, which is not in the site`);
        }
    }
}

if (checked === 0) {
    console.error(`no relative links found under ${siteArg} — is it the assembled site?`);
    process.exit(1);
}
if (broken > 0) {
    console.error(`\n${broken} link(s) to nothing, of ${checked} checked`);
    process.exit(1);
}
console.log(`${checked} relative link(s), every one to a page in the site`);
