// Runnable cells: the book's own kernel, in the page.
//
// A chapter writes
//
//     <div class="ikigai-run" data-cmd='source urn:iki:tutorial:title
//     source urn:iki:tutorial:title'>
//     <pre class="ikigai-run-expected">…what the listing produces…</pre>
//     </div>
//
// and this script turns it into the command, a Run button and an output pane. Run sends
// each line to `crates/book-wasm` — `hello_camel::kernel()` under the CLI's engine,
// compiled to WebAssembly by pages.yml — and shows what came back with the engine's cache
// verdict, exactly as `ikigai --plain` prints it.
//
// Two rules the markup relies on. `data-cmd` is delimited by SINGLE quotes, because a
// REPL line can carry double quotes (`in="a b"`), and the URN gate (`crates/book-urns`)
// reads the attribute the same way. And the expected output is the listing's output —
// the tests in the crate are the source of truth for it — so if the wasm does not load,
// the cell shows that instead and says so; the page is never broken by the kernel being
// absent.
//
// ⚠ One CommonMark rule bites here: an HTML block that starts with `<div` ENDS at the
// first blank line, and everything after it is ordinary markdown again — so a blank line
// inside the expected `<pre>` closes the cell early and swallows the rest of the chapter
// into it (found on the trace cell, whose output has one). Write `&#32;` on a line that
// should read as blank; the URN gate and this script are unaffected either way.
//
// One kernel per page, shared by every cell: the cache and the golden threads live in it,
// and "cached the second time" is only demonstrable against state that persists. So
// the order cells are run in matters, and a second Run of the same cell answers
// differently. That is the lesson, not a bug, and the chapter says so.
(function () {
    "use strict";

    // mdbook defines `path_to_root` in every page's head; the wasm lives beside the
    // book's root, in `wasm/`. Falls back to the current directory for a page served
    // from an unusual place.
    var root = typeof path_to_root === "string" ? path_to_root : "./";

    var kernel = null;   // resolves to the wasm module's exports, or rejects once
    function load() {
        if (kernel) {
            return kernel;
        }
        // A dynamic import inside a classic script: supported everywhere wasm is.
        kernel = import(root + "wasm/book_wasm.js").then(function (mod) {
            return mod.default({ module_or_path: root + "wasm/book_wasm_bg.wasm" })
                .then(function () { return mod; });
        });
        return kernel;
    }

    function el(tag, className, text) {
        var node = document.createElement(tag);
        if (className) {
            node.className = className;
        }
        if (text !== undefined) {
            node.textContent = text;
        }
        return node;
    }

    // What one evaluated line looks like in the pane — the text, then the engine's cache
    // verdict in brackets on its own line, the way `ikigai --plain` prints it.
    function render(line, reply) {
        var out = "";
        if (reply.kind === "error") {
            out += "error: " + reply.text + "\n";
        } else if (reply.text) {
            out += reply.text;
            if (!/\n$/.test(reply.text)) {
                out += "\n";
            }
        }
        if (reply.cache) {
            out += "[" + reply.cache + "]\n";
        }
        return out;
    }

    function install(cell) {
        var cmd = cell.getAttribute("data-cmd") || "";
        var lines = cmd.split("\n").map(function (l) { return l.trim(); })
            .filter(function (l) { return l.length > 0; });
        var expected = cell.querySelector(".ikigai-run-expected");
        var expectedText = expected ? expected.textContent : "";

        cell.textContent = "";

        var head = el("div", "ikigai-run-head");
        var cmdPane = el("pre", "ikigai-run-cmd");
        lines.forEach(function (line) {
            var row = el("code", "ikigai-run-line");
            row.appendChild(el("span", "ikigai-run-prompt", "ikigai> "));
            row.appendChild(document.createTextNode(line));
            cmdPane.appendChild(row);
            cmdPane.appendChild(document.createTextNode("\n"));
        });
        head.appendChild(cmdPane);

        // The visible text IS the accessible name — no `aria-label`. One that said "Run in
        // this page" stopped containing the visible label the moment the button read "Run
        // again" (WCAG 2.2 SC 2.5.3 Label in Name), and a speech-input user saying what
        // they see would have missed it. What the button acts on is the command pane, and
        // that relationship is a description, not a name.
        cellCount += 1;
        cmdPane.id = "ikigai-run-cmd-" + cellCount;
        var button = el("button", "ikigai-run-button", "Run");
        button.type = "button";
        button.setAttribute("aria-describedby", cmdPane.id);
        head.appendChild(button);
        cell.appendChild(head);

        // Caption and output are ONE status region (SC 4.1.3 Status Messages): "running…",
        // "from the kernel in this page" and "the in-page kernel did not load" are the
        // status of an action the reader took without moving focus, so they have to be
        // announced — and a caption outside the region would be the message that never
        // reaches a screen reader. `aria-atomic` so a change to either half re-reads both.
        var result = el("div", "ikigai-run-result");
        result.setAttribute("role", "status");
        result.setAttribute("aria-live", "polite");
        result.setAttribute("aria-atomic", "true");
        var caption = el("div", "ikigai-run-caption", "expected output — what the listing produces");
        result.appendChild(caption);
        var pane = el("pre", "ikigai-run-out", expectedText);
        result.appendChild(pane);
        cell.appendChild(result);

        var runs = 0;
        button.addEventListener("click", function () {
            button.disabled = true;
            caption.textContent = "running…";
            load().then(function (mod) {
                var acc = "";
                // Sequential on purpose: a later line may depend on an earlier one's
                // side effect (a Sink, a cut), so they run in order, one at a time.
                return lines.reduce(function (chain, line) {
                    return chain.then(function () {
                        return mod.evalLineAsync(line).then(function (json) {
                            acc += render(line, JSON.parse(json));
                        });
                    });
                }, Promise.resolve()).then(function () {
                    runs += 1;
                    pane.textContent = acc;
                    caption.textContent = runs === 1
                        ? "from the kernel in this page"
                        : "from the kernel in this page — run " + runs + " (the same kernel; note what is cached now)";
                    button.disabled = false;
                    button.textContent = "Run again";
                });
            }).catch(function (err) {
                pane.textContent = expectedText;
                caption.textContent =
                    "the in-page kernel did not load (" + (err && err.message ? err.message : err) +
                    ") — this is the output the listing produces";
                button.disabled = true;
                if (window.console) {
                    console.warn("ikigai-run: wasm did not load", err);
                }
            });
        });
    }

    var cellCount = 0;

    function installAll() {
        var cells = document.querySelectorAll("div.ikigai-run[data-cmd]");
        for (var i = 0; i < cells.length; i++) {
            install(cells[i]);
        }
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", installAll);
    } else {
        installAll();
    }
})();
