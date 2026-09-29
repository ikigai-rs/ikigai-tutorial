// A sheet you can type into: htmx markup, answered by the book's own kernel.
//
// A chapter writes
//
//     <div class="sheet-play" data-shell='urn:iki:tutorial:sheet:template:page'>…a line for
//     readers without the kernel…</div>
//
// and this script fills it with the shell, the resource `data-shell` names, and lets htmx
// run it: the shell asks for the grid (`iki/tutorial/sheet/view/grid`) when it loads, and its
// formula bar posts to `iki/tutorial/sheet/view/edit`. Nothing here draws a cell.
//
// THE SHIM is `js/ttt.js`'s, with one addition: a form's fields. htmx fires
// `htmx:beforeRequest` before it sends anything; for a request from inside a sheet this
// cancels it, maps the relative path to a name by the book's one rule (`a/b/c` is `urn:a:b:c`;
// GET is Source, POST Sink, DELETE Delete), and asks the page's kernel, in the root's chain,
// with the form's fields as named arguments (`issueWithArgsAsync`), the way a server reads a
// form body. The answer goes to `htmx.swap`.
//
// THE PAGE IS A COPY, as on a board: the grid on the screen is HTML the kernel handed out
// earlier, and nothing cuts that. So after an edit, and after any runnable cell on the page
// runs in the root's chain, every sheet on the page draws its grid again. That is cheap, and
// that is the lesson: every value the change did not touch is served from the cache.
//
// THE CACHE VIEWER (`data-cache` on the host). Before the grid is drawn again, the page asks the
// kernel which of the sheet's values are not in its cache (`uncachedAmong`, a probe per name
// that resolves nothing), and says so under the sheet: those are the values the redraw
// computes. The names come from the resource `range:A1:D6`. It is host code, not a resource,
// because no resource can say it: `urn:kernel:cache` lists entries whose threads were cut until
// something reads them again, and an endpoint cannot ask the probe (spreadsheet-3.md says
// more). It is plain text, not a live region: the reply above it is the status that is
// announced, and this is there to read when you want it.
//
// A SHEET THAT POLLS (parts IV and V). A shell may mark elements `data-poll`: the grid, asked
// again every two seconds (`hx-trigger="load, every 2s"`), and part V's ticker, which writes
// the market's next price every three. Polling is what the markup asks for, and the page only
// lets it through: it starts PAUSED, behind a toggle (`aria-pressed`, labeled by the shell's
// `data-poll-label`), for the reasons `js/view.js` gives (WCAG 2.2 SC 2.2.2, and a page that
// polled on load would fill the cache before the chapter's cells ran). While paused, a request
// from a `data-poll` element is canceled before it reaches the kernel; the grid's first load,
// and a redraw after an edit, still go through. Each poll of the grid is counted, as a view's
// are: answered from the cache, or computed. That count is the in-page face of what an HTTP
// host does with a conditional GET: an answer that has not changed is not made again.
//
// WHAT IS OUTSIDE THE SHEET (`data-outside`, part V) does not redraw it. A write from the
// formula bar, or from a runnable cell, redraws every sheet at once: the page did it, so the
// page knows. A write from inside `data-outside` (the market) is somebody else's, and the grid
// finds out the only way a reader of a name ever finds out about a write it did not make: by
// asking again. That is the poll.
//
// Without the kernel (or htmx), a sheet is one sentence saying what did not load, and nothing
// that looks editable. `a11y/no-kernel.mjs` pins that in CI.
//
// ⚠ htmx is loaded the way `js/ttt.js` and `js/view.js` load it, and none of them shares its load
// with the others: a page with a sheet and a board would load htmx twice. No page has both.
(function () {
    "use strict";

    var hosts = document.querySelectorAll("div.sheet-play[data-shell]");
    if (hosts.length === 0) {
        return;
    }
    var root = typeof path_to_root === "string" ? path_to_root : "./";
    var WHOLE_SHEET = "urn:iki:tutorial:sheet:range:A1:D6";

    function loadHtmx() {
        if (window.htmx) {
            return Promise.resolve(window.htmx);
        }
        return new Promise(function (resolve, reject) {
            var script = document.createElement("script");
            script.src = root + "vendor/htmx-2.0.4.min.js";
            script.onload = function () { resolve(window.htmx); };
            script.onerror = function () { reject(new Error("htmx did not load")); };
            document.head.appendChild(script);
        });
    }

    function loadKernel() {
        if (typeof window.ikigaiBookKernel === "function") {
            return window.ikigaiBookKernel();
        }
        return Promise.reject(new Error("js/run.js, which loads it, is not on this page"));
    }

    function escape(text) {
        return String(text).replace(/[&<>"']/g, function (c) {
            return { "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" }[c];
        });
    }

    // The path ↔ IRI rule, as `js/ttt.js` states it: relative paths only.
    function iriOf(path) {
        if (!path || path.charAt(0) === "/" || path.indexOf("://") !== -1) {
            return null;
        }
        var segments = path.split("?")[0].split("/");
        for (var i = 0; i < segments.length; i++) {
            if (segments[i] === "" || segments[i] === "." || segments[i] === "..") {
                return null;
            }
        }
        return "urn:" + segments.join(":");
    }

    var VERBS = { get: "source", post: "sink", delete: "delete" };

    function unavailable(host, what, err) {
        var why = err && err.message ? err.message : String(err);
        host.textContent = "";
        var note = document.createElement("p");
        note.className = "sheet-unavailable";
        note.textContent = "This sheet needs " + what + ", which did not load (" + why +
            "), so there is nothing to type into here. Each runnable cell's “Expected output” " +
            "shows what its request answers.";
        host.appendChild(note);
    }

    function labeled(promise, what) {
        return promise.catch(function (err) {
            throw { what: what, err: err };
        });
    }

    // A form's fields as [name, value] pairs, in form order.
    function fields(config) {
        var pairs = [];
        if (config.formData && typeof config.formData.forEach === "function") {
            config.formData.forEach(function (value, name) {
                pairs.push([name, String(value)]);
            });
        }
        return pairs;
    }

    Promise.all([
        labeled(loadHtmx(), "htmx"),
        labeled(loadKernel(), "the in-page kernel"),
    ]).then(function (loaded) {
        var htmx = loaded[0];
        var kernel = loaded[1];
        htmx.config.allowEval = false;
        htmx.config.historyEnabled = false;

        // Each sheet's polling state: paused or not, whether its grid has been drawn once,
        // whether the next grid request is a redraw the page asked for, and the poll count.
        var states = new Map();

        function stateOf(host) {
            var state = states.get(host);
            if (!state) {
                state = { polling: false, drawn: false, force: false, asked: 0, cached: 0,
                    computed: 0, tally: null };
                states.set(host, state);
            }
            return state;
        }

        function tallyText(state) {
            if (state.asked === 0) {
                return state.polling ? "Polling: waiting for the first answer."
                    : "Paused: the grid has not been polled.";
            }
            return (state.polling ? "Polling. " : "Paused. ") + "The grid was polled " +
                state.asked + (state.asked === 1 ? " time: " : " times: ") + state.cached +
                " answered from the cache, " + state.computed + " computed.";
        }

        // Say which of the sheet's values a redraw is about to compute: those not cached now.
        function showRecomputed(host) {
            var out = host.querySelector(".sheet-recomputed");
            if (!out) {
                return Promise.resolve();
            }
            return kernel.issueAsync("", "source", WHOLE_SHEET).then(function (json) {
                var reply = JSON.parse(json);
                if (reply.kind !== "output") {
                    out.textContent = "The page could not list the sheet's cells: " + reply.text;
                    return;
                }
                var names = reply.text.split("\n").filter(Boolean);
                var cells = kernel.uncachedAmong(names.join("\n")).split("\n").filter(Boolean)
                    .map(function (name) { return name.slice(name.lastIndexOf(":") + 1); });
                out.textContent = cells.length === 0
                    ? "The grid was drawn again entirely from the cache: no value was computed."
                    : "Computed again when the grid was drawn: " + cells.join(", ") + " (" +
                        cells.length + " of " + names.length + "). Every other value was " +
                        "served from the cache.";
            });
        }

        // Draw a sheet's grid again. The request goes through the shim like any other, which
        // lets the viewer (if the sheet has one) say first what the redraw is about to cost.
        function redraw(host) {
            var grid = host.querySelector(".sheet-view[hx-get]");
            if (!grid) {
                return;
            }
            stateOf(host).force = true;
            htmx.ajax("GET", grid.getAttribute("hx-get"), { source: grid, target: grid });
        }

        document.body.addEventListener("htmx:beforeRequest", function (event) {
            var detail = event.detail;
            var host = detail.elt && detail.elt.closest
                ? detail.elt.closest("div.sheet-play[data-shell]") : null;
            if (!host) {
                return;
            }
            event.preventDefault();
            var state = stateOf(host);
            var elt = detail.elt;
            var isGrid = elt.matches && elt.matches(".sheet-view[hx-get]");
            // The first draw, and a redraw the page asked for, are not polls.
            var poll = isGrid ? state.drawn && !state.force : elt.hasAttribute("data-poll");
            if (poll && !state.polling) {
                return;
            }
            var config = detail.requestConfig;
            var verb = VERBS[String(config.verb).toLowerCase()];
            var iri = iriOf(config.path);
            var target = detail.target;
            if (!verb || !iri) {
                htmx.swap(target, escape("the page cannot send " + config.verb + " " + config.path),
                    { swapStyle: "innerHTML" });
                return;
            }
            var outside = elt.closest && elt.closest("[data-outside]");
            var before = isGrid && state.drawn ? showRecomputed(host) : Promise.resolve();
            if (isGrid) {
                state.drawn = true;
                state.force = false;
            }
            var args = verb === "source" ? [] : fields(config);
            before.then(function () {
                return kernel.issueWithArgsAsync("", verb, iri, JSON.stringify(args));
            }).then(function (json) {
                var reply = JSON.parse(json);
                if (poll && isGrid && reply.kind === "output" && state.tally) {
                    state.asked += 1;
                    if (reply.cache === "cached") {
                        state.cached += 1;
                    } else {
                        state.computed += 1;
                    }
                    state.tally.textContent = tallyText(state);
                }
                var html = reply.kind === "output" ? reply.text : escape("error: " + reply.text);
                htmx.swap(target, html, { swapStyle: "innerHTML" });
                if (verb !== "source" && !outside) {
                    redraw(host);
                }
            }).catch(function (err) {
                htmx.swap(target, escape("error: " + (err && err.message ? err.message : err)),
                    { swapStyle: "innerHTML" });
            });
        });

        // A runnable cell with no game ran in the root's chain, which is the sheet's.
        document.addEventListener("ikigai:cell-ran", function (event) {
            if (event.detail && event.detail.game) {
                return;
            }
            hosts.forEach(redraw);
        });

        hosts.forEach(function (host) {
            kernel.issueAsync("", "source", host.getAttribute("data-shell"))
                .then(function (json) {
                    var reply = JSON.parse(json);
                    if (reply.kind !== "output") {
                        unavailable(host, "its markup from the kernel", reply.text);
                        return;
                    }
                    host.innerHTML = reply.text;
                    var state = stateOf(host);
                    if (host.querySelector("[data-poll]")) {
                        var labeled = host.querySelector("[data-poll-label]");
                        var controls = document.createElement("div");
                        controls.className = "ikigai-view-controls sheet-controls";
                        var toggle = document.createElement("button");
                        toggle.type = "button";
                        toggle.className = "ikigai-run-secondary ikigai-view-toggle";
                        toggle.textContent = labeled
                            ? labeled.getAttribute("data-poll-label") : "Poll";
                        toggle.setAttribute("aria-pressed", "false");
                        var tally = document.createElement("p");
                        tally.className = "ikigai-view-tally";
                        state.tally = tally;
                        tally.textContent = tallyText(state);
                        toggle.addEventListener("click", function () {
                            state.polling = !state.polling;
                            toggle.setAttribute("aria-pressed", state.polling ? "true" : "false");
                            tally.textContent = tallyText(state);
                        });
                        controls.appendChild(toggle);
                        controls.appendChild(tally);
                        host.appendChild(controls);
                    }
                    if (host.hasAttribute("data-cache")) {
                        var out = document.createElement("p");
                        out.className = "sheet-recomputed";
                        out.textContent = "Nothing has changed yet. After an edit, or a poll, " +
                            "this says which values the grid computed again.";
                        host.appendChild(out);
                    }
                    htmx.process(host);
                }).catch(function (err) {
                    unavailable(host, "its markup from the kernel", err);
                });
        });
    }).catch(function (failed) {
        hosts.forEach(function (host) {
            unavailable(host, failed && failed.what ? failed.what : "the in-page kernel",
                failed && failed.what ? failed.err : failed);
        });
    });
})();
