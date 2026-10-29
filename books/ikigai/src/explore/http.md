# Somebody else's resource

The recorded week is yours: a file in your workspace, which changes only when you change
it. The live feed is USGS's. They decide what it says, they change it every minute, and
they say how long a copy of it stays good. This chapter reads it through the kernel and
shows what the kernel does with each of those facts.

This page's kernel does not reach the network, so every step here is for the `ikigai` REPL
you started in [A week of earthquakes](index.md#what-you-need). That is a choice, not a
limit of the browser: USGS lets any page read this feed, and ikigai-http runs in a browser
over its `fetch`. But every Run cell in this book is checked against the answer printed
beside it, and a feed that changes every minute would never match, so the live steps stay
in your REPL. The answers below are what
`ikigai-cli` 0.2.4 printed on 2026-10-28. Yours will have different numbers, because the
feed has moved on since.

## 1. A URL is a name

`urn:http:get` reads a URL and answers what the server sent. The past week of earthquakes
of magnitude 4.5 and up is
`https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson`. Reading it
prints about 130 KB, so pipe it to `urn:text:wc`, which counts lines:

```text
source urn:http:get url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson | urn:text:wc
```

```text
189
[2 computed]
```

The feed uses the same layout as the recorded week, one earthquake to a line, so this
week had 189.

## 2. How long a copy stays good

Ask the cache about the same request:

```text
cache urn:http:get url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson
```

```text
cached
```

The kernel kept the answer, because USGS said it could. Ask the server directly, with
`curl`, and it says for how long:

```bash
curl -sI https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson | grep -i cache-control
```

```text
cache-control: public, max-age=60
```

Sixty seconds. `urn:http:get` reads that header and gives its answer the same lifetime.
Wait a minute and ask again:

```text
cache urn:http:get url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson
source urn:http:get url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson | urn:text:wc
```

```text
not cached
189
[2 computed]
```

The copy expired on USGS's schedule, and the next read fetched the feed again. The count
was computed again too, though nobody asked for that: an answer is only as fresh as what it
was computed from. Anything you build on the live feed is good for a minute at most, and
anything built on the recorded week is good until the file changes. That difference comes
back later in this part, when a page joins the two.

## 3. The URL is a thread

In [A lab with names in it](lab.md#7-what-hangs-from-the-sample), the answers computed
from the recorded week hung from a thread named after the file. A URL works the same way:
the thread is the URL itself. Ask what hangs from it now:

```text
source urn:kernel:dependents thread=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson
```

```text
dependents of https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson  (cut 0 times)
  2 live cached entries hang from it — a cut makes them stale, and the next read of each recomputes it
  urn:http:get  application/json           129.1 KB  2 threads   root
  urn:text:wc   text/plain                      3 B  3 threads   root

[uncacheable]
```

The feed and the count computed from it, which is the read you just made.

You do not have to wait out the minute. If you know the feed has changed (an earthquake
was just in the news), cut the thread, and both answers go stale at once:

```text
sink urn:kernel:cut https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson
cache urn:http:get url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson
```

```text
cut https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson

[uncacheable]
not cached
```

## 4. Keeping a copy

A copy you keep is yours again. Pipe the feed into a file in the workspace:

```text
source urn:http:get url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson | sink urn:file:week.json
```

```text
wrote 132235 bytes to week.json
[1 computed · 1 uncacheable]
```

The write is uncacheable (it has to happen every time you ask), and the read before it was
computed, because the cut above made the cached copy stale. `urn:file:week.json` now holds the week as it was a moment ago. It no longer
expires after a minute; it changes when you write it again. [Reshaping JSON with jq](jq.md)
asks it questions. The name ends in `.json` on purpose, which that chapter explains.

## 5. Only the hosts you allow

`urn:http:get` can read any URL the session's authority allows, and the session starts with
all of it: `root`. A session can give that up. `cap` narrows the session's capability, and
`urn:cap:net:earthquake.usgs.gov` is the authority to reach that one host and nothing else:

```text
cap urn:cap:net:earthquake.usgs.gov
cache urn:http:get url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson
source urn:http:get url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson | urn:text:wc
source urn:http:get url=https://example.com/
cap reset
```

```text
narrowed — capability: urn:cap:net:earthquake.usgs.gov
not cached
189
[2 computed]
error: denied: capability does not grant `GET` to `example.com/`
reset to identity — capability: root (full authority)
```

USGS still answers, and any other host is refused before a request is sent. The refusal
names what was asked for and what the session lacks.

The `not cached` on the second line is not a mistake. The feed had just been read, but by
a session with different authority, and the cache keeps answers apart by the authority
that computed them. A narrower session cannot be handed an answer it might not have been
allowed to compute.

`cap reset` gives the session its full authority back. The narrowing lasts until you reset
it, so reset it before going on.

Next, the questions get sharper: [Reshaping JSON with jq](jq.md).
