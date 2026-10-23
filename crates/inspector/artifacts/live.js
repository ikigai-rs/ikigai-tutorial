// The inspector's push listener (UI conventions D9), until ikigai-ui's shell script carries it.
//
// One EventSource on the door's push stream, under the page's own capability. Every `cut` (and
// every `resync`, which means cuts were missed) is announced to the page as one `ikigai:cut`
// event on <body>; a view that shows changing data listens for it with
// `hx-trigger="ikigai:cut from:body"` and refetches itself, quietly ([data-ui-quiet]), so the
// status region never repeats it. Nothing is parsed as markup and nothing is evaluated.
(function () {
  "use strict";
  if (!window.EventSource) return;
  var source = new EventSource("/_ikigai/push");
  function announce() {
    document.body.dispatchEvent(new CustomEvent("ikigai:cut"));
  }
  source.addEventListener("cut", announce);
  source.addEventListener("resync", announce);
})();
