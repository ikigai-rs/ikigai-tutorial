// The resilience harness: its picture, its words, and the controls that drive it.
//
// A chapter writes
//
//     <figure class="ikigai-harness" data-harness='overlays'>
//     <a href="harness-start.svg"><img src="harness-start.svg" alt="…"></a>
//     <figcaption>…</figcaption>
//     <ul class="ikigai-harness-controls" hidden>
//     <li data-cmd='sink urn:iki:tutorial:harness:mode fail'>Set the upstream to fail</li>
//     …
//     </ul>
//     </figure>
//
// around a committed picture of the harness as it starts. A cell carrying the same
// `data-harness='overlays'` runs its lines in that harness (`js/run.js`, chain
// `harness:overlays`), and so does each control, which this script turns into a button named by
// its text once the kernel has loaded. After every such cell or control, this script asks the
// harness for its picture (`harnessPictureAsync`: `urn:diagram:state`, the arrangement with the
// circuit breaker's state drawn over it) and draws it in place of the one on the page.
//
// ONE PERSISTENT STATUS REGION, polite, outside the picture: the picture is redrawn by replacing
// it, and a region inside the replaced picture is a region that is never heard. So the words
// are `urn:throttle:state` in text/plain, the board's own words, refreshed in this one region
// after every step: what the control answered, then what the board says. Every state is a word
// there ("open", "half-open", "closed"), never a color alone.
//
// NOTHING RUNS ON LOAD. The committed picture IS the harness as it starts (a test in
// `crates/architecture-book-wasm` holds the two byte for byte), and asking for it before the
// reader does anything would start the harness for nothing.
//
// WITHOUT THE KERNEL there are no controls (the list stays hidden: a button that cannot do
// anything is not offered), the committed picture stays, which is still the truth about a
// harness that never started, and the status region says so. `a11y/no-kernel.mjs` pins that.
(function () {
    "use strict";

    var figures = document.querySelectorAll("figure.ikigai-harness[data-harness]");
    if (figures.length === 0) {
        return;
    }

    function loadKernel() {
        if (typeof window.ikigaiBookKernel === "function") {
            return window.ikigaiBookKernel();
        }
        return Promise.reject(new Error("js/run.js, which loads it, is not on this page"));
    }

    function message(err) {
        return err && err.message ? err.message : String(err);
    }

    // The board's words on one line each, as the region says them.
    function oneLine(text) {
        return text.split("\n").map(function (l) { return l.trim(); })
            .filter(function (l) { return l.length > 0; }).join("; ");
    }

    // What a run of a control's lines answered, in words: each distinct answer once, with how
    // many lines gave it, so "ten calls, of which one reached the upstream" can be heard.
    function answers(replies) {
        var seen = [];
        var counts = {};
        replies.forEach(function (reply) {
            var said = reply.kind === "error" ? "error: " + reply.text : reply.text;
            said = oneLine(said || "");
            if (!(said in counts)) {
                seen.push(said);
                counts[said] = 0;
            }
            counts[said] += 1;
        });
        return seen.map(function (said) {
            return (replies.length > 1 ? counts[said] + " × " : "") + said;
        }).join("; ");
    }

    function install(figure) {
        var name = figure.getAttribute("data-harness");
        var chain = "harness:" + name;
        var img = figure.querySelector("img");
        var link = img && img.parentNode && img.parentNode.tagName === "A" ? img.parentNode : null;
        var list = figure.querySelector(".ikigai-harness-controls");

        var status = document.createElement("p");
        status.className = "ikigai-harness-status";
        status.setAttribute("role", "status");
        status.setAttribute("aria-live", "polite");
        figure.appendChild(status);

        var drawn = null;   // the object URL of the picture now on the page, if it is not the file
        var busy = false;
        var buttons = [];

        // Redraw the picture, then say `lead` and what the board says now.
        function redraw(lead) {
            return loadKernel().then(function (mod) {
                return mod.harnessPictureAsync(name);
            }).then(function (json) {
                var answer = JSON.parse(json);
                if (answer.error) {
                    status.textContent = lead + " The picture could not be redrawn: " + answer.error;
                    return;
                }
                if (img) {
                    var url = URL.createObjectURL(new Blob([answer.svg], { type: "image/svg+xml" }));
                    img.alt = answer.alt;
                    img.src = url;
                    if (link) {
                        link.href = url;
                    }
                    if (drawn) {
                        URL.revokeObjectURL(drawn);
                    }
                    drawn = url;
                }
                var caption = figure.querySelector("figcaption");
                if (caption) {
                    caption.textContent = "The harness in this page, as its kernel draws " +
                        "urn:diagram:state after the last step you took in it. Select the " +
                        "picture to open it at full size.";
                }
                status.textContent = lead + " The board says: " + oneLine(answer.words);
            }).catch(function (err) {
                status.textContent = lead + " The picture could not be redrawn (" + message(err) + ").";
            });
        }

        // Run one control: its lines in order, in the harness, then redraw and say what happened.
        function press(label, lines) {
            if (busy) {
                return Promise.resolve();
            }
            busy = true;
            buttons.forEach(function (b) { b.disabled = true; });
            status.textContent = label + ": running…";
            return loadKernel().then(function (mod) {
                var replies = [];
                return lines.reduce(function (done, line) {
                    return done.then(function () {
                        return mod.evalLineInChainAsync(chain, line).then(function (json) {
                            replies.push(JSON.parse(json));
                        });
                    });
                }, Promise.resolve()).then(function () {
                    return redraw(label + ": " + answers(replies) + ".");
                });
            }).catch(function (err) {
                status.textContent = label + ": the page failed while running it (" + message(err) + ").";
            }).then(function () {
                busy = false;
                buttons.forEach(function (b) { b.disabled = false; });
            });
        }

        loadKernel().then(function () {
            if (!list) {
                return;
            }
            var row = document.createElement("div");
            row.className = "ikigai-run-buttons ikigai-harness-buttons";
            row.setAttribute("role", "group");
            row.setAttribute("aria-label", "Drive the harness");
            var items = list.querySelectorAll("li[data-cmd]");
            for (var i = 0; i < items.length; i++) {
                (function (item) {
                    var label = item.textContent.trim();
                    var lines = item.getAttribute("data-cmd").split("\n")
                        .map(function (l) { return l.trim(); })
                        .filter(function (l) { return l.length > 0; });
                    // The visible text IS the button's accessible name (SC 2.5.3 Label in Name).
                    var button = document.createElement("button");
                    button.type = "button";
                    button.className = "ikigai-run-secondary";
                    button.textContent = label;
                    button.addEventListener("click", function () {
                        press(label, lines);
                    });
                    buttons.push(button);
                    row.appendChild(button);
                })(items[i]);
            }
            // The buttons ARE the list now: the hidden markup they were built from goes.
            list.parentNode.replaceChild(row, list);
            status.textContent = "The harness is ready: each button runs its step in the harness " +
                "in this page, then the picture is redrawn and this line says what the board holds.";
        }).catch(function (err) {
            status.textContent = "The in-page kernel did not load (" + message(err) + "), so " +
                "the harness cannot be driven and this picture is not redrawn: it shows the " +
                "harness as it starts, every circuit closed.";
        });

        document.addEventListener("ikigai:cell-ran", function (event) {
            if (event.detail && event.detail.chain === chain) {
                redraw("After the cell you ran in the harness:");
            }
        });
    }

    for (var i = 0; i < figures.length; i++) {
        install(figures[i]);
    }
})();
