// The playable tic-tac-toe board: htmx markup, answered by the book's own kernel.
//
// A chapter writes
//
//     <div class="ttt-play" data-game='a'>…a line for readers without the kernel…</div>
//
// and this script fills it with the game's page shell — the resource
// `urn:iki:tutorial:ttt:template:game`, the same bytes `ttt-host` serves over HTTP — and
// lets htmx run it. Nothing here renders a board: the markup is the game's own resources
// (`view:board`, `view:status`), and htmx asks for them by the relative paths in that
// markup (`iki/tutorial/ttt/view/board`), exactly as it would of a server.
//
// THE SHIM. There is no server. htmx fires `htmx:beforeRequest` before it sends anything;
// for a request from inside a board this cancels it, maps it to the kernel, and swaps the
// answer in with `htmx.swap` — the function htmx's own response handling calls, so
// `htmx:afterSwap` / `htmx:afterSettle` fire as they would for a real response, and the
// markup's own triggers (the board refreshes when the status settles) work unchanged.
//
//   the request         →  the kernel
//   GET  <path>         →  Source urn:<path with / as :>
//   POST <path>         →  Sink   urn:<path with / as :>
//   DELETE <path>       →  Delete urn:<path with / as :>
//
// Only relative paths are answered (`iki/tutorial/ttt/view/board`, never `/…` or a URL):
// the path ↔ IRI rule is the one every host of this markup follows, and a path is relative
// so the host decides the game by where the page is. Here, that is the board's own markup:
// the game comes from `data-game` on the `.ttt-play` it sits in — never from the request —
// and goes to `issueAsync` as the chain to resolve in, exactly as a cell's `data-game` does.
//
// Why cancel-and-swap rather than a custom htmx extension or a fake XMLHttpRequest: an
// extension has no hook to replace the transport, and a fake XHR has to impersonate a dozen
// properties htmx reads. `htmx:beforeRequest` is the documented place to stop a request,
// and `htmx.swap` is the documented way to put content where htmx would have. The one thing
// this does NOT reproduce is `hx-swap`: every swap here is `innerHTML`, which is the only
// swap the game's markup uses.
//
// IDS ARE NAMESPACED PER GAME. The squares carry ids (`ttt-square-1-1`) so that htmx can put
// focus back on the square you pressed after the board is re-rendered — its focus
// restoration is by id. A page with two boards would then have every id twice, so the shim
// prefixes each id in an answer with the game (`a-ttt-square-1-1`). A host that serves one
// board per page (ttt-host) has no need to.
//
// HTMX IS VENDORED, not fetched from a CDN: `src/vendor/htmx-2.0.4.min.js`, byte for byte
// the copy `ikigai-web/assets/htmx.min.js` serves, sha256
// e209dda5c8235479f3166defc7750e1dbcd5a5c1808b7792fc2e6733768fb447, license 0BSD
// (https://github.com/bigskysoftware/htmx). It is loaded only on a page that has a board.
(function () {
    "use strict";

    var boards = document.querySelectorAll("div.ttt-play[data-game]");
    if (boards.length === 0) {
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

    // run.js's loader when it is on the page (it always is, before this script), so the
    // board and the cells share one wasm instance and one kernel.
    function loadKernel() {
        if (typeof window.ikigaiBookKernel === "function") {
            return window.ikigaiBookKernel();
        }
        return import(root + "wasm/book_wasm.js").then(function (mod) {
            return mod.default({ module_or_path: root + "wasm/book_wasm_bg.wasm" })
                .then(function () { return mod; });
        });
    }

    function escape(text) {
        return String(text).replace(/[&<>"']/g, function (c) {
            return { "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" }[c];
        });
    }

    // The template slot format, as every host of this markup fills it (the tic-tac-toe
    // crate's README states it, with the cases a filler must refuse): every `{{` opens a
    // slot that ends at the first `}}`, and what is between must be `{{name}}` or
    // `{{name 0 -2}}` — a lower-case name, then integers in their one plain spelling, one
    // space before each — or the template is refused. A value is text (escaped as it goes
    // in) unless the caller hands back `{ html: … }`. (Integers past 2^53 are refused here
    // and not in Rust; no template comes near.)
    var SLOT = /^([a-z][a-z-]*)((?: (?:0|-?[1-9][0-9]*))*)$/;
    var INSIDE = /\{\{([\s\S]*?)\}\}/g;
    function fill(template, value) {
        if (template.split(INSIDE).some(function (text, i) { return i % 2 === 0 && text.indexOf("{{") !== -1; })) {
            throw new Error("a template: a `{{` is never closed");
        }
        return template.replace(INSIDE, function (_, inside) {
            var slot = SLOT.exec(inside);
            var args = slot && slot[2] ? slot[2].trim().split(" ").map(Number) : [];
            if (!slot || !args.every(Number.isSafeInteger)) {
                throw new Error("a template: `{{" + inside + "}}` is not a slot");
            }
            var filled = value(slot[1], args);
            return typeof filled === "object" ? filled.html : escape(filled);
        });
    }

    // The path ↔ IRI rule: a relative path's segments, joined by `:`, after `urn:`.
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

    function namespaced(html, game) {
        return html.replace(/(\s)id="/g, "$1id=\"" + game + "-");
    }

    // WITHOUT THE KERNEL (or htmx), A BOARD IS A SENTENCE, NOT A GRID. Everything a board
    // shows is the kernel's answer to a request — the squares, whose turn it is, who won —
    // so with nothing to answer, the board is replaced by one short message: no squares, no
    // status, no state. A half-drawn grid would look playable and answer nothing; the cells'
    // own "Expected output" disclosures are where a reader without the kernel can see what
    // each request returns. `a11y/no-kernel.mjs` pins this in CI. With the kernel, the
    // board's first render is the game's UI, not an answer to a cell, and is unchanged.
    function unavailable(board, what, err) {
        var why = err && err.message ? err.message : String(err);
        board.textContent = "";
        var note = document.createElement("p");
        note.className = "ttt-unavailable";
        note.textContent = "Game " + board.getAttribute("data-game") + "'s board needs " + what +
            ", which did not load (" + why + "), so there is nothing to play here. Each " +
            "runnable cell's \u201cExpected output\u201d shows what its request answers.";
        board.appendChild(note);
    }

    // Each load says which half failed, so the message names the right one.
    function labeled(promise, what) {
        return promise.catch(function (err) {
            throw { what: what, err: err };
        });
    }

    Promise.all([
        labeled(loadHtmx(), "htmx"),
        labeled(loadKernel(), "the in-page kernel"),
    ]).then(function (loaded) {
        var htmx = loaded[0];
        var kernel = loaded[1];
        htmx.config.allowEval = false;
        htmx.config.historyEnabled = false;

        document.body.addEventListener("htmx:beforeRequest", function (event) {
            var detail = event.detail;
            var board = detail.elt && detail.elt.closest
                ? detail.elt.closest("div.ttt-play[data-game]") : null;
            if (!board) {
                return;
            }
            event.preventDefault();
            var game = board.getAttribute("data-game");
            var config = detail.requestConfig;
            var verb = VERBS[String(config.verb).toLowerCase()];
            var iri = iriOf(config.path);
            var target = detail.target;
            if (!verb || !iri) {
                htmx.swap(target, escape("the page cannot send " + config.verb + " " + config.path),
                    { swapStyle: "innerHTML" });
                return;
            }
            kernel.issueAsync(game, verb, iri).then(function (json) {
                var reply = JSON.parse(json);
                var html = reply.kind === "output" ? reply.text : escape("error: " + reply.text);
                htmx.swap(target, namespaced(html, game), { swapStyle: "innerHTML" });
            }).catch(function (err) {
                htmx.swap(target, escape("error: " + (err && err.message ? err.message : err)),
                    { swapStyle: "innerHTML" });
            });
        });

        // THE PAGE IS A COPY. A cell that plays in a board's game changes the kernel's
        // state, and the kernel cuts every cached answer that depended on it — but the
        // board on screen is HTML the kernel handed out earlier, and nothing cuts that. So
        // when a cell has run in a board's game, ask again: re-fetch the status the way its
        // own markup does (its `hx-get`), and the board follows it as it always does.
        document.addEventListener("ikigai:cell-ran", function (event) {
            var game = event.detail && event.detail.game;
            boards.forEach(function (board) {
                var status = board.querySelector(".ttt-status[hx-get]");
                if (game && board.getAttribute("data-game") === game && status) {
                    htmx.ajax("GET", status.getAttribute("hx-get"),
                        { source: status, target: status });
                }
            });
        });

        boards.forEach(function (board) {
            var game = board.getAttribute("data-game");
            kernel.issueAsync(game, "source", "urn:iki:tutorial:ttt:template:game")
                .then(function (json) {
                    var reply = JSON.parse(json);
                    if (reply.kind !== "output") {
                        unavailable(board, "its page shell from the kernel", reply.text);
                        return;
                    }
                    board.innerHTML = fill(reply.text, function (name) {
                        return name === "game" ? game : "";
                    });
                    htmx.process(board);
                }).catch(function (err) {
                    unavailable(board, "its page shell from the kernel", err);
                });
        });
    }).catch(function (failed) {
        boards.forEach(function (board) {
            unavailable(board, failed && failed.what ? failed.what : "the in-page kernel",
                failed && failed.what ? failed.err : failed);
        });
    });
})();
