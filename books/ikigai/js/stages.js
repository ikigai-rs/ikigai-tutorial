// The stages viewer: a picture that changes as a chapter's steps run, kept as numbered tabs.
//
// ONE COPY FOR EVERY BOOK, as `js/run.js` is: this file lives in The ikigai Book and each other
// book that uses it links to it (`books/<book>/js/stages.js -> ../../ikigai/js/stages.js`) and
// lists it in its book.toml after `js/run.js`. It knows nothing about what it draws: a page
// names the lines that answer the picture and the words, in markup, and any book's in-page
// kernel answers them. `docs/stages.md` is the contract, with an example.
//
//     <div class="ikigai-stages" data-chain='harness:overlays'
//          data-picture='source urn:diagram:state scheme={scheme}'
//          data-words='source urn:throttle:state'
//          data-alt='What the picture draws, in words.'>
//     <ul class="ikigai-stages-controls" hidden>
//     <li data-cmd='sink urn:iki:tutorial:harness:mode fail'>Set the upstream to fail</li>
//     </ul>
//     <ul class="ikigai-stages-facts">
//     <li data-cmd='source urn:iki:tutorial:harness:mode' data-label='upstream'>upstream: ok</li>
//     </ul>
//     <section class="ikigai-stage" data-label="closed">
//     … the steps that make this stage (cells running in the same chain), prose …
//     <div class="ikigai-stage-result">
//     <p class="ikigai-stage-facts">upstream: ok</p>
//     <p class="ikigai-stage-picture"><a class="ikigai-scheme-light" href="p-1-light.svg"><img …></a><a class="ikigai-scheme-dark" …>…</a></p>
//     <pre class="ikigai-stage-words">what data-words answered at this stage</pre>
//     </div>
//     </section>
//     …
//     </div>
//
// A STAGE is the state after a step that changed the picture or the words. The chapter's stages
// are committed (pictures as files, words as text, a test holding both to what the kernel
// answers), and each `section.ikigai-stage` becomes a numbered tab: the ARIA tabs pattern, with
// Left and Right between tabs, Home and End to the ends, and Tab into the panel. None of that
// needs the kernel, so the stages read before it loads, without it, and in print (every panel).
//
// WHAT CHANGED is computed here, for every stage after the first, by comparing its words with
// the stage before's, line by line: a line that is new, one that is gone, and a line that
// replaced another with the words that differ marked (<mark>, and the old line said beside it,
// so never by color alone). A new line replaced the gone line that begins with the same words
// before its first ": " (where the line is: `layer 1`, a circuit's name); lines left over on
// both sides in equal numbers are paired in order. A control may add its own sentence (`data-note`), and a chapter's
// stage may hold one (`p.ikigai-stage-note`), but the comparison is always there.
//
// A PICTURE IS SHOWN, NEVER REDRAWN. A tab keeps its stage's picture as it was. When the picture
// line holds `{scheme}`, it is asked for twice, `light` and `dark`, and the stylesheet
// (`css/stages.css`) shows the copy the book's theme needs, so a theme change asks nothing.
//
// AFTER EVERY STEP (a control pressed, or a cell in the same chain run: `ikigai:cell-ran` from
// `js/run.js`) the viewer reads the picture, the words and the facts through the kernel, as a
// reader's own lines, and the cache verdict on the picture comes back with it. Then:
//
// - nothing changed (the words and the picture are what the page last saw): "No change", in
//   words, with the picture's verdict (a step that cuts nothing reads it from the cache);
// - a control changed something: a new stage after the chapter's, numbered on, and selected
//   (the controls sit above the tabs, so selecting it hides nothing pressed);
// - a cell changed something to its own stage's words: that stage is the reader's too, and its
//   panel says so; to anything else, a new stage, appended and not selected, since selecting it
//   would hide the cell just run.
//
// The first step compares with the first stage's committed words: nothing is read on load.
//
// ONE PERSISTENT STATUS REGION, polite, above the tabs, says each of those once.
//
// WITHOUT THE KERNEL the tabs and the committed stages still work, there are no controls (the
// list stays hidden), and the status region says the kernel did not load. `a11y/no-kernel.mjs`
// pins that.
(function () {
    "use strict";

    var viewers = document.querySelectorAll(".ikigai-stages[data-picture]");
    if (viewers.length === 0) {
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

    function linesOf(text) {
        return (text || "").split("\n").map(function (l) { return l.trim(); })
            .filter(function (l) { return l.length > 0; });
    }

    // Two answers in words are the same when their lines are, trimmed, blank lines dropped.
    function same(a, b) {
        return linesOf(a).join("\n") === linesOf(b).join("\n");
    }

    // What a run of lines answered, in words: each distinct answer once, with how many lines
    // gave it, so "ten calls, of which one reached the upstream" can be heard.
    function answers(replies) {
        var seen = [];
        var counts = {};
        replies.forEach(function (reply) {
            var said = linesOf(reply.kind === "error" ? "error: " + reply.text : reply.text).join("; ");
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

    // The words that differ between two lines, by a longest common subsequence of their words:
    // returns the new line as [{ text, changed }] runs.
    function wordDiff(before, after) {
        var a = before.split(" ");
        var b = after.split(" ");
        var n = a.length;
        var m = b.length;
        var table = [];
        for (var i = 0; i <= n; i++) {
            table.push(new Array(m + 1).fill(0));
        }
        for (i = n - 1; i >= 0; i--) {
            for (var j = m - 1; j >= 0; j--) {
                table[i][j] = a[i] === b[j] ? table[i + 1][j + 1] + 1
                    : Math.max(table[i + 1][j], table[i][j + 1]);
            }
        }
        var runs = [];
        function push(text, changed) {
            var last = runs[runs.length - 1];
            if (last && last.changed === changed) {
                last.text += " " + text;
            } else {
                runs.push({ text: text, changed: changed });
            }
        }
        i = 0;
        j = 0;
        while (j < m) {
            if (i < n && a[i] === b[j]) {
                push(b[j], false);
                i++;
                j++;
            } else if (i < n && table[i + 1][j] >= table[i][j + 1]) {
                i++;
            } else {
                push(b[j], true);
                j++;
            }
        }
        return runs;
    }

    // "What changed" from one stage's words to the next: a paragraph of real text.
    function changed(before, after) {
        var p = el("p", "ikigai-stage-changed");
        p.appendChild(el("strong", null, "What changed:"));
        if (before === null) {
            p.appendChild(document.createTextNode(" nothing yet: this is the first stage."));
            return p;
        }
        var was = linesOf(before);
        var now = linesOf(after);
        var gone = was.filter(function (l) { return now.indexOf(l) < 0; });
        var added = now.filter(function (l) { return was.indexOf(l) < 0; });
        if (gone.length === 0 && added.length === 0) {
            p.appendChild(document.createTextNode(" the words are the same; only the picture moved."));
            return p;
        }
        var list = el("ul", "ikigai-stage-changes");
        // Which gone line each new line replaced: first the one that says the same place in the
        // same words (the words before the first ": ", `layer 1`, a circuit's name), then, when
        // as many lines are left on each side, the rest in order. Anything else is new or gone.
        var keyOf = function (line) {
            var at = line.indexOf(": ");
            return at > 0 ? line.slice(0, at) : null;
        };
        var replaced = added.map(function () { return null; });
        var left = gone.slice();
        added.forEach(function (line, k) {
            var key = keyOf(line);
            var match = key === null ? -1 : left.findIndex(function (old) { return keyOf(old) === key; });
            if (match >= 0) {
                replaced[k] = left.splice(match, 1)[0];
            }
        });
        var unpaired = replaced.filter(function (old) { return old === null; }).length;
        if (unpaired > 0 && unpaired === left.length) {
            replaced = replaced.map(function (old) { return old !== null ? old : left.shift(); });
        }
        added.forEach(function (line, k) {
            var li = el("li");
            var old = replaced[k];
            li.appendChild(document.createTextNode(old !== null ? "Now: " : "New: "));
            (old !== null ? wordDiff(old, line) : [{ text: line, changed: true }]).forEach(function (run, r) {
                if (r > 0) {
                    li.appendChild(document.createTextNode(" "));
                }
                li.appendChild(run.changed ? el("mark", null, run.text) : document.createTextNode(run.text));
            });
            if (old !== null) {
                // What the marked words replaced: the old line's own changed words.
                var was = wordDiff(line, old).filter(function (run) { return run.changed; })
                    .map(function (run) { return run.text.replace(/[,:;.]+$/, ""); });
                li.appendChild(document.createTextNode(" (was " + (was.length ? was.join(" … ") : "absent") + ")."));
            }
            list.appendChild(li);
        });
        left.forEach(function (line) {
            list.appendChild(el("li", null, "Gone: " + line));
        });
        p.appendChild(list);
        return p;
    }

    // The value a fact shows: line `data-line` (1-based, default 1) of what its line answered,
    // trimmed, blank lines not counted; `data-none` when the answer has no such line.
    function factValue(item, reply) {
        var at = parseInt(item.getAttribute("data-line") || "1", 10) - 1;
        var lines = linesOf(reply.kind === "error" ? "error: " + reply.text : reply.text);
        return lines[at] !== undefined ? lines[at] : (item.getAttribute("data-none") || "");
    }

    function factText(item, value) {
        var label = item.getAttribute("data-label");
        return label ? label + ": " + value : value;
    }

    function install(root, index) {
        var chain = root.getAttribute("data-chain");
        var pictureLine = root.getAttribute("data-picture");
        var wordsLine = root.getAttribute("data-words");
        var altBase = root.getAttribute("data-alt") || "The picture at this stage.";
        var schemes = /\{scheme\}/.test(pictureLine) ? ["light", "dark"] : [null];
        var list = root.querySelector(".ikigai-stages-controls");
        var facts = root.querySelector(".ikigai-stages-facts");
        var factItems = facts ? Array.prototype.slice.call(facts.querySelectorAll("li[data-cmd]")) : [];
        var name = "ikigai-stages-" + (index + 1);

        function evalLine(mod, line) {
            var reply = chain ? mod.evalLineInChainAsync(chain, line) : mod.evalLineAsync(line);
            return reply.then(function (json) { return JSON.parse(json); });
        }

        var status = el("p", "ikigai-stages-status");
        status.setAttribute("role", "status");
        status.setAttribute("aria-live", "polite");

        // ── the tabs ─────────────────────────────────────────────────────────
        var sections = Array.prototype.slice.call(root.querySelectorAll(":scope > section.ikigai-stage"));
        if (sections.length === 0) {
            return;
        }
        var tablist = el("div", "ikigai-stage-tabs");
        tablist.setAttribute("role", "tablist");
        tablist.setAttribute("aria-label", root.getAttribute("data-tabs-label") || "Stages");
        root.insertBefore(status, sections[0]);
        root.insertBefore(tablist, sections[0]);
        var tabs = [];

        function select(tab) {
            tabs.forEach(function (t) {
                var on = t === tab;
                t.setAttribute("aria-selected", on ? "true" : "false");
                t.tabIndex = on ? 0 : -1;
                document.getElementById(t.getAttribute("aria-controls")).hidden = !on;
            });
        }

        function addTab(section) {
            var n = tabs.length + 1;
            var base = name + "-stage-" + n;
            section.setAttribute("data-stage", String(n));
            section.id = section.id || base;
            section.setAttribute("role", "tabpanel");
            section.tabIndex = 0;
            var tab = el("button", "ikigai-stage-tab");
            tab.type = "button";
            tab.id = base + "-tab";
            tab.setAttribute("role", "tab");
            tab.setAttribute("aria-controls", section.id);
            // The visible "2 · open" is inside the accessible name "Stage 2 · open" (SC 2.5.3).
            tab.appendChild(el("span", "ikigai-stage-sr", "Stage "));
            tab.appendChild(document.createTextNode(n + " · " + (section.getAttribute("data-label") || "")));
            section.setAttribute("aria-labelledby", tab.id);
            tab.addEventListener("click", function () {
                select(tab);
            });
            tabs.push(tab);
            tablist.appendChild(tab);
            return tab;
        }

        tablist.addEventListener("keydown", function (event) {
            var at = tabs.indexOf(document.activeElement);
            if (at < 0) {
                return;
            }
            var to = event.key === "ArrowRight" ? (at + 1) % tabs.length
                : event.key === "ArrowLeft" ? (at - 1 + tabs.length) % tabs.length
                : event.key === "Home" ? 0
                : event.key === "End" ? tabs.length - 1
                : -1;
            if (to < 0) {
                return;
            }
            event.preventDefault();
            tabs[to].focus();
            select(tabs[to]);
        });

        function wordsOf(section) {
            var pre = section.querySelector(".ikigai-stage-words");
            return pre ? pre.textContent : "";
        }

        // A chapter's stage: its tab, what made it (the last step above its result, a cell; the
        // page's loading for the first), and what changed since the stage before.
        var previous = null;
        sections.forEach(function (section, k) {
            addTab(section);
            var result = section.querySelector(".ikigai-stage-result");
            if (!result) {
                return;
            }
            var steps = Array.prototype.slice.call(section.querySelectorAll("div.ikigai-run[data-cmd]"))
                .filter(function (cell) {
                    return cell.compareDocumentPosition(result) & Node.DOCUMENT_POSITION_FOLLOWING;
                });
            var made = el("p", "ikigai-stage-made");
            // The first stage is the kernel as the page loads it: a cell in it changes nothing
            // (the native test holds that), so it never made the stage.
            var last = k === 0 ? null : steps[steps.length - 1];
            if (last) {
                made.appendChild(document.createTextNode("Made by "));
                linesOf(last.getAttribute("data-cmd")).forEach(function (line, k) {
                    if (k > 0) {
                        made.appendChild(document.createTextNode(", then "));
                    }
                    made.appendChild(el("code", null, line));
                });
                made.appendChild(document.createTextNode("."));
            } else {
                made.textContent = "Made by loading the page.";
            }
            result.insertBefore(made, result.firstChild);
            result.insertBefore(changed(previous, wordsOf(section)), made.nextSibling);
            previous = wordsOf(section);
        });

        function openHashTarget() {
            var target = location.hash && document.getElementById(decodeURIComponent(location.hash.slice(1)));
            var section = target && target.closest ? target.closest("section.ikigai-stage") : null;
            if (section && root.contains(section)) {
                select(document.getElementById(section.getAttribute("aria-labelledby")));
                target.scrollIntoView();
            }
        }
        select(tabs[0]);
        openHashTarget();
        window.addEventListener("hashchange", openHashTarget);

        // ── where the reader's own kernel stands ─────────────────────────────
        // What the page last saw: the first stage's committed words until the first step.
        var seen = { words: wordsOf(sections[0]), picture: null };

        function note(section, text) {
            var result = section.querySelector(".ikigai-stage-result");
            if (!result) {
                return;
            }
            var p = result.querySelector(".ikigai-stage-reached");
            if (!p) {
                p = el("p", "ikigai-stage-reached");
                result.insertBefore(p, result.querySelector(".ikigai-stage-picture"));
            }
            p.textContent = text;
        }

        // Read the stage as it is now: the picture in each scheme, the words, the facts.
        function read(mod) {
            var stage = { pictures: {}, cache: "", facts: [] };
            var reads = schemes.map(function (scheme) {
                return function () {
                    var line = scheme ? pictureLine.replace(/\{scheme\}/g, scheme) : pictureLine;
                    return evalLine(mod, line).then(function (reply) {
                        if (reply.kind !== "output") {
                            throw new Error("the picture did not draw: " + reply.text);
                        }
                        stage.pictures[scheme || "only"] = reply.text;
                        stage.cache = stage.cache || reply.cache;
                    });
                };
            });
            reads.push(function () {
                return evalLine(mod, wordsLine).then(function (reply) {
                    stage.words = reply.kind === "error" ? "error: " + reply.text : reply.text;
                });
            });
            factItems.forEach(function (item) {
                reads.push(function () {
                    return evalLine(mod, item.getAttribute("data-cmd")).then(function (reply) {
                        stage.facts.push(factText(item, factValue(item, reply)));
                    });
                });
            });
            return reads.reduce(function (done, next) { return done.then(next); }, Promise.resolve())
                .then(function () { return stage; });
        }

        function picture(stage, scheme, alt) {
            var url = URL.createObjectURL(new Blob([stage.pictures[scheme || "only"]], { type: "image/svg+xml" }));
            var link = el("a", scheme ? "ikigai-scheme-" + scheme : null);
            link.href = url;
            var img = el("img");
            img.src = url;
            img.alt = alt;
            link.appendChild(img);
            return link;
        }

        function altOf(words) {
            return altBase + " At this stage: " + linesOf(words).join("; ") + ".";
        }

        function addStage(stage, before, madeBy, sentence) {
            var section = el("section", "ikigai-stage ikigai-stage-yours");
            section.setAttribute("data-label", "yours");
            var n = tabs.length + 1;
            // A heading at the level the chapter's stages use, so the outline stays whole.
            var first = sections[0].querySelector("h2, h3, h4, h5, h6");
            section.appendChild(el(first ? first.tagName.toLowerCase() : "h3", null,
                "Stage " + n + ", yours: " + madeBy.title));
            var result = el("div", "ikigai-stage-result");
            var made = el("p", "ikigai-stage-made");
            made.appendChild(document.createTextNode("Made by "));
            madeBy.parts.forEach(function (part, k) {
                if (k > 0) {
                    made.appendChild(document.createTextNode(", then "));
                }
                made.appendChild(part);
            });
            made.appendChild(document.createTextNode("."));
            result.appendChild(made);
            var what = changed(before, stage.words);
            result.appendChild(what);
            if (sentence) {
                result.appendChild(el("p", "ikigai-stage-note", sentence));
            }
            if (stage.facts.length) {
                result.appendChild(el("p", "ikigai-stage-facts", stage.facts.join(" · ")));
            }
            var pictures = el("p", "ikigai-stage-picture");
            schemes.forEach(function (scheme) {
                pictures.appendChild(picture(stage, scheme, altOf(stage.words)));
            });
            result.appendChild(pictures);
            result.appendChild(el("pre", "ikigai-stage-words", stage.words));
            section.appendChild(result);
            section.hidden = true;
            root.appendChild(section);
            var said = Array.prototype.map.call(what.querySelectorAll("li"), function (li) {
                return li.textContent;
            }).join(" ") || what.textContent.replace(/^What changed:\s*/, "");
            return { n: n, tab: addTab(section), what: said };
        }

        // After a step: read the stage, keep the facts current, and say what it made. The step
        // is compared with what the page last saw, which is where the reader's kernel was.
        function after(step) {
            return loadKernel().then(read).then(function (stage) {
                if (facts && stage.facts.length) {
                    factItems.forEach(function (item, k) {
                        item.textContent = stage.facts[k];
                    });
                }
                var before = seen.words;
                var light = stage.pictures[schemes[0] || "only"];
                var moved = !same(stage.words, seen.words) || (seen.picture !== null && light !== seen.picture);
                seen = { words: stage.words, picture: light };
                if (!moved) {
                    status.textContent = step.lead + "No change: this step moved nothing the picture " +
                        "shows, so it makes no stage (" + (stage.cache === "cached"
                            ? "the picture came from the cache" :
                            "the picture was " + stage.cache + ": the first time this page asked for it") + ").";
                    return;
                }
                var own = step.section;
                if (own && !own.classList.contains("ikigai-stage-yours") && same(wordsOf(own), stage.words)) {
                    note(own, "Your kernel reached this stage too.");
                    status.textContent = step.lead + "Your kernel now matches stage " +
                        own.getAttribute("data-stage") + ".";
                    return;
                }
                var made = addStage(stage, before, step.madeBy, step.sentence);
                if (!own) {
                    select(made.tab);
                } else {
                    note(own, "Your kernel made stage " + made.n + " instead, which is not the one " +
                        "this tab shows: its tab is " + made.n + ".");
                }
                status.textContent = step.lead + "New stage " + made.n + ": " + made.what +
                    (step.sentence ? " " + step.sentence : "");
            }).catch(function (err) {
                status.textContent = step.lead + "The stage could not be read (" + message(err) + ").";
            });
        }

        // ── the controls ─────────────────────────────────────────────────────
        var busy = false;
        var buttons = [];

        function press(label, lines, sentence) {
            if (busy) {
                return Promise.resolve();
            }
            busy = true;
            buttons.forEach(function (b) { b.disabled = true; });
            status.textContent = label + ": running…";
            return loadKernel().then(function (mod) {
                var replies = [];
                return lines.reduce(function (done, l) {
                    return done.then(function () {
                        return evalLine(mod, l).then(function (reply) { replies.push(reply); });
                    });
                }, Promise.resolve()).then(function () {
                    return after({
                        lead: label + ": " + answers(replies) + ". ",
                        section: null,
                        sentence: sentence,
                        madeBy: { title: "after “" + label + "”", parts: [el("span", null, "the button “" + label + "”")] },
                    });
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
                status.textContent = "The kernel in this page is ready: a step that changes the " +
                    "picture adds a tab.";
                return;
            }
            var row = el("div", "ikigai-stages-buttons");
            row.setAttribute("role", "group");
            row.setAttribute("aria-label", root.getAttribute("data-controls-label") || "Steps");
            Array.prototype.slice.call(list.querySelectorAll("li[data-cmd]")).forEach(function (item) {
                var label = item.textContent.trim();
                var lines = linesOf(item.getAttribute("data-cmd"));
                // The visible text IS the button's accessible name (SC 2.5.3 Label in Name).
                var button = el("button", "ikigai-run-secondary", label);
                button.type = "button";
                button.addEventListener("click", function () {
                    press(label, lines, item.getAttribute("data-note"));
                });
                buttons.push(button);
                row.appendChild(button);
            });
            // The buttons ARE the list now: the hidden markup they were built from goes.
            list.parentNode.replaceChild(row, list);
            status.textContent = "The kernel in this page is ready: each button runs its step " +
                "there, and a step that changes the picture adds a tab.";
        }).catch(function (err) {
            status.textContent = "The in-page kernel did not load (" + message(err) + "), so " +
                "nothing can be stepped: the tabs show the chapter's stages as they were drawn.";
        });

        document.addEventListener("ikigai:cell-ran", function (event) {
            var detail = event.detail || {};
            if ((detail.chain || null) !== (chain || null) || !detail.cell) {
                return;
            }
            // A cell in the same chain is a step wherever it is; one inside a stage is that
            // stage's step.
            var section = detail.cell.closest("section.ikigai-stage");
            if (section && !root.contains(section)) {
                section = null;
            }
            var parts = (detail.lines || []).map(function (l) { return el("code", null, l); });
            var where = section ? "in stage " + section.getAttribute("data-stage") : "outside the stages";
            // The reads below are reads of the same kernel the next cell runs in: where run.js
            // offers it (`hold`), the chain's next run waits for them, so a cut in that cell
            // cannot land between them.
            var reading = after({
                lead: "After the cell you ran " + where + ": ",
                section: section,
                sentence: null,
                madeBy: {
                    title: "after a cell " + where,
                    parts: parts.length ? parts : [el("span", null, "a cell")],
                },
            });
            if (typeof detail.hold === "function") {
                detail.hold(reading);
            }
        });
    }

    for (var i = 0; i < viewers.length; i++) {
        install(viewers[i], i);
    }
})();
