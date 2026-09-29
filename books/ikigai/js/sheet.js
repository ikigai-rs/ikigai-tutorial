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
// THE CACHE VIEWER (`data-cache` on the host). Before the grid is drawn again, the page reads
// one more resource, `iki/tutorial/sheet/view/cost`, and puts it under the sheet: a table the
// shape of the grid saying, cell by cell, whether the redraw computes that value or the cache
// serves it. The table is a composition over the kernel's own probe (`urn:kernel:cached`, a
// question that resolves nothing), so this script only asks for it, and asks FIRST, since the
// redraw is what computes the values again. It is not a live region: the reply above it is the
// status that is announced, and this is there to read when you want it. (Until core 0.1.82
// this was the page's own code, probing names the kernel handed it; spreadsheet-3.md says why.)
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
// A SHEET THAT LISTENS (`data-push` on the host, part VII). Its shell draws each cell as a view
// of its own (`iki/tutorial/sheet/view/cell/A1`, …), and each cell's trigger is
// `load, stale, every 2s`: drawn once when the page loads, drawn again when the page says it
// is stale, and asked again every two seconds only while polling is on. Three toggles, all
// off to start (WCAG 2.2 SC 2.2.2): the market, polling, and PUSH. Push registers a cut
// listener (`sink urn:iki:tutorial:push:listener:page0 urn:iki:tutorial:sheet:`, a resource,
// refused like any resource when the page's capability does not grant `urn:cap:kernel:listen`)
// and then waits on it (`pushWaitAsync`). The kernel never calls the page: a cut appends to the
// listener's queue and wakes the wait, and the page, on its own turn, triggers `stale` on
// exactly the cells whose last answer carried the thread that was cut (`book-wasm` remembers
// what each answer carried: every cell of a listening sheet is drawn through `drawAsync`). And
// since TIME DOES NOT CUT, the page keeps one timer, at the earliest deadline among the cells
// it drew (`pushDeadline`), and draws again whatever has expired when it fires
// (`pushExpired`). A listening sheet never redraws itself after an edit or a cell: a write is
// a write, whoever made it, and the listener hears it. With push and polling both off, the
// grid is the copy the page drew when it loaded, and the tally says so.
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
    var COST = "urn:iki:tutorial:sheet:view:cost";

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

        // Show what a redraw is about to cost: the cost view, read before the redraw.
        function showRecomputed(host) {
            var out = host.querySelector(".sheet-recomputed");
            if (!out) {
                return Promise.resolve();
            }
            return kernel.issueAsync("", "source", COST).then(function (json) {
                var reply = JSON.parse(json);
                if (reply.kind !== "output") {
                    out.textContent = "The page could not ask what the redraw costs: " + reply.text;
                    return;
                }
                out.innerHTML = reply.text;
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

        // ANCHOR: push_host
        // A sheet that listens: its listener's name on this page, and the prefix it listens
        // under (every name the sheet has).
        var PREFIX = "urn:iki:tutorial:sheet:";
        var LISTENER = "urn:iki:tutorial:push:listener:";

        // The cells of `host` that show `views` (names like urn:iki:tutorial:sheet:view:cell:A1),
        // found by the path each cell asks for.
        function cellsShowing(host, views) {
            var found = [];
            views.forEach(function (view) {
                var path = view.replace(/^urn:/, "").split(":").join("/");
                host.querySelectorAll('td[hx-get="' + path + '"]').forEach(function (td) {
                    found.push(td);
                });
            });
            return found;
        }

        function cellName(td) {
            return td.getAttribute("hx-get").split("/").pop();
        }

        // Draw `cells` again, and say why: htmx asks for each one (its `stale` trigger), and the
        // shim below lets the request through because the page asked for it.
        function drawAgain(host, state, cells, why) {
            state.marked.forEach(function (td) {
                td.classList.remove("sheet-pushed");
            });
            state.marked = cells;
            cells.forEach(function (td) {
                state.asked.set(td, why);
                td.classList.add("sheet-pushed");
                htmx.trigger(td, "stale");
            });
            state.redrew += cells.length;
        }

        // Wait for the next cuts, draw the cells they make stale, and wait again, for as long
        // as push is on and this is still the listener it started.
        function listen(host, state, generation) {
            kernel.pushWaitAsync(state.listener).then(function (json) {
                if (!state.push || generation !== state.generation) {
                    return;
                }
                var batch = JSON.parse(json);
                if (batch.error) {
                    stopPush(host, state, "Push stopped: " + batch.error + ".");
                    return;
                }
                state.heard += batch.cuts.length;
                state.dropped += batch.dropped;
                var cells = cellsShowing(host, batch.stale);
                drawAgain(host, state, cells, "push");
                var cut = batch.cuts.map(function (thread) {
                    return thread.replace(PREFIX, "");
                }).join(", ");
                state.last = batch.dropped > 0
                    ? "The page fell behind: " + batch.dropped + " cuts were dropped, so it drew " +
                        "every cell again."
                    : "Last: " + (batch.cuts.length === 1 ? "a cut of " : "cuts of ") + cut +
                        (cells.length ? ", which drew " + cells.map(cellName).join(", ") + " again."
                            : ", which no cell on the grid rests on.");
                showPush(state);
                listen(host, state, generation);
            });
        }

        // One timer, at the earliest deadline among the cells drawn: an expiry is not a cut,
        // so nobody announces it, and the page has to know when to look.
        function schedule(host, state) {
            clearTimeout(state.timer);
            state.timer = null;
            state.deadline = state.push ? kernel.pushDeadline() : -1;
            if (state.deadline >= 0) {
                state.timer = setTimeout(function () {
                    state.timer = null;
                    var cells = cellsShowing(host, JSON.parse(kernel.pushExpired()));
                    if (cells.length === 0) {
                        schedule(host, state);
                        return;
                    }
                    state.expired += cells.length;
                    drawAgain(host, state, cells, "expired");
                    state.last = "Last: the deadline " + clockText(state.deadline) +
                        " passed, which drew " + cells.map(cellName).join(", ") + " again.";
                    showPush(state);
                }, Math.max(0, state.deadline - Date.now()) + 5);
            }
            showPush(state);
        }

        function startPush(host, state) {
            kernel.issueWithArgsAsync("", "sink", LISTENER + state.listener,
                JSON.stringify([["content", PREFIX]])).then(function (json) {
                var reply = JSON.parse(json);
                if (reply.kind !== "output") {
                    stopPush(host, state, "Push was refused: " + reply.text);
                    return;
                }
                state.push = true;
                state.generation += 1;
                state.pushToggle.setAttribute("aria-pressed", "true");
                listen(host, state, state.generation);
                schedule(host, state);
            });
        }

        function stopPush(host, state, why) {
            var was = state.push;
            state.push = false;
            state.generation += 1;
            state.pushToggle.setAttribute("aria-pressed", "false");
            state.last = why || "";
            schedule(host, state);
            if (was) {
                kernel.issueAsync("", "delete", LISTENER + state.listener);
            }
        }
        // ANCHOR_END: push_host

        function clockText(millis) {
            return new Date(millis).toISOString().slice(11, 19) + "Z";
        }

        function showPush(state) {
            var text;
            if (!state.push) {
                text = state.last ? state.last + " " : "";
                text += "Not pushing: nothing tells the grid about a change.";
            } else {
                text = "Pushing: listening for cuts under " + PREFIX + ". ";
                text += state.heard === 0 && state.expired === 0 ? "Nothing has been cut yet."
                    : "Heard " + state.heard + (state.heard === 1 ? " cut" : " cuts") + "; drew " +
                        state.redrew + (state.redrew === 1 ? " cell" : " cells") +
                        " again. " + state.last;
                if (state.deadline >= 0) {
                    text += " The next deadline: " + clockText(state.deadline) + ".";
                }
            }
            state.pushTally.textContent = text;
        }

        function showPoll(state) {
            var polls = state.polls;
            state.pollTally.textContent = polls.asked === 0
                ? (state.polling ? "Polling: waiting for the first answer."
                    : "Not polling: the cells have not been asked again.")
                : (state.polling ? "Polling. " : "Not polling. ") + "The cells were asked " +
                    polls.asked + (polls.asked === 1 ? " time: " : " times: ") + polls.cached +
                    " answered from the cache, " + polls.computed + " computed.";
        }

        // A request from inside a sheet that listens. A cell's GET is drawn through `drawAsync`,
        // so the page remembers what the answer carried: on load, when the page said it was
        // stale, and on a poll (only while polling is on). The market's ticker runs only while
        // the market does. Everything else (the formula bar, the market's button) goes straight
        // to the kernel, and the grid is NOT drawn again after it: the listener hears the write.
        function pushRequest(host, state, detail) {
            var elt = detail.elt;
            var config = detail.requestConfig;
            var verb = VERBS[String(config.verb).toLowerCase()];
            var iri = iriOf(config.path);
            var target = detail.target;
            if (!verb || !iri) {
                htmx.swap(target, escape("the page cannot send " + config.verb + " " + config.path),
                    { swapStyle: "innerHTML" });
                return;
            }
            if (elt.matches && elt.matches("td[hx-get]")) {
                var why = state.asked.get(elt);
                state.asked.delete(elt);
                var poll = !why && state.drawn.has(elt);
                if (poll && !state.polling) {
                    return;
                }
                state.drawn.add(elt);
                kernel.drawAsync(iri).then(function (json) {
                    var reply = JSON.parse(json);
                    if (poll && reply.kind === "output") {
                        state.polls.asked += 1;
                        state.polls[reply.cache === "cached" ? "cached" : "computed"] += 1;
                        showPoll(state);
                    }
                    htmx.swap(target, reply.kind === "output" ? reply.text
                        : escape("error: " + reply.text), { swapStyle: "innerHTML" });
                    if (state.push) {
                        schedule(host, state);
                    }
                });
                return;
            }
            if (elt.hasAttribute("data-market") && !state.market) {
                return;
            }
            var args = verb === "source" ? [] : fields(config);
            kernel.issueWithArgsAsync("", verb, iri, JSON.stringify(args)).then(function (json) {
                var reply = JSON.parse(json);
                htmx.swap(target, reply.kind === "output" ? reply.text
                    : escape("error: " + reply.text), { swapStyle: "innerHTML" });
            });
        }

        // The three toggles of a sheet that listens, and the two lines that say what each
        // costs. Not live regions: they change on every cut and every poll.
        function pushControls(host, state) {
            state.listener = "page" + Array.prototype.indexOf.call(hosts, host);
            state.push = false;
            state.market = false;
            state.generation = 0;
            state.heard = 0;
            state.dropped = 0;
            state.redrew = 0;
            state.expired = 0;
            state.deadline = -1;
            state.timer = null;
            state.last = "";
            state.marked = [];
            state.asked = new Map();
            state.drawn = new WeakSet();
            state.polls = { asked: 0, cached: 0, computed: 0 };

            var controls = document.createElement("div");
            controls.className = "ikigai-view-controls sheet-controls";
            function toggle(label, onClick) {
                var button = document.createElement("button");
                button.type = "button";
                button.className = "ikigai-run-secondary ikigai-view-toggle";
                button.textContent = label;
                button.setAttribute("aria-pressed", "false");
                button.addEventListener("click", onClick);
                controls.appendChild(button);
                return button;
            }
            var market = toggle("Run the market", function () {
                state.market = !state.market;
                market.setAttribute("aria-pressed", state.market ? "true" : "false");
            });
            state.pushToggle = toggle("Push", function () {
                if (state.push) {
                    stopPush(host, state);
                } else {
                    startPush(host, state);
                }
            });
            var poll = toggle("Poll", function () {
                state.polling = !state.polling;
                poll.setAttribute("aria-pressed", state.polling ? "true" : "false");
                showPoll(state);
            });
            state.pushTally = document.createElement("p");
            state.pushTally.className = "ikigai-view-tally";
            state.pollTally = document.createElement("p");
            state.pollTally.className = "ikigai-view-tally";
            host.appendChild(controls);
            host.appendChild(state.pushTally);
            host.appendChild(state.pollTally);
            showPush(state);
            showPoll(state);
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
            if (host.hasAttribute("data-push")) {
                pushRequest(host, state, detail);
                return;
            }
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

        // A runnable cell with no chain ran in the root's, which is the sheet's. A cell in a
        // game or a scenario changes nothing the shared sheet shows.
        document.addEventListener("ikigai:cell-ran", function (event) {
            if (event.detail && (event.detail.game || event.detail.chain)) {
                return;
            }
            // A sheet that listens hears the cell's writes itself, or, while push is off, is
            // meant not to know about them.
            hosts.forEach(function (host) {
                if (!host.hasAttribute("data-push")) {
                    redraw(host);
                }
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
                    host.innerHTML = reply.text;
                    var state = stateOf(host);
                    if (host.hasAttribute("data-push")) {
                        pushControls(host, state);
                    } else if (host.querySelector("[data-poll]")) {
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
                        var out = document.createElement("div");
                        out.className = "sheet-recomputed";
                        // Like the grid's region: it may scroll sideways on a narrow page, so
                        // it takes focus and scrolls by keyboard too.
                        out.setAttribute("role", "region");
                        out.setAttribute("aria-label", "What drawing the grid again cost");
                        out.setAttribute("tabindex", "0");
                        out.textContent = "Nothing has changed yet. After an edit, or a poll, " +
                            "this shows which values drawing the grid again computed.";
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
