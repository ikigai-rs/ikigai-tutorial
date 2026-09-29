// A page whose in-page kernel is missing answers NOTHING in advance.
//
//   node no-kernel.mjs ../books/ikigai/book [<another built book>…]   # exit 1 on any failure
//
// Every page of every book given that carries a runnable cell (`div.ikigai-run[data-cmd]`),
// a playable board (`div.ttt-play[data-game]`), a polling view (`div.ikigai-view[data-shell]`)
// or a sheet you can type into (`div.sheet-play[data-shell]`) is loaded in jsdom, with the book's own scripts run and
// the kernel ABSENT, and must come up like this:
//
//   cells    every result pane (`.ikigai-run-out`) empty; every Run button disabled, and
//            described (aria-describedby) by a caption that says the kernel did not load
//            and points at "Expected output"; that disclosure still closed and still
//            holding the listing's output; no history. Then every cell is poked the ways
//            a reader can — Run clicked, the form submitted, Enter pressed in the field —
//            and all of that must still hold.
//   status   the page's one kernel status line says the kernel did not load.
//   boards   no squares, no grid, no buttons: one message saying what did not load.
//   views    nothing that polls, no controls, no count: one message saying what did not load.
//   sheets   no grid, no formula bar, no field: one message saying what did not load.
//
// WHY THIS EXISTS. `js/run.js` once filled every cell's result with the listing's expected
// output whenever the wasm failed to load, and Brian met it on a drafts server that had lost
// its wasm: every cell on the page arrived already answered (2026-09-27). CI never saw it,
// because CI always builds the wasm — the no-kernel path is exactly the path nothing
// exercised. This pins it.
//
// HOW THE KERNEL IS MADE ABSENT, twice over. jsdom cannot evaluate a dynamic `import()`, so
// run.js's `import("…/wasm/book_wasm.js")` rejects, as a missing file would in a browser.
// That alone is jsdom's limitation rather than this test's decision, so the loader ALSO
// refuses every path under `wasm/`: a jsdom that one day learns module imports still gets
// no kernel here. And the test requires the status line to say the kernel did not load
// before it believes anything else — an empty pane while the load is merely PENDING would
// otherwise pass for the right reason's absence.
//
// A check that saw nothing is not a pass: with no cell, no board or no view across all the
// books given, it exits 1. CI passes the public build and the drafts build (`serve-with-drafts.sh
// build <dir>`), because a chapter's widgets can live in drafts until it is linked (the sheets
// do, while the spreadsheet arc is unlinked).
import { readFileSync } from "node:fs";
import { relative } from "node:path";
import { openPage, pages } from "./book-dom.mjs";

const books = process.argv.slice(2);
if (books.length === 0) {
    console.error("usage: node no-kernel.mjs <built book directory> [<another>…]");
    process.exit(2);
}

const noKernel = (rel) => rel === "wasm" || rel.startsWith("wasm/");
const settle = (ms) => new Promise((done) => setTimeout(done, ms));

// Wait, up to a bound, for the page to have reached its no-kernel state.
async function until(window, ready, ms = 3000) {
    const deadline = Date.now() + ms;
    while (Date.now() < deadline) {
        if (ready(window.document)) {
            return true;
        }
        await settle(25);
    }
    return ready(window.document);
}

const DID_NOT_LOAD = /did not load/;

function checkCells(doc, fail) {
    const cells = doc.querySelectorAll("div.ikigai-run");
    const status = doc.querySelector(".ikigai-run-kernel");
    if (cells.length && !(status && DID_NOT_LOAD.test(status.textContent))) {
        fail(`kernel status line does not say the kernel did not load: ${JSON.stringify(status && status.textContent)}`);
    }
    cells.forEach((cell, i) => {
        const n = `cell ${i + 1}`;
        const pane = cell.querySelector(".ikigai-run-out");
        if (!pane) {
            fail(`${n}: no result pane — run.js did not build the cell`);
            return;
        }
        if (pane.textContent !== "") {
            fail(`${n}: result pane is not empty: ${JSON.stringify(pane.textContent.slice(0, 80))}`);
        }
        const run = cell.querySelector(".ikigai-run-button");
        if (!run || !run.disabled) {
            fail(`${n}: Run is not disabled`);
        }
        const described = run && run.getAttribute("aria-describedby");
        const reason = described && doc.getElementById(described);
        if (!reason) {
            fail(`${n}: Run has no description a screen reader can hear (aria-describedby)`);
        } else if (!DID_NOT_LOAD.test(reason.textContent) || !/Expected output/.test(reason.textContent)) {
            fail(`${n}: Run's description does not say why and where to look: ${JSON.stringify(reason.textContent)}`);
        }
        const wrap = cell.querySelector("details.ikigai-run-expected-wrap");
        if (!wrap) {
            fail(`${n}: no "Expected output" disclosure`);
        } else {
            if (wrap.open) {
                fail(`${n}: the "Expected output" disclosure is open`);
            }
            const expected = wrap.querySelector(".ikigai-run-expected");
            if (!expected || expected.textContent.trim() === "") {
                fail(`${n}: the disclosure lost the listing's output`);
            }
        }
        if (cell.querySelector(".ikigai-run-history li")) {
            fail(`${n}: a run was recorded`);
        }
    });
    return cells.length;
}

