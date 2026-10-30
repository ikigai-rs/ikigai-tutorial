# WebAssembly: in the browser

*This chapter is not written yet.*

It will run the inspector's kernel in the page itself, as this book's own cells already do,
and render its views there rather than print them. The precedent is The ikigai Book's in-page
kernel, which plays tic-tac-toe and recalculates a spreadsheet in the page, and listens for its
own cuts.

It waits on two pieces of this book's kernel (`crates/ui-book-wasm`): a listener the page can
wait on, and a script that renders a view's HTML in place, as
[Live data and history](../model/live.md) says.
