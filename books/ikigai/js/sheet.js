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

        // Draw a sheet's grid again, after the viewer (if it has one) has said what that costs.
        function redraw(host) {
            var grid = host.querySelector(".sheet-view[hx-get]");
            if (!grid) {
                return;
            }
            showRecomputed(host).then(function () {
                htmx.ajax("GET", grid.getAttribute("hx-get"), { source: grid, target: grid });
            });
        }

        document.body.addEventListener("htmx:beforeRequest", function (event) {
            var detail = event.detail;
            var host = detail.elt && detail.elt.closest
                ? detail.elt.closest("div.sheet-play[data-shell]") : null;
            if (!host) {
                return;
            }
            event.preventDefault();
            var config = detail.requestConfig;
            var verb = VERBS[String(config.verb).toLowerCase()];
            var iri = iriOf(config.path);
            var target = detail.target;
            if (!verb || !iri) {
                htmx.swap(target, escape("the page cannot send " + config.verb + " " + config.path),
                    { swapStyle: "innerHTML" });
                return;
            }
            var args = verb === "source" ? [] : fields(config);
            kernel.issueWithArgsAsync("", verb, iri, JSON.stringify(args)).then(function (json) {
                var reply = JSON.parse(json);
                var html = reply.kind === "output" ? reply.text : escape("error: " + reply.text);
                htmx.swap(target, html, { swapStyle: "innerHTML" });
                if (verb !== "source") {
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
                    if (host.hasAttribute("data-cache")) {
                        var out = document.createElement("p");
                        out.className = "sheet-recomputed";
                        out.textContent = "Nothing has changed yet. After an edit, this says " +
                            "which values the grid computed again.";
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
