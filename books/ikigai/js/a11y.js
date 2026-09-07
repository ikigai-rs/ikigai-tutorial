// A skip link, added rather than themed.
//
// mdbook's page template has no skip link and this book does not fork the theme for one:
// `book.toml` supports `additional-js`, so the link is inserted into the page mdbook
// generated. The trade is stated plainly — with JavaScript off there is no skip link,
// which is a much smaller loss than a forked template that silently stops tracking
// upstream's markup.
//
// WCAG 2.2 SC 2.4.1 (Bypass Blocks) is the rule. The reason it matters in *this* book is
// that the sidebar lists every chapter on every page, so without it a keyboard or
// screen-reader user traverses the whole table of contents before reaching the words.
(function () {
    "use strict";

    var TARGET = "mdbook-content";

    function install() {
        if (document.querySelector(".skip-link")) {
            return;
        }
        var content = document.getElementById(TARGET);
        if (!content) {
            return;
        }

        // The target must be focusable for the jump to move focus and not merely scroll —
        // otherwise the next Tab continues from the sidebar, which is the thing being
        // skipped.
        if (!content.hasAttribute("tabindex")) {
            content.setAttribute("tabindex", "-1");
        }

        var link = document.createElement("a");
        link.className = "skip-link";
        link.href = "#" + TARGET;
        link.textContent = "Skip to content";
        document.body.insertBefore(link, document.body.firstChild);
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", install);
    } else {
        install();
    }
})();