function checkBoards(doc, fail) {
    const boards = doc.querySelectorAll("div.ttt-play[data-game]");
    boards.forEach((board) => {
        const n = `board ${board.getAttribute("data-game")}`;
        if (board.querySelector(".ttt-square, .ttt-grid, .ttt-status, button")) {
            fail(`${n}: shows squares or game state with no kernel to answer them`);
        }
        const notes = board.querySelectorAll(".ttt-unavailable");
        if (notes.length !== 1 || !DID_NOT_LOAD.test(notes[0].textContent)) {
            fail(`${n}: does not say what did not load: ${JSON.stringify(board.textContent.trim())}`);
        }
    });
    return boards.length;
}

function checkViews(doc, fail) {
    const views = doc.querySelectorAll("div.ikigai-view[data-shell]");
    views.forEach((view, i) => {
        const n = `view ${i + 1}`;
        if (view.querySelector("[hx-get], [hx-post], [hx-trigger], button, .ikigai-view-tally")) {
            fail(`${n}: shows something that polls or counts with no kernel to answer it`);
        }
        const notes = view.querySelectorAll(".ikigai-view-unavailable");
        if (notes.length !== 1 || !DID_NOT_LOAD.test(notes[0].textContent)) {
            fail(`${n}: does not say what did not load: ${JSON.stringify(view.textContent.trim())}`);
        }
    });
    return views.length;
}

function checkSheets(doc, fail) {
    const sheets = doc.querySelectorAll("div.sheet-play[data-shell]");
    sheets.forEach((sheet, i) => {
        const n = `sheet ${i + 1}`;
        if (sheet.querySelector("[hx-get], [hx-post], form, input, button, table, .sheet-recomputed")) {
            fail(`${n}: shows a grid or a formula bar with no kernel to answer it`);
        }
        const notes = sheet.querySelectorAll(".sheet-unavailable");
        if (notes.length !== 1 || !DID_NOT_LOAD.test(notes[0].textContent)) {
            fail(`${n}: does not say what did not load: ${JSON.stringify(sheet.textContent.trim())}`);
        }
    });
    return sheets.length;
}

let failures = 0;
let cellsSeen = 0;
let boardsSeen = 0;
let viewsSeen = 0;
let sheetsSeen = 0;
let pagesSeen = 0;
for (const book of books) {
    for (const path of pages(book)) {
        const raw = readFileSync(path, "utf8");
        if (!/class="ikigai-run"|class="ttt-play"|class="ikigai-view"|class="sheet-play"/.test(raw)) {
            continue;
        }
        const label = relative(book, path);
        const fail = (msg) => {
            failures += 1;
            console.error(`${label}: ${msg}`);
        };
        const window = await openPage(book, path, { refuse: noKernel });
        const doc = window.document;
        const hasCells = doc.querySelector("div.ikigai-run") !== null;
        const settled = await until(window, (d) => {
            const status = d.querySelector(".ikigai-run-kernel");
            const cellsDone = !hasCells || (status && DID_NOT_LOAD.test(status.textContent));
            const boardsDone = [...d.querySelectorAll("div.ttt-play[data-game]")]
                .every((b) => DID_NOT_LOAD.test(b.textContent));
            const viewsDone = [...d.querySelectorAll("div.ikigai-view[data-shell]")]
                .every((v) => DID_NOT_LOAD.test(v.textContent));
            const sheetsDone = [...d.querySelectorAll("div.sheet-play[data-shell]")]
                .every((v) => DID_NOT_LOAD.test(v.textContent));
            return cellsDone && boardsDone && viewsDone && sheetsDone;
        });
        if (!settled) {
            fail("never reached the no-kernel state within 3s (is the load still pending?)");
        }

        // As loaded, before anyone touches it.
        checkCells(doc, fail);
        // Then every way a reader can ask a cell to run.
        for (const cell of doc.querySelectorAll("div.ikigai-run")) {
            const run = cell.querySelector(".ikigai-run-button");
            const form = cell.querySelector("form");
            const field = cell.querySelector(".ikigai-run-cmd");
            if (run) {
                run.click();
            }
            if (form) {
                form.requestSubmit();
            }
            if (field) {
                field.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
            }
        }
        await settle(200);
        cellsSeen += checkCells(doc, (msg) => fail(`after Run: ${msg}`));
        boardsSeen += checkBoards(doc, fail);
        viewsSeen += checkViews(doc, fail);
        sheetsSeen += checkSheets(doc, fail);
        pagesSeen += 1;
        window.close();
    }
}

if (cellsSeen === 0 || boardsSeen === 0 || viewsSeen === 0 || sheetsSeen === 0) {
    console.error(`no-kernel: saw ${cellsSeen} cells, ${boardsSeen} boards, ${viewsSeen} views and ${sheetsSeen} sheets — ` +
        "a check that saw nothing is not a pass (pass the drafts build too; see the header)");
    process.exit(1);
}
if (failures) {
    console.error(`\nno-kernel: ${failures} failure${failures === 1 ? "" : "s"} across ${pagesSeen} pages`);
    process.exit(1);
}
console.log(`no-kernel: ${pagesSeen} pages, ${cellsSeen} cells, ${boardsSeen} boards, ${viewsSeen} views and ${sheetsSeen} sheets — nothing answered, every Run disabled`);
