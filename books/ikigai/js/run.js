// Runnable cells: the book's own kernel, in the page — and nothing runs until you say so.
//
// A chapter writes
//
//     <div class="ikigai-run" data-cmd='source urn:iki:tutorial:title
//     source urn:iki:tutorial:title'>
//     <pre class="ikigai-run-expected">…what the listing produces…</pre>
//     </div>
//
// and this script turns it into an editable command, Run / Reset / Clear buttons, an
// output pane, and a history of runs. Run sends each line of the command to
// `crates/book-wasm` — `hello_camel::kernel()` under the CLI's engine, compiled to
// WebAssembly by pages.yml — and shows what came back with the engine's cache verdict,
// exactly as `ikigai --plain` prints it.
//
// Three things are deliberate about how it behaves.
//
// NOTHING RUNS ON LOAD except loading the kernel, and NOTHING IS ANSWERED IN ADVANCE: a
// cell shows its command and an empty result until the reader presses Run. The listing's
// own output sits behind a disclosure ("Expected output") for whoever wants to check, and
// stands in for the result only when the kernel cannot load. "Cached the second time" is
// a thing the reader does, twice, and watches change; a page that had already shown the
// answer would have taken the payoff away (Brian, on the first version, which did).
//
// THE COMMAND IS EDITABLE. It is a real textarea (one row per line, monospace): Enter
// runs it, Shift+Enter adds a line, Run runs it, Reset restores the chapter's original.
// Edits are free-form. A name the kernel does not bind answers the way the CLI answers —
// `error: no endpoint resolved for urn:…` — which is itself a lesson, not a failure of
// the page. The URN gate (`crates/book-urns`) checks the ORIGINAL `data-cmd` only;
// what the reader types is theirs.
//
// EVERY RUN IS KEPT. A short history under the cell — command, output, cache verdict,
// elapsed — newest last, so "run it again" has something to compare against. Clear
// empties it. It is a history, not a REPL.
//
// Two rules the markup relies on. `data-cmd` is delimited by SINGLE quotes, because a
// REPL line can carry double quotes (`in="a b"`). And the expected output is the
// listing's output — the tests in the crate are the source of truth for it — so if the
// wasm does not load, the cell keeps showing that and says so; the page is never broken
// by the kernel being absent.
//
// ⚠ One CommonMark rule bites here: an HTML block that starts with `<div` ENDS at the
// first blank line, and everything after it is ordinary markdown again — so a blank line
// inside the expected `<pre>` closes the cell early and swallows the rest of the chapter
// into it. Write `&#32;` on a line that should read as blank.
//
// One kernel per page, shared by every cell: the cache and the golden threads live in it,
// and "cached the second time" is only demonstrable against state that persists. So
// the order cells are run in matters, and that is the lesson, not a bug.
(function () {
    "use strict";

    // mdbook defines `path_to_root` in every page's head; the wasm lives beside the
    // book's root, in `wasm/`.
    var root = typeof path_to_root === "string" ? path_to_root : "./";

    var kernel = null;   // resolves to the wasm module's exports, or rejects once
    function load() {
        if (kernel) {
            return kernel;
        }
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

    function now() {
        return (window.performance && performance.now) ? performance.now() : Date.now();
    }

    // What one evaluated line looks like — the text, then the engine's cache verdict in
    // brackets on its own line, the way `ikigai --plain` prints it. An error prints as
    // the CLI prints it: `error: …`.
    function render(reply) {
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

    function linesOf(text) {
        return text.split("\n").map(function (l) { return l.trim(); })
            .filter(function (l) { return l.length > 0; });
    }

    // One status line for the page, before the first cell: loading, ready, or absent.
    // One region rather than one per cell, so a screen reader hears it once (WCAG 2.2
    // SC 4.1.3 Status Messages).
    function announceKernel(firstCell) {
        var status = el("p", "ikigai-run-kernel", "Loading the in-page kernel…");
        status.setAttribute("role", "status");
        status.setAttribute("aria-live", "polite");
        firstCell.parentNode.insertBefore(status, firstCell);
        load().then(function () {
            status.textContent =
                "The in-page kernel is ready: every Run on this page resolves for real, and " +
                "the command beside it can be edited first.";
        }).catch(function (err) {
            status.textContent =
                "The in-page kernel did not load (" + (err && err.message ? err.message : err) +
                "). Each cell shows the output its listing produces instead.";
        });
    }

    var cellCount = 0;

    function install(cell) {
        var original = cell.getAttribute("data-cmd") || "";
        var expected = cell.querySelector(".ikigai-run-expected");
        var expectedText = expected ? expected.textContent : "";
        cellCount += 1;
        var id = "ikigai-run-" + cellCount;

        cell.textContent = "";

        // ── the command, editable ────────────────────────────────────────────
        // A labelled textarea (SC 3.3.2 Labels or Instructions; SC 4.1.2), one row per
        // line, with the keys explained in a description the field points at.
        var form = el("form", "ikigai-run-head");
        var label = el("label", "ikigai-run-label", "Command");
        label.htmlFor = id + "-cmd";
        var field = el("textarea", "ikigai-run-cmd");
        field.id = id + "-cmd";
        field.value = original;
        field.rows = Math.max(1, linesOf(original).length);
        field.spellcheck = false;
        field.setAttribute("autocapitalize", "off");
        field.setAttribute("autocomplete", "off");
        field.setAttribute("aria-describedby", id + "-help");
        var help = el("span", "ikigai-run-help",
            "Enter runs it; Shift+Enter adds a line; Reset restores the chapter's command.");
        help.id = id + "-help";

        var buttons = el("div", "ikigai-run-buttons");
        // The visible text IS each button's accessible name (SC 2.5.3 Label in Name).
        var run = el("button", "ikigai-run-button", "Run");
        run.type = "submit";
        var reset = el("button", "ikigai-run-secondary", "Reset");
        reset.type = "button";
        var clear = el("button", "ikigai-run-secondary", "Clear history");
        clear.type = "button";
        clear.hidden = true;
        buttons.appendChild(run);
        buttons.appendChild(reset);
        buttons.appendChild(clear);

        form.appendChild(label);
        form.appendChild(field);
        form.appendChild(help);
        form.appendChild(buttons);
        cell.appendChild(form);

        // ── the latest result: one status region ─────────────────────────────
        // Caption and output pane together (SC 4.1.3): "running…", "run 2 — from the
        // kernel" and "the kernel did not load" are the status of what the reader did,
        // and a caption outside the region would be the message that is never heard.
        var result = el("div", "ikigai-run-result");
        result.setAttribute("role", "status");
        result.setAttribute("aria-live", "polite");
        result.setAttribute("aria-atomic", "true");
        var caption = el("div", "ikigai-run-caption",
            "press Run to resolve it in this page");
        var pane = el("pre", "ikigai-run-out", "");
        result.appendChild(caption);
        result.appendChild(pane);
        cell.appendChild(result);

        // The listing's own output stays available — behind a disclosure, so the
        // answer is not on the page before the reader has asked the question. It is
        // also what the pane shows when the kernel cannot load (the static fallback).
        var expectedWrap = el("details", "ikigai-run-expected-wrap");
        var expectedSummary = el("summary", "ikigai-run-expected-summary",
            "Expected output (what the listing produces)");
        expectedWrap.appendChild(expectedSummary);
        expectedWrap.appendChild(el("pre", "ikigai-run-expected", expectedText));
        cell.appendChild(expectedWrap);
        load().catch(function () {
            // Static fallback: no kernel, so the listing's output stands in, labelled.
            pane.textContent = expectedText;
            caption.textContent = "the in-page kernel did not load — this is the output the listing produces";
        });

        // ── the history ──────────────────────────────────────────────────────
        var history = el("ol", "ikigai-run-history");
        history.hidden = true;
        history.setAttribute("aria-label", "Runs of this cell, oldest first");
        cell.appendChild(history);
        var runs = 0;

        function record(commandText, output, verdict, elapsed) {
            runs += 1;
            var item = el("li", "ikigai-run-entry");
            var head = el("div", "ikigai-run-entry-head");
            head.appendChild(el("span", "ikigai-run-entry-n", "run " + runs));
            head.appendChild(el("code", "ikigai-run-entry-cmd", commandText.replace(/\n/g, " ⏎ ")));
            head.appendChild(el("span", "ikigai-run-entry-meta",
                (verdict || "no cache verdict") + " · " + elapsed.toFixed(1) + " ms"));
            item.appendChild(head);
            item.appendChild(el("pre", "ikigai-run-entry-out", output));
            history.appendChild(item);
            history.hidden = false;
            clear.hidden = false;
        }

        function execute() {
            var lines = linesOf(field.value);
            if (lines.length === 0) {
                caption.textContent = "nothing to run — the command is empty; Reset restores the chapter's";
                return;
            }
            run.disabled = true;
            caption.textContent = "running…";
            var started = now();
            load().then(function (mod) {
                var acc = "";
                var verdicts = [];
                // Sequential on purpose: a later line may depend on an earlier one's
                // side effect (a Sink, a cut), so they run in order, one at a time.
                return lines.reduce(function (chain, line) {
                    return chain.then(function () {
                        return mod.evalLineAsync(line).then(function (json) {
                            var reply = JSON.parse(json);
                            acc += render(reply);
                            if (reply.cache) {
                                verdicts.push(reply.cache);
                            } else if (reply.kind === "error") {
                                verdicts.push("error");
                            }
                        });
                    });
                }, Promise.resolve()).then(function () {
                    var elapsed = now() - started;
                    pane.textContent = acc;
                    record(lines.join("\n"), acc, verdicts.join(" · "), elapsed);
                    caption.textContent = "run " + runs + " — from the kernel in this page" +
                        (runs > 1 ? "; compare it with the runs below" : "");
                    run.disabled = false;
                });
            }).catch(function (err) {
                pane.textContent = expectedText;
                caption.textContent =
                    "the in-page kernel did not load (" + (err && err.message ? err.message : err) +
                    ") — this is the output the listing produces";
                run.disabled = true;
            });
        }

        form.addEventListener("submit", function (event) {
            event.preventDefault();
            execute();
        });
        field.addEventListener("keydown", function (event) {
            if (event.key === "Enter" && !event.shiftKey) {
                event.preventDefault();
                execute();
            }
        });
        // Reset puts the WHOLE cell back the way the chapter shipped it: the command,
        // the result pane, and the history — not just the text, which read as "nothing
        // happened" when the last run's output stayed on screen.
        reset.addEventListener("click", function () {
            field.value = original;
            field.rows = Math.max(1, linesOf(original).length);
            history.textContent = "";
            history.hidden = true;
            clear.hidden = true;
            runs = 0;
            pane.textContent = "";
            caption.textContent = "reset — the chapter's command is back; press Run to resolve it in this page";
            field.focus();
        });
        clear.addEventListener("click", function () {
            history.textContent = "";
            history.hidden = true;
            clear.hidden = true;
            runs = 0;
            pane.textContent = "";
            caption.textContent = "history cleared — press Run to resolve it in this page";
        });
    }

    function installAll() {
        var cells = document.querySelectorAll("div.ikigai-run[data-cmd]");
        if (cells.length === 0) {
            return;
        }
        for (var i = 0; i < cells.length; i++) {
            install(cells[i]);
        }
        announceKernel(cells[0]);
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", installAll);
    } else {
        installAll();
    }
})();
