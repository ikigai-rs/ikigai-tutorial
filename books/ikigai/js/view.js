// A view that polls: htmx markup served as a resource, answered by the book's own kernel.
//
// A chapter writes
//
//     <div class="ikigai-view" data-shell='urn:iki:tutorial:time:template:face'>…a line for
//     readers without the kernel…</div>
//
// and this script fills it with the shell, the resource `data-shell` names, and lets htmx
// run it. The shell carries its own `hx-get` and `hx-trigger` (`every 1s` for the clock), so
// what is asked, and how often, is the markup's business, not this script's.
//
// THE SHIM is `js/ttt.js`'s, cut down: htmx fires `htmx:beforeRequest` before it sends
// anything; for a request from inside a view this cancels it, maps the relative path to a
// name by the book's one rule (`a/b/c` is `urn:a:b:c`; GET is Source, POST Sink, DELETE
// Delete), asks the page's kernel, and hands the answer to `htmx.swap`. The kernel is the
// root's: a view names no game.
//
// POLLING STARTS PAUSED, for two reasons. A page that updates itself every second has to
// offer a way to stop it (WCAG 2.2 SC 2.2.2 Pause, Stop, Hide), and a poll that started on
// load would fill the kernel's cache before the reader runs a single cell, so every cell's
// "computed" would read "cached" (the chapter's cells are written for a page nobody has
// polled yet). A toggle button (`aria-pressed`) starts and stops it. While paused, every
// request htmx makes from inside the view is canceled before it reaches the kernel: htmx
// keeps its timer, and nothing is asked.
//
// THE COUNT is the lesson made visible: every answer to a read says whether the kernel
// served it from its cache or computed it (`issueAsync`'s `cache`, from a probe taken before
// the request), and the view counts them. It is not a live region, since it changes every
// second and a screen reader would repeat it forever; it is text to read when you want it.
//
// Without the kernel (or htmx), a view is one sentence saying what did not load, and
// nothing that looks live. `a11y/no-kernel.mjs` pins that in CI.
//
// ⚠ htmx is loaded the way `js/ttt.js` loads it, and neither shares its load with the
// other: a page with both a board and a view would load htmx twice. No page has both.
(function () {
    "use strict";

    var hosts = document.querySelectorAll("div.ikigai-view[data-shell]");
    if (hosts.length === 0) {
        return;
    }
    var root = typeof path_to_root === "string" ? path_to_root : "./";

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
        note.className = "ikigai-view-unavailable";
        note.textContent = "This view needs " + what + ", which did not load (" + why +
            "), so there is nothing live here. Each runnable cell's “Expected output” " +
            "shows what its request answers.";
        host.appendChild(note);
    }

    function labeled(promise, what) {
        return promise.catch(function (err) {
            throw { what: what, err: err };
        });
    }

    // One view's polling state: paused, and how its reads have been answered so far.
    function freshState() {
        return { polling: false, asked: 0, cached: 0, computed: 0 };
    }

    function tallyText(state) {
        if (state.asked === 0) {
            return state.polling ? "Polling: waiting for the first answer."
                : "Paused: nothing has been asked yet.";
        }
        return (state.polling ? "Polling. " : "Paused. ") + "Asked " + state.asked +
            (state.asked === 1 ? " time: " : " times: ") + state.cached +
            " answered from the cache, " + state.computed + " computed.";
    }

    Promise.all([
        labeled(loadHtmx(), "htmx"),
        labeled(loadKernel(), "the in-page kernel"),
    ]).then(function (loaded) {
        var htmx = loaded[0];
        var kernel = loaded[1];
        htmx.config.allowEval = false;
        htmx.config.historyEnabled = false;

        var states = new Map();

        document.body.addEventListener("htmx:beforeRequest", function (event) {
            var detail = event.detail;
            var host = detail.elt && detail.elt.closest
                ? detail.elt.closest("div.ikigai-view[data-shell]") : null;
            if (!host) {
                return;
            }
            event.preventDefault();
            var state = states.get(host);
            if (!state || !state.polling) {
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
            kernel.issueAsync("", verb, iri).then(function (json) {
                var reply = JSON.parse(json);
                if (verb === "source" && reply.kind === "output") {
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
            }).catch(function (err) {
                htmx.swap(target, escape("error: " + (err && err.message ? err.message : err)),
                    { swapStyle: "innerHTML" });
            });
        });

        hosts.forEach(function (host) {
            kernel.issueAsync("", "source", host.getAttribute("data-shell"))
                .then(function (json) {
                    var reply = JSON.parse(json);
                    if (reply.kind !== "output") {
                        unavailable(host, "its markup from the kernel", reply.text);
                        return;
                    }
                    var state = freshState();
                    host.innerHTML = reply.text;

                    var controls = document.createElement("div");
                    controls.className = "ikigai-view-controls";
                    var toggle = document.createElement("button");
                    toggle.type = "button";
                    toggle.className = "ikigai-run-secondary ikigai-view-toggle";
                    toggle.textContent = "Poll";
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

                    states.set(host, state);
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
