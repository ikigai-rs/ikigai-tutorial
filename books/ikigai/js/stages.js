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
// stage may hold one (`p.ikigai-stage-note`), but the comparison is always there. The same
// comparison runs between ANY two stages the reader picks to compare (below).
//
// A PICTURE IS SHOWN, NEVER REDRAWN. A tab keeps its stage's picture as it was. When the picture
// line holds `{scheme}`, it is asked for twice, `light` and `dark`, and the stylesheet
// (`css/stages.css`) shows the copy the book's theme needs, so a theme change asks nothing.
//
// A PICTURE MAY MARK WHAT CHANGED. With `data-before`, that line is read after every step too,
// and its answer goes to the next picture read as `before="…"`, so the picture marks what changed
// since (ikigai-diagram's `urn:diagram:state`); a stage is then judged by the words and that
// answer, since a marked picture differs whenever the board does.
//
// AFTER EVERY STEP (a control pressed, or a cell in the same chain run: `ikigai:cell-ran` from
// `js/run.js`) the viewer reads the picture, the words and the facts through the kernel, as a
// reader's own lines, and the cache verdict on the picture comes back with it. Then:
//
// - nothing changed (the words and the picture are what the page last saw): "No change", in
//   words, with the picture's verdict (a step that cuts nothing reads it from the cache);
// - a control changed something: a new stage after the chapter's, numbered on, and selected;
// - a cell changed something to its own stage's words: the reader's kernel is at that stage;
//   to anything else, a new stage of the reader's, and the cell's own tab says where it went.
//
// The first step compares with the first stage's committed words: nothing is read on load.
//
// BEFORE | AFTER, BELOW THE TABS (Brian, 2026-10-26: the fixed view he chose, so that nothing
// moves but what the reader asked to see). Between the tabs and the open tab's panel
// (`div.ikigai-stages-compare`, a labeled region) are, top to bottom:
//
// - "Compare with": one native <select> with its visible label (ledger #2298: a row of buttons,
//   one per stage, under a row of tabs, read as two near-identical rows), offering the stages
//   before the open tab, numbered and named as the tabs are. It starts at the stage just before,
//   and opening a tab puts it back there. On the first tab a sentence says there is nothing
//   before it, in its place.
// - Two pictures side by side (stacked on a narrow screen, Before on top): Before is the stage
//   picked, After the open tab's. Each is the picture the reader's kernel drew for that stage if
//   its tab's step ran in this page, else the chapter's committed picture, and its caption says
//   which. The chapter's is in a dashed frame, with a note on what running the step does.
// - What changed from the one to the other: the comparison above, for that pair.
//
// The panel below holds the tab's step: its lead, its cells and its prose, as the chapter wrote
// them. Switching tabs or "Compare with" asks the kernel nothing: every picture shown is one a
// step already drew, and the chapter's are files. Until the kernel loads (and without it, and in
// print) each tab shows its own committed picture instead, inside the tab.
//
// ONE PERSISTENT STATUS REGION, polite, above the tabs, says each of those once, and which stage
// Before is when the reader picks one to compare with.
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

    // "What changed" from one stage's words to another's: real text, led by `lead` (default
    // "What changed:"). A block, since it may hold a list.
    function changed(before, after, lead) {
        var p = el("div", "ikigai-stage-changed");
        p.appendChild(el("strong", null, lead || "What changed:"));
        if (before === null) {
            p.appendChild(document.createTextNode(" nothing yet: this is the first stage."));
            return p;
        }
        var was = linesOf(before);
        var now = linesOf(after);
        var gone = was.filter(function (l) { return now.indexOf(l) < 0; });
        var added = now.filter(function (l) { return was.indexOf(l) < 0; });
        if (gone.length === 0 && added.length === 0) {
            p.appendChild(document.createTextNode(lead ? " the words are the same; only the picture differs."
                : " the words are the same; only the picture moved."));
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
        var beforeLine = root.getAttribute("data-before");
        // The kernel's state above the tabs: a line redrawn after every step, into a figure the
        // chapter names (`data-overview-target`), in place of the picture it committed.
        var overviewLine = root.getAttribute("data-overview");
        var overview = overviewLine ? document.getElementById(root.getAttribute("data-overview-target") || "") : null;
        var overviewSchemes = overviewLine && /\{scheme\}/.test(overviewLine) ? ["light", "dark"] : [null];
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

        // ── Before | After, between the tabs and the open panel ──────────────
        // Put in the page only once the kernel loads: until then each tab's own committed picture
        // is the one to read (and in print, always), and a page without a kernel offers no
        // control at all (a11y/no-kernel.mjs).
        var compare = el("div", "ikigai-stages-compare");
        compare.setAttribute("role", "region");
        compare.setAttribute("aria-label", root.getAttribute("data-compare-label") || "The open stage, beside an earlier one");
        compare.hidden = true;
        var againstRow = el("div", "ikigai-stages-against");
        var againstLabel = el("label", "ikigai-stages-against-label", "Compare with");
        var againstSelect = el("select", "ikigai-stages-against-select");
        againstSelect.id = name + "-against";
        againstLabel.htmlFor = againstSelect.id;
        againstRow.appendChild(againstLabel);
        againstRow.appendChild(againstSelect);
        var againstNote = el("p", "ikigai-stages-against-note",
            "Stage 1 is where it starts: there is nothing before it to compare it with.");
        var pair = el("div", "ikigai-stages-pair");
        var diff = el("div", "ikigai-stages-diff");
        compare.appendChild(againstRow);
        compare.appendChild(againstNote);
        compare.appendChild(pair);
        compare.appendChild(diff);
        // The stage to compare the open one with (an index into `tabs`), or null: the one before.
        var against = null;

        function slot(kind) {
            var figure = el("figure", "ikigai-stages-slot ikigai-stages-" + kind);
            var caption = el("figcaption", "ikigai-stages-slot-caption");
            var holder = el("div", "ikigai-stages-slot-picture");
            var note = el("p", "ikigai-stages-slot-note");
            figure.appendChild(caption);
            figure.appendChild(holder);
            figure.appendChild(note);
            pair.appendChild(figure);
            return { figure: figure, caption: caption, holder: holder, note: note };
        }
        var beforeSlot = slot("before");
        var afterSlot = slot("after");

        // "Compare with" offers the stages before tab `k`, named as the tabs are ("Stage 2 · open",
        // a reader's own stages among them), with stage `c` picked. Tabs are only ever added at the
        // end, so the options are made again only when the open tab has a different number of
        // stages before it, never while the reader picks one.
        function syncAgainst(k, c) {
            if (againstSelect.options.length !== k) {
                againstSelect.textContent = "";
                for (var i = 0; i < k; i++) {
                    var option = el("option", null, "Stage " + labelOf(sectionOf(tabs[i])));
                    option.value = String(i);
                    againstSelect.appendChild(option);
                }
            }
            againstSelect.selectedIndex = c;
        }

        // A stage picked: Before is that stage, What changed starts from it, and the status
        // region says so. The kernel is asked nothing.
        againstSelect.addEventListener("change", function () {
            against = parseInt(againstSelect.value, 10);
            render();
            status.textContent = "Compare with: Before is now stage " + labelOf(sectionOf(tabs[against])) +
                ", beside stage " + labelOf(sectionOf(selectedTab())) + " as After.";
        });

        // Open a tab. `with` is the tab to compare it with (an index into `tabs`); without it,
        // the stage just before.
        function select(tab, with_) {
            tabs.forEach(function (t) {
                var on = t === tab;
                t.setAttribute("aria-selected", on ? "true" : "false");
                t.tabIndex = on ? 0 : -1;
                document.getElementById(t.getAttribute("aria-controls")).hidden = !on;
            });
            against = typeof with_ === "number" ? with_ : null;
            render();
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
        var seen = { words: wordsOf(sections[0]), picture: null, before: null };

        // A value as one quoted word of a line, as the engine reads one: `\` and `"` escaped.
        function quote(text) {
            return '"' + text.replace(/\\/g, "\\\\").replace(/"/g, '\\"') + '"';
        }

        // What the current stage's picture was marked against (data-before's answer then).
        var drawnWith = null;

        // Read the stage as it is now: the words, what data-before answers, the facts, and then
        // the picture in each scheme. With data-before, a step that moved something draws the
        // picture marked against the last read (`before=`), and one that moved nothing draws it
        // exactly as the current stage was drawn: the same request, so a step that cut nothing
        // finds it in the cache.
        function read(mod) {
            var stage = { pictures: {}, cache: "", facts: [], before: null, moved: false };
            var reads = [];
            reads.push(function () {
                return evalLine(mod, wordsLine).then(function (reply) {
                    stage.words = reply.kind === "error" ? "error: " + reply.text : reply.text;
                });
            });
            if (beforeLine) {
                reads.push(function () {
                    return evalLine(mod, beforeLine).then(function (reply) {
                        stage.before = reply.kind === "output" ? reply.text : null;
                    });
                });
            }
            factItems.forEach(function (item) {
                reads.push(function () {
                    return evalLine(mod, item.getAttribute("data-cmd")).then(function (reply) {
                        stage.facts.push(factText(item, factValue(item, reply)));
                    });
                });
            });
            reads.push(function () {
                if (beforeLine) {
                    stage.moved = !same(stage.words, seen.words) ||
                        (seen.before !== null && stage.before !== seen.before);
                    if (stage.moved) {
                        drawnWith = seen.before;
                    }
                }
                return Promise.resolve();
            });
            schemes.forEach(function (scheme) {
                reads.push(function () {
                    var line = scheme ? pictureLine.replace(/\{scheme\}/g, scheme) : pictureLine;
                    if (beforeLine && drawnWith !== null) {
                        line += " before=" + quote(drawnWith);
                    }
                    return evalLine(mod, line).then(function (reply) {
                        if (reply.kind !== "output") {
                            throw new Error("the picture did not draw: " + reply.text);
                        }
                        stage.pictures[scheme || "only"] = reply.text;
                        stage.cache = stage.cache || reply.cache;
                    });
                });
            });
            if (overview) {
                stage.overview = {};
                overviewSchemes.forEach(function (scheme) {
                    reads.push(function () {
                        var line = scheme ? overviewLine.replace(/\{scheme\}/g, scheme) : overviewLine;
                        return evalLine(mod, line).then(function (reply) {
                            if (reply.kind !== "output") {
                                throw new Error("the picture above the tabs did not draw: " + reply.text);
                            }
                            stage.overview[scheme || "only"] = reply.text;
                            stage.overviewCache = stage.overviewCache || reply.cache;
                        });
                    });
                });
            }
            return reads.reduce(function (done, next) { return done.then(next); }, Promise.resolve())
                .then(function () { return stage; });
        }

        function picture(stage, scheme, alt) {
            return drawnPicture(stage.pictures[scheme || "only"], scheme, alt);
        }

        function drawnPicture(svg, scheme, alt) {
            var url = URL.createObjectURL(new Blob([svg], { type: "image/svg+xml" }));
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

        // ── Before | After ───────────────────────────────────────────────────
        function labelOf(section) {
            return section.getAttribute("data-stage") + " · " + (section.getAttribute("data-label") || "");
        }

        function sectionOf(tab) {
            return document.getElementById(tab.getAttribute("aria-controls"));
        }

        function selectedTab() {
            return tabs.filter(function (t) { return t.getAttribute("aria-selected") === "true"; })[0] || tabs[0];
        }

        // What each tab's own step drew the last time it ran (by section id).
        var drawn = {};
        // How far the reader's steps have reached: the highest chapter tab whose step ran.
        var furthest = 0;
        // The tab whose stage the reader's kernel is at now: the first until a step moves it.
        var current = tabs[0];

        function indexOf(section) {
            return sections.indexOf(section);
        }

        // A step's pictures (each scheme), made once and copied for each place they are shown.
        function picturesOf(snap) {
            if (!snap.pictures) {
                snap.pictures = el("p", "ikigai-stage-picture");
                schemes.forEach(function (scheme) {
                    snap.pictures.appendChild(picture(snap.stage, scheme, altOf(snap.stage.words)));
                });
            }
            return snap.pictures.cloneNode(true);
        }

        // A stage as the viewer shows it: what its tab's step drew in this page if it ran, else
        // the chapter's committed picture and words. Asks the kernel nothing.
        function stageOf(k) {
            var section = sectionOf(tabs[k]);
            var snap = drawn[section.id];
            if (snap) {
                return { section: section, kernel: true, snap: snap, words: snap.stage.words, pictures: picturesOf(snap) };
            }
            var committed = section.querySelector(".ikigai-stage-result .ikigai-stage-picture");
            return {
                section: section, kernel: false, words: wordsOf(section),
                pictures: committed ? committed.cloneNode(true) : el("p", null, "(The chapter keeps no picture of this stage.)"),
            };
        }

        function fill(into, side, stage) {
            into.figure.classList.toggle("ikigai-stages-slot-chapter", !stage.kernel);
            into.figure.classList.toggle("ikigai-stages-slot-kernel", stage.kernel);
            into.caption.textContent = "";
            into.caption.appendChild(el("strong", null, side + ":"));
            // A step that moved nothing yet (the first cell of a tab often sets something the
            // picture does not show) drew the stage the kernel was still at: say which.
            var at = stage.kernel ? sectionOf(stage.snap.at) : null;
            into.caption.appendChild(document.createTextNode(!stage.kernel
                ? " stage " + labelOf(stage.section) + ", the chapter's picture"
                : at === stage.section
                    ? " stage " + labelOf(stage.section) + ", as your kernel drew it"
                    : " stage " + labelOf(at) + ", as your kernel drew it: the step of stage " +
                      labelOf(stage.section) + " has moved nothing yet"));
            into.holder.textContent = "";
            into.holder.appendChild(stage.pictures);
            var where = indexOf(stage.section);
            into.note.textContent = stage.kernel
                ? "Drawn in this page: " + stage.snap.verdict
                : where === 0
                    ? "The space as the page loads it. Your kernel draws nothing until a step runs."
                    : where < furthest
                        ? "Its step has not run in this page (a later tab's has), so this is the chapter's picture of it."
                        : "Run this tab's step and this becomes your kernel's own picture.";
        }

        // Between the tabs and the open panel: the open tab's stage beside the one it is compared
        // with, and what changed from that one to this. Nothing here asks the kernel.
        function render() {
            if (compare.hidden) {
                return;
            }
            var k = tabs.indexOf(selectedTab());
            var c = against !== null && against < k ? against : k - 1;
            syncAgainst(k, c);
            againstRow.hidden = k === 0;
            againstNote.hidden = k !== 0;
            var after = stageOf(k);
            fill(afterSlot, "After", after);
            diff.textContent = "";
            if (c < 0) {
                beforeSlot.figure.classList.remove("ikigai-stages-slot-chapter", "ikigai-stages-slot-kernel");
                beforeSlot.caption.textContent = "";
                beforeSlot.caption.appendChild(el("strong", null, "Before:"));
                beforeSlot.caption.appendChild(document.createTextNode(" nothing"));
                beforeSlot.holder.textContent = "";
                beforeSlot.holder.appendChild(el("p", null, "Nothing comes before the first stage."));
                beforeSlot.note.textContent = "";
                diff.appendChild(changed(null, after.words));
            } else {
                var before = stageOf(c);
                fill(beforeSlot, "Before", before);
                diff.appendChild(changed(before.words, after.words,
                    "What changed from stage " + labelOf(before.section) + " to stage " + labelOf(after.section) + ":"));
            }
            markScrollers();
        }

        // A box that scrolls sideways (a picture on a narrow screen, a long line of words, a
        // command or its answer) takes focus while it overflows, so a keyboard can scroll it
        // (SC 2.1.1; axe: scrollable-region-focusable), as js/a11y.js does for code blocks. A
        // tabindex this script did not add is never removed.
        var SCROLLERS = ".ikigai-stages-slot-picture, .ikigai-stage-changes, .ikigai-run-out, " +
            ".ikigai-run-entry-out, .ikigai-run-entry-cmd";
        var ADDED = "data-stages-scroll-tabindex";

        function overflows(box) {
            return box.scrollWidth > box.clientWidth;
        }

        function markScrollers() {
            Array.prototype.forEach.call(root.querySelectorAll(SCROLLERS), function (box) {
                // A picture's box holds a link, which already takes focus.
                var focusable = box.classList.contains("ikigai-stages-slot-picture") ? false : overflows(box);
                if (focusable && !box.hasAttribute("tabindex")) {
                    box.setAttribute("tabindex", "0");
                    box.setAttribute(ADDED, "");
                } else if (!focusable && box.hasAttribute(ADDED)) {
                    box.removeAttribute("tabindex");
                    box.removeAttribute(ADDED);
                }
            });
        }

        // mdbook binds Left and Right on the document to the previous and next chapter, exempting
        // only form fields. In a box that scrolls sideways, and on the tabs, those keys are the
        // box's or the tabs' own: they stop here (the browser's own scroll still happens).
        root.addEventListener("keydown", function (event) {
            if ((event.key !== "ArrowLeft" && event.key !== "ArrowRight") ||
                event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
                return;
            }
            var box = event.target.closest ? event.target.closest(SCROLLERS) : null;
            if ((box && root.contains(box) && overflows(box)) || tabs.indexOf(event.target) >= 0) {
                event.stopPropagation();
            }
        });
        window.addEventListener("resize", function () { markScrollers(); });

        // Above the tabs: the kernel as it is now, in place of the figure's committed picture.
        var overviewAlt = overview && overview.querySelector("img") ? overview.querySelector("img").alt : altBase;
        function showOverview(stage) {
            if (!overview || !stage.overview) {
                return;
            }
            var caption = overview.querySelector("figcaption");
            Array.prototype.slice.call(overview.childNodes).forEach(function (node) {
                if (node !== caption) {
                    overview.removeChild(node);
                }
            });
            var pictures = el("p", "ikigai-stage-picture");
            overviewSchemes.forEach(function (scheme) {
                pictures.appendChild(drawnPicture(stage.overview[scheme || "only"], scheme,
                    root.getAttribute("data-overview-alt") || overviewAlt));
            });
            overview.insertBefore(pictures, caption);
            if (caption) {
                caption.textContent = "";
                caption.appendChild(document.createTextNode("The whole host as your kernel holds it now, after " +
                    "your last step: "));
                caption.appendChild(el("code", null, overviewLine.replace(/\s*scheme=\{scheme\}/, "")));
                caption.appendChild(document.createTextNode(", " + (stage.overviewCache === "cached"
                    ? "served from the cache: nothing it hangs from was cut."
                    : "drawn now, once: the step cut what it hangs from, or this page had not asked " +
                      "for it yet.")));
            }
        }

        // After a step: read the stage, keep the facts current, say what it made, and show it
        // below the tabs, After, beside the stage before it. The step is compared with what the
        // page last saw, which is where the reader's kernel was.
        function after(step) {
            return loadKernel().then(read).then(function (stage) {
                if (facts && stage.facts.length) {
                    factItems.forEach(function (item, k) {
                        item.textContent = stage.facts[k];
                    });
                }
                showOverview(stage);
                var before = seen.words;
                var light = stage.pictures[schemes[0] || "only"];
                // With data-before the picture is marked, so it is not what says a stage moved:
                // read() judged that by the words and data-before's answer.
                var moved = beforeLine ? stage.moved : !same(stage.words, seen.words) ||
                    (seen.picture !== null && light !== seen.picture);
                seen = { words: stage.words, picture: light, before: stage.before };
                // `at`: the tab whose stage the kernel is at after the step.
                var snap = { stage: stage, before: before, moved: moved, verdict: "", at: current };
                var own = step.section;
                if (own && indexOf(own) > furthest) {
                    furthest = indexOf(own);
                }
                if (!moved) {
                    var cache = stage.cache === "cached" ? "the picture came from the cache" :
                        "the picture was " + stage.cache + ": the first time this page asked for it";
                    status.textContent = step.lead + "No change: this step moved nothing the picture " +
                        "shows, so it makes no stage (" + cache + ").";
                    snap.verdict = "no change, so no new stage (" + cache + ").";
                    if (own) {
                        drawn[own.id] = snap;
                        select(document.getElementById(own.getAttribute("aria-labelledby")));
                    }
                    // A button that moved nothing leaves the view as it was; the status says so.
                    return;
                }
                if (own && !own.classList.contains("ikigai-stage-yours") && same(wordsOf(own), stage.words)) {
                    status.textContent = step.lead + "Your kernel now matches stage " +
                        own.getAttribute("data-stage") + ".";
                    snap.verdict = "your kernel is at stage " + labelOf(own) + ", as the chapter draws it.";
                    current = document.getElementById(own.getAttribute("aria-labelledby"));
                    snap.at = current;
                    drawn[own.id] = snap;
                    select(current);
                    return;
                }
                var made = addStage(stage, before, step.madeBy, step.sentence);
                // Where the kernel was before this step: what a reader's own stage is shown beside.
                var was = tabs.indexOf(current);
                current = made.tab;
                snap.at = current;
                snap.verdict = "a new stage, " + made.n + ", of yours." + (step.sentence ? " " + step.sentence : "");
                drawn[made.tab.getAttribute("aria-controls")] = snap;
                if (own) {
                    // The cell's tab keeps showing what its step drew, and says where it went.
                    drawn[own.id] = {
                        stage: stage, before: before, moved: true, at: current,
                        verdict: "your kernel made a stage that is not this tab's: it is stage " + made.n +
                            ", of yours. (Run the tabs in order on a fresh page to follow the chapter.)",
                    };
                    select(document.getElementById(own.getAttribute("aria-labelledby")));
                } else {
                    // A button's stage, beside the stage the kernel was at before the press.
                    select(made.tab, was);
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
            root.classList.add("ikigai-stages-live");
            root.insertBefore(compare, sections[0]);
            compare.hidden = false;
            // Where a tab's own picture was, the prose before it still leads to it: point at
            // After, below the tabs.
            sections.forEach(function (section) {
                var result = section.querySelector(":scope > .ikigai-stage-result");
                if (result) {
                    result.parentNode.insertBefore(
                        el("p", "ikigai-stage-pointer", "The picture for this stage is below the tabs, as After."),
                        result.nextSibling);
                }
            });
            render();
            // A cell's answer arrives after its run; a box it makes too wide takes focus then.
            if (typeof MutationObserver === "function") {
                var pending = false;
                new MutationObserver(function () {
                    if (!pending) {
                        pending = true;
                        window.requestAnimationFrame(function () {
                            pending = false;
                            markScrollers();
                        });
                    }
                }).observe(root, { childList: true, subtree: true, characterData: true });
            }
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
