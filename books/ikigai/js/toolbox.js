// The RDF Toolbox, opened IN PLACE on one of a chapter's resources ("Checking a resource").
//
// A chapter writes
//
//     <div class="toolbox-check" data-iri='urn:…:player:o' data-shapes='urn:…:shapes'
//          data-name='player O against the player shapes'
//          data-faces='building/check/player-o.ttl building/check/shapes.ttl'>
//     <p class="toolbox-check-fallback"><a href="../toolbox/toolbox.html?from=…#shacl"
//        target="_blank" rel="noopener">Check …</a> …</p>
//     </div>
//
// WITH THE KERNEL, the check opens in the page. The book's in-page kernel (`crates/book-wasm`)
// composes `urn:toolbox:check:{iri}` with `shapes=` and the page's theme, from the toolbox
// build's own template, which it fetches from `toolbox/` beside the book (`urn:iki:toolbox:dist:`,
// published by `scripts/build-toolbox.sh`). This script is then the HOST the toolbox's README
// describes, in the reader's tab: it routes the page's `/toolbox/{…}` paths to the published
// files, and the page's two inputs, `/toolbox/input/{iri}`, to `urn:toolbox:input:{iri}` in the
// same kernel, which reads each one as its Turtle face. The page is written into a same-origin
// frame in an expanding panel under the control. Its scripts, and the 8.3 MB engine its worker
// compiles, load only then: nothing of the toolbox is fetched until a reader asks.
//
// WITHOUT THE KERNEL (or without scripts), the markup's own link stays: the toolbox page in a new
// tab, opened on copies of the two Turtle faces published beside the chapter
// (`crates/check-resource/tests/published_faces.rs` holds them equal to what the kernel answers a
// reader with the scope). `a11y/no-kernel.mjs` pins that.
//
// FOCUS. The control is a button with `aria-expanded`; opening moves focus to the panel's title,
// which names what is being checked; Close, and Escape anywhere in the panel or its frame, close
// it and return focus to the button. The panel's one status line (polite) says what is loading.
(function () {
    "use strict";

    var checks = document.querySelectorAll("div.toolbox-check[data-iri]");
    if (checks.length === 0) {
        return;
    }

    var root = typeof path_to_root === "string" ? path_to_root : "./";
    // The absolute path of the published build: what `/toolbox/` means on this site.
    var base = new URL(root + "toolbox/", location.href).pathname;
    var siteRoot = new URL(root, location.href).pathname;
    // Where the kernel's `urn:iki:toolbox:dist:{path}` door fetches the build from
    // (`crates/book-wasm`, `fetch_published`).
    window.ikigaiToolboxBase = base;

    function loadKernel() {
        if (typeof window.ikigaiBookKernel === "function") {
            return window.ikigaiBookKernel();
        }
        return Promise.reject(new Error("js/run.js, which loads it, is not on this page"));
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

    // `/toolbox/input/{iri}`, asked by a toolbox page in one of this page's frames: the IRI's
    // Turtle face from the kernel, as the Response a server would send. A refusal is the
    // kernel's own words, with the status a server would give it.
    window.ikigaiToolboxInput = function (iri) {
        return loadKernel().then(function (mod) {
            return mod.issueAsync("", "source", "urn:toolbox:input:" + iri);
        }).then(function (json) {
            var reply = JSON.parse(json);
            var ok = reply.kind === "output";
            return new Response(reply.text, {
                status: ok ? 200 : (/^denied/.test(reply.text) ? 403 : 404),
                headers: { "content-type": "text/plain; charset=utf-8" },
            });
        });
    };

    // mdbook's light themes, and the rest dark: the palette the toolbox page is composed with.
    function theme() {
        var classes = document.documentElement.classList;
        return classes.contains("light") || classes.contains("rust") ? "light" : "dark";
    }

    // The composed page, routed: its `/toolbox/` paths to the published build, its inputs to
    // this page's kernel (the module script put first in its head, which runs before the page's
    // own `app.js`, since module scripts run in document order), and the tool it opens on.
    function routed(html, tool) {
        var doc = new DOMParser().parseFromString(html, "text/html");
        var moved = function (path) {
            return path.indexOf("/toolbox/") === 0 ? base + path.slice("/toolbox/".length) : path;
        };
        doc.querySelectorAll("link[href], script[src]").forEach(function (node) {
            var attr = node.hasAttribute("href") ? "href" : "src";
            node.setAttribute(attr, moved(node.getAttribute(attr)));
        });
        var inputs = [];
        try {
            inputs = JSON.parse(doc.body.getAttribute("data-inputs") || "[]");
        } catch (e) {
            inputs = [];
        }
        doc.body.setAttribute("data-inputs", JSON.stringify(inputs.map(moved)));
        // The header's link to the input's text face names a path no server here answers;
        // keep its text (the IRI being checked) and drop the link.
        doc.querySelectorAll("a.checked").forEach(function (a) {
            a.removeAttribute("href");
        });
        repair(doc);
        var shim = doc.createElement("script");
        shim.type = "module";
        shim.textContent =
            "const prefix = " + JSON.stringify(base + "input/") + ";\n" +
            "history.replaceState(null, \"\", location.pathname + location.search + " +
            JSON.stringify("#" + tool) + ");\n" +
            "const real = window.fetch.bind(window);\n" +
            "window.fetch = (input, init) => {\n" +
            "  const url = new URL(typeof input === \"string\" ? input : input.url, location.href);\n" +
            "  if (url.origin === location.origin && url.pathname.startsWith(prefix)) {\n" +
            "    return parent.ikigaiToolboxInput(decodeURIComponent(url.pathname.slice(prefix.length)));\n" +
            "  }\n" +
            "  return real(input, init);\n" +
            "};\n";
        doc.head.insertBefore(shim, doc.head.firstChild);
        return "<!doctype html>\n" + doc.documentElement.outerHTML;
    }

    // TWO REPAIRS to the toolbox page, made by this host as `js/a11y.js` repairs mdbook's:
    // at load, recorded in ACCESSIBILITY.md, and owed upstream (ikigai-toolbox), where the fix
    // removes them. Both were found by `a11y/axe-browser.mjs` in the opened panel.
    //  1. Upstream's `--faint` (the input and tool counts, the drop hint) has no shared token in
    //     ikigai-toolbox's `tokens.css`, and is 2.34:1 on its light surface and 3.72:1 on its dark
    //     one (SC 1.4.3 needs 4.5:1). It is mapped to `--muted`, the token's own muted text.
    //  2. The two hidden file inputs the Open buttons click have no name (SC 4.1.2). They are
    //     out of the tab order and named for what they hold.
    function repair(doc) {
        var style = doc.createElement("style");
        style.textContent = ":root:is([data-theme], :not([data-theme])) { --faint: var(--muted); }";
        doc.head.appendChild(style);
        var names = { picker: "Files to open", "folder-picker": "A folder to open" };
        Object.keys(names).forEach(function (id) {
            var input = doc.getElementById(id);
            if (input && !input.hasAttribute("aria-label")) {
                input.setAttribute("aria-label", names[id]);
            }
        });
    }

    var count = 0;

    function install(check, mod) {
        var iri = check.getAttribute("data-iri");
        var shapes = check.getAttribute("data-shapes");
        var name = check.getAttribute("data-name") || iri;
        var fallback = check.querySelector(".toolbox-check-fallback");
        var link = fallback ? fallback.querySelector("a") : null;
        count += 1;
        var id = "toolbox-check-" + count;

        var bar = el("div", "toolbox-check-bar");
        var open = el("button", "toolbox-check-open", "Check " + name + " here");
        open.type = "button";
        open.setAttribute("aria-expanded", "false");
        open.setAttribute("aria-controls", id);
        open.setAttribute("aria-describedby", id + "-cost");
        var cost = el("span", "toolbox-check-cost",
            "Opens the RDF Toolbox in this page; the first one loads its 8.3 MB checking engine.");
        cost.id = id + "-cost";
        bar.appendChild(open);
        bar.appendChild(cost);

        var panel = el("div", "toolbox-panel");
        panel.id = id;
        panel.hidden = true;
        panel.setAttribute("role", "region");
        panel.setAttribute("aria-labelledby", id + "-title");
        var head = el("div", "toolbox-panel-head");
        var title = el("p", "toolbox-panel-title",
            "RDF Toolbox: " + iri + (shapes ? " against " + shapes : ""));
        title.id = id + "-title";
        title.tabIndex = -1;
        var close = el("button", "toolbox-panel-close", "Close");
        close.type = "button";
        head.appendChild(title);
        head.appendChild(close);
        var status = el("p", "toolbox-panel-status", "");
        status.setAttribute("role", "status");
        status.setAttribute("aria-live", "polite");
        panel.appendChild(head);
        panel.appendChild(status);

        // The no-kernel link, offered again inside the panel when composing fails.
        function offerNewTab(why) {
            status.textContent = why + " ";
            if (link) {
                var again = link.cloneNode(true);
                again.textContent = "Open it in a new tab instead";
                status.appendChild(again);
            }
        }

        var frame = null;

        function watchReady() {
            var body = frame.contentDocument && frame.contentDocument.body;
            if (!body) {
                return;
            }
            var say = function () {
                if (body.getAttribute("data-ready") === "true") {
                    status.textContent = "The toolbox is ready: its engine checked the resource in this tab.";
                    observer.disconnect();
                }
            };
            var observer = new MutationObserver(say);
            observer.observe(body, { attributes: true, attributeFilter: ["data-ready"] });
            say();
            frame.contentDocument.addEventListener("keydown", onKey);
        }

        function compose() {
            status.textContent = "Composing the check page in this page's kernel…";
            var args = [["theme", theme()]];
            if (shapes) {
                args.unshift(["shapes", shapes]);
            }
            mod.issueWithArgsAsync("", "source", "urn:toolbox:check:" + iri, JSON.stringify(args))
                .then(function (json) {
                    var reply = JSON.parse(json);
                    if (reply.kind !== "output") {
                        frame = null;
                        offerNewTab("The kernel could not compose the check page (" + reply.text + ").");
                        return;
                    }
                    status.textContent = "Loading the toolbox and its 8.3 MB engine…";
                    frame = document.createElement("iframe");
                    frame.className = "toolbox-frame";
                    frame.title = "RDF Toolbox checking " + name;
                    panel.appendChild(frame);
                    var doc = frame.contentDocument;
                    doc.open();
                    doc.write(routed(reply.text, shapes ? "shacl" : "validate"));
                    doc.close();
                    watchReady();
                }, function (err) {
                    offerNewTab("The page failed while composing the check page (" +
                        (err && err.message ? err.message : err) + ").");
                });
        }

        function show() {
            panel.hidden = false;
            open.setAttribute("aria-expanded", "true");
            title.focus();
            if (!frame) {
                frame = true; // composing; a second press does not compose twice
                compose();
            }
        }

        function hide() {
            panel.hidden = true;
            open.setAttribute("aria-expanded", "false");
            open.focus();
        }

        function onKey(event) {
            if (event.key === "Escape" && !panel.hidden) {
                event.preventDefault();
                hide();
            }
        }

        open.addEventListener("click", function () {
            if (panel.hidden) {
                show();
            } else {
                hide();
            }
        });
        close.addEventListener("click", hide);
        panel.addEventListener("keydown", onKey);

        if (fallback) {
            fallback.hidden = true;
        }
        check.appendChild(bar);
        check.appendChild(panel);
    }

    // The no-kernel link, pointed at this site's own copies whatever prefix it is served under
    // (the markup's href is the published site's).
    function pointFallback(check) {
        var link = check.querySelector(".toolbox-check-fallback a");
        var faces = (check.getAttribute("data-faces") || "").split(/\s+/).filter(Boolean);
        if (!link || faces.length === 0) {
            return;
        }
        var query = faces.map(function (f) {
            return "from=" + encodeURIComponent(siteRoot + f);
        }).join("&");
        link.href = base + "toolbox.html?" + query + (check.getAttribute("data-shapes") ? "#shacl" : "#validate");
    }

    function installAll() {
        for (var i = 0; i < checks.length; i++) {
            pointFallback(checks[i]);
        }
        loadKernel().then(function (mod) {
            for (var i = 0; i < checks.length; i++) {
                install(checks[i], mod);
            }
        }, function () {
            // No kernel: the new-tab links are the whole affordance, as the markup wrote them.
        });
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", installAll);
    } else {
        installAll();
    }
})();
