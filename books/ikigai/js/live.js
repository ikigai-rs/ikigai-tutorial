// A picture of a live host, redrawn after every cell that changes it.
//
// A chapter writes
//
//     <figure class="ikigai-live" data-live='workshop'>
//     <a href="live-space-empty.svg"><img src="live-space-empty.svg" alt="…"></a>
//     <figcaption>…</figcaption>
//     </figure>
//
// around a committed picture of the host as it starts, and a cell that carries the same
// `data-live='workshop'` runs its lines in that host (`js/run.js`, chain `live:workshop`). After
// every such cell, `js/run.js` says so (`ikigai:cell-ran`), and this script asks the host for its
// picture (`livePictureAsync`: `urn:diagram:kernel`, read through the host's cache like any other
// read) and draws it in place of the one on the page.
//
// THE NEXT CELL WAITS FOR THE REDRAW. The redraw reads the host, so it is part of the order the
// chapter's outputs assume: after one cell, before the next. It goes to the event's `hold`, and
// `js/run.js` runs nothing more in this host until it has landed. Left to race, a redraw that
// landed after the next cell's write had cut the picture filled the cache again, and that
// cell's `trace` read the picture `cached` where the chapter shows `computed` (ledger #2191).
//
// THE TEXT ALTERNATIVE CHANGES WITH THE PICTURE. The kernel answers the picture together with
// its `alt`, written by `live_space::picture_text` from the same arrangement the picture is drawn
// from, so the words are never older than the boxes. The committed picture's `alt` is that same
// text, held to it by a test in `crates/book-wasm`. The link around the picture follows it too:
// "open it at full size" opens the picture now on the page, not the committed one.
//
// NOTHING IS DRAWN ON LOAD. The committed picture IS the host as it starts (a test holds the two
// byte for byte), and asking the host for it before the reader runs anything would start the host
// and fill its cache before the chapter's first cell. The first redraw follows the first Run.
//
// ONE STATUS LINE per picture, polite, saying it was redrawn and how many declarations have
// bound: the cell's own region already announces what the cell printed, so this says only what
// changed out of view. Without the kernel, the same line says the picture will not be redrawn,
// and the committed picture stays, which is still the truth about a host that never started.
// `a11y/no-kernel.mjs` pins that.
(function () {
    "use strict";

    var figures = document.querySelectorAll("figure.ikigai-live[data-live]");
    if (figures.length === 0) {
        return;
    }

    function loadKernel() {
        if (typeof window.ikigaiBookKernel === "function") {
            return window.ikigaiBookKernel();
        }
        return Promise.reject(new Error("js/run.js, which loads it, is not on this page"));
    }

    function install(figure) {
        var name = figure.getAttribute("data-live");
        var img = figure.querySelector("img");
        var link = img && img.parentNode && img.parentNode.tagName === "A" ? img.parentNode : null;
        if (!img) {
            return;
        }
        var status = document.createElement("p");
        status.className = "ikigai-live-status";
        status.setAttribute("role", "status");
        status.setAttribute("aria-live", "polite");
        figure.appendChild(status);

        var drawn = null;   // the object URL of the picture now on the page, if it is not the file
        var runs = 0;

        function draw() {
            return loadKernel().then(function (mod) {
                return mod.livePictureAsync(name);
            }).then(function (json) {
                var answer = JSON.parse(json);
                if (answer.error) {
                    status.textContent = "The picture could not be redrawn: " + answer.error;
                    return;
                }
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
                runs += 1;
                var caption = figure.querySelector("figcaption");
                if (caption) {
                    caption.textContent = "The live host in this page, as its kernel draws " +
                        "urn:diagram:kernel after the last cell you ran in it (" +
                        answer.installs + " declaration" + (answer.installs === 1 ? "" : "s") +
                        " bound so far). Select the picture to open it at full size.";
                }
                status.textContent = "Picture redrawn (" + runs + "): " +
                    answer.installs + " declaration" + (answer.installs === 1 ? "" : "s") +
                    " bound so far.";
            }).catch(function (err) {
                status.textContent = "The picture could not be redrawn (" +
                    (err && err.message ? err.message : err) + ").";
            });
        }

        loadKernel().catch(function (err) {
            status.textContent = "The in-page kernel did not load (" +
                (err && err.message ? err.message : err) + "), so this picture is not " +
                "redrawn: it shows the host as it starts, before anything is declared.";
        });

        document.addEventListener("ikigai:cell-ran", function (event) {
            if (event.detail && event.detail.chain === "live:" + name) {
                var drawing = draw();
                if (typeof event.detail.hold === "function") {
                    event.detail.hold(drawing);
                }
            }
        });
    }

    for (var i = 0; i < figures.length; i++) {
        install(figures[i]);
    }
})();
