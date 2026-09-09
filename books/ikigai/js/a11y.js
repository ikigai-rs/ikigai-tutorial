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

    // Two repairs to markup this repository does not own, the search box's — both
    // upstream mdbook 0.5.4, both reported by `crates/book-a11y` on the static page, and
    // both applied here at load because the alternative is forking the page template.
    //
    // 1. `<input type="search" id="mdbook-searchbar">` has a placeholder and no label.
    //    A placeholder is not an accessible name (WCAG 2.2 SC 4.1.2 Name, Role, Value;
    //    SC 3.3.2 Labels or Instructions), so a screen reader announces an unnamed edit
    //    field. The placeholder text becomes the name, so what is heard is what is seen.
    // 2. Its `aria-describedby="searchresults-header"` names an id that does not exist —
    //    the element is `mdbook-searchresults-header` — so the description is silently
    //    lost (SC 1.3.1 Info and Relationships). Re-point it at the element that is there.
    function repairSearch() {
        var input = document.getElementById("mdbook-searchbar");
        if (!input) {
            return;
        }
        if (!input.hasAttribute("aria-label") && !input.hasAttribute("aria-labelledby")) {
            input.setAttribute("aria-label", input.getAttribute("placeholder") || "Search this book");
        }
        var described = input.getAttribute("aria-describedby");
        if (described && !document.getElementById(described)) {
            var actual = document.getElementById("mdbook-" + described);
            if (actual) {
                input.setAttribute("aria-describedby", actual.id);
            } else {
                input.removeAttribute("aria-describedby");
            }
        }
    }

    // The sidebar toggle is a `<label for=checkbox>` styled as a button, and the checkbox
    // it drives is `display: none` — so nothing in the pair can take focus, and a keyboard
    // user cannot open or close the table of contents (SC 2.1.1 Keyboard). mdbook's own
    // script also writes `aria-expanded` onto the label, which is not an attribute a label
    // may carry (SC 4.1.2 Name, Role, Value; axe: aria-allowed-attr). Both are one repair:
    // make the label the button it looks like — a role, a tab stop, Enter and Space — and
    // keep `aria-expanded` true to the checkbox it controls.
    function repairSidebarToggle() {
        var label = document.getElementById("mdbook-sidebar-toggle");
        var anchor = document.getElementById("mdbook-sidebar-toggle-anchor");
        if (!label || !anchor) {
            return;
        }
        label.setAttribute("role", "button");
        label.setAttribute("tabindex", "0");
        var sync = function () {
            label.setAttribute("aria-expanded", anchor.checked ? "true" : "false");
        };
        sync();
        anchor.addEventListener("change", sync);
        label.addEventListener("keydown", function (event) {
            if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                anchor.click();
            }
        });
    }

    // mdbook renders the previous/next chapter links twice — once inline, once for wide
    // screens — as two `<nav aria-label="Page navigation">`. Same role, same name: a
    // screen reader's landmark list shows two entries it cannot tell apart (axe:
    // landmark-unique, a best practice under SC 1.3.1). Name the second for what it is.
    function repairDuplicateNavLabels() {
        var navs = document.querySelectorAll('nav[aria-label="Page navigation"]');
        if (navs.length > 1) {
            navs[navs.length - 1].setAttribute("aria-label", "Page navigation (wide screens)");
        }
    }

    function install() {
        repairSearch();
        repairSidebarToggle();
        repairDuplicateNavLabels();
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
