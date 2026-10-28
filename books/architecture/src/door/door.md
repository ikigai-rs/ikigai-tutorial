# One module, a boundary and a surface

Everything so far has been inside the kernel: a capability, a manifold, a grant. A **door** is
where that inside meets a browser. It is the component that accepts a request from the outside,
turns a proof of identity into a principal, hands the resolution exactly that principal's authority
and no more, and serves the answer back as a page. The claim of this chapter is that a door is the
single clearest example of the whole book's thesis, because it is *at once* a security boundary and
a user-experience surface, and it is written **once** so that every page gets both.

## What a door is responsible for

```text
        request  ──▶  ┌───────────────────────────────────────────┐
                      │                  the door                 │
                      │                                           │
   proof (passkey,    │  1. authenticate → mint a principal       │
   certificate,       │  2. admit by Origin; mutations need       │
   token)             │     Origin + SameSite (never a bare GET)  │
                      │  3. resolve under exactly that authority  │
                      │  4. set security headers (CSP, …)         │
                      │  5. escape every value into the page      │
                      │                                           │
                      └───────────────────┬───────────────────────┘
                                          │
                       page  ◀────────────┘   (or a legible refusal,
                                               never a 200 success shell)
```

Read that list once as a security engineer and once as a front-end engineer. The security engineer
sees authentication, CSRF defense, a content-security policy, and output encoding. The front-end
engineer sees sign-in, forms that submit safely, a page that renders, and errors that read well.
**They are the same five steps.** A door that did the security half without the UX half would serve
blank pages; one that did the UX half without the security half is the vulnerability. Writing the
door once is how a project stops re-deciding those five things, differently and wrongly, per page.

## The escaping module, written once

Step five — putting a value into a page — is where injection lives, and it is also where a broken
link lives, and those turn out to be the same problem solved by the same small module.

Consider two functions such a module provides: one that escapes text for HTML, and one that makes a
URL safe to use as a link. The first prevents a value containing markup from becoming markup — a
security property. The second makes a value that is *not* a valid, safe URL render as **plain
text** instead of as a broken or dangerous link (a `javascript:` URL, say) — a correctness and UX
property. One module, one set of rules, and — because the same rules are needed in Rust, in an
XSLT transform, in a page's own script — **one shared set of test vectors** that every language's
copy must pass. A bug fixed in the escaper is fixed everywhere; a divergence between two hand-rolled
escapers is exactly the class of bug this consolidation removes.

The discipline behind it is worth stating because it generalizes: **do not re-implement behavior a
standard already defines.** Escaping, percent-encoding, auth headers, digests — each wants one
implementation in its natural module, called everywhere, not a private copy per caller that drifts.

## The content type carries a trust decision

Picking which representation to send a client is content negotiation — a reach decision, the UX
face of transreption. The same `Content-Type` carries a security decision. HTML that the door
**authored** is served with the strict content-security policy, because the door knows what is in
it. HTML it **did not** author — a value a caller supplied, a document from a writable mount, a
representation from a peer — is served sandboxed, or as `text/plain`, never same-origin with scripts
it did not vet. The media type is simultaneously "what shall I show you" and "how much do I trust
this to run," and the door decides both at once.

## What a URL may and may not carry

A door makes the system's addressing real: a URL path *is* a resource IRI, so links are stable and
the browser's Back button works. That convenience comes with a companion rule that is pure security:
a URL encodes a resource's declared arguments and **nothing else** — no hidden UI state the resource
did not declare, and above all **no bearer token and no personal data in a query string**. URLs are
logged, shared, and cached; the addressing convention and the leak-prevention rule are the same rule
seen from two sides.

## Bodies larger than memory

A door also frames what it sends and bounds what it takes, and since the streams release it
does both without holding a whole body. What follows is `ikigai serve --http` from
`ikigai-cli` 0.2.2 (ikigai-web 0.2.2, built with the cli's `web` feature), with the file
answers of 0.2.3; *The ikigai Book*'s part on streams says what a body is.

- **Framing.** An answer that is one buffer goes whole, with `Content-Length`, as every
  answer did before. An answer that streams is framed by `Content-Length` when its length is
  known and chunked when it is not. A stream that fails part-way fails *visibly*: a chunked
  answer ends without its last chunk, a measured one ends short, and the connection closes;
  never a well-formed shorter body.
- **Ranges.** A `GET` over a body that can seek, knows its length and has a strong validator
  says `Accept-Ranges: bytes` and honors a `Range` of one byte range: `206 Partial Content`,
  or `416 Range Not Satisfiable` when the range lies past the end. `If-Range` is compared
  strongly against the `ETag`, and anything else (a mismatch, a date, several ranges, a
  `Range` that does not parse) is answered with the whole body and `200`. A file is such a
  body from 0.2.3, so a file download resumes.
- **Uploads.** A route named an upload route (`http.upload` in the config home) does not
  read its body into memory: the door writes it to a spill under
  `<config home>/spill/http-<port>` as it arrives, then hands the sink the content's id. Each
  bound is a refusal, never a truncation: `413` past the route's largest body (checked against
  `Content-Length` before a byte is read) or past the caller's spill grant, `408` for a body
  that falls behind the minimum rate after its grace (`http.upload_min_rate`,
  `http.upload_grace`), and `503` when the uploads in flight hold the most the disk may
  (`http.spill_total`; `http.spill_ceiling` bounds one request). A caller granted no spill is
  served as before, its body in memory under the door's ordinary bound.
- **Validators.** Every `ETag` is now one rule over the type and the content id, whatever
  the body's shape, so a buffer and a stream of the same bytes carry the same tag. Before
  0.2.2 the hash ran over the bytes themselves, so every tag changed **once**: a client's
  cache revalidates each stored answer once, a `200` where it had a `304`, and is right from
  then on.

The bounds are the security half and the framing is the UX half, and they are, once more,
the same decisions: a download that cannot pretend to have finished and an upload that
cannot fill the disk are one rule seen from two sides.

## Where this runs, and where it is drawn

A browser's content-security policy and its Origin checks are enforced by the *real* browser, which
a kernel compiled into a page cannot stand in for. So the door is the one pattern in this book that
is partly a **picture**: the kernel logic runs in the page, but the headers above are shown as
annotations rather than exercised, and the escaping is the piece you can run for real against reader
input. The boundary is real in a deployment; here it is drawn, and labeled as drawn.
