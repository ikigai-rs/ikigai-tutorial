# A note from a human

This tutorial is obviously generated using agentic tools, based on their
understanding of the larger project. I know this may be off-putting to
some readers, but it is simultaneously a convenience and a reality
that allowed me to get a reasonably comprehensive overview of the
[ikigai project](https://ikigai-rs.dev) out the door. I will be
revisiting the documentation and tutorial over time with an
increasingly human touch. For now I beg your patience.

## An acknowledgement of the highest order

ikigai represents a vision I have had for 10 to 15 years, based
predominantly on a software ecosystem I have been using for 23 years
at this point: NetKernel. I was exposed to it first in 2003 and
had no idea what it meant or was good for. A year later I migrated an
XML processing pipeline to it in a week and picked up linear
scalability in the process with virtually no effort.

NetKernel is the brainchild of [1060
Research](https://1060research.com) and I owe them entire credit for
the ideas, terminology, and vision. Peter Rodgers and Tony Butterfield
have produced an amazing body of work. Basically what I have done is
take those ideas and expand their applicability beyond the JVM (and
back onto it; more on that soon) across a dramatically wider
footprint, with some personal design tweaks and additions.

## What's the point?

As you go through this tutorial, I expect a common reaction will be
puzzlement and confusion about what the purpose is. That's a fair
assessment, but I guarantee you there is a point, and it will be
revealed in practice over time. Things are moving very quickly and
ikigai is not really ready for a general audience, but enough people
were asking about it that I thought it was time to start discussing it.

The purpose of the project is to benefit from the incomparable
compositional power of resource-oriented design. You can think of it a
bit like the Web meets Unix pipes and filters, but that is simply a
reductive convenience.

NetKernel was clearly ahead of its time and so many things about where
the industry has gone have only started to approximate what it has
been able to do for decades. I've added capability-based
security, standards-based linked data bones, architectural
flexibility, freedom from the JVM, browser residency, and more. But I
have no illusions that I am doing anything but extending what has
already been laid down.

## Why?

These ideas are transformative when they are embraced. I have had both
success and failure in trying to get developers to think this way. I'm
trying to address some of the onboarding impedance (which you may not
believe from this tutorial, but with patience I hope it'll become
clearer and more true).

These ideas are also of this time. I haven't been designing these
things with AI in mind, but the synergy is natural and
compelling. There are several ways LLMs are invoked from ikigai, but
only when they need to be. This, too, is part of the story I hope to
continue to unveil in the coming weeks.

## Rust

ikigai is built in Rust for all of the reasons you would think. It is
safe, fast, compiles to WebAssembly, well-supported by the crates
ecosystem, and more. If you are unfamiliar with Rust, don't be put
off. The goal is that you will be able to benefit from ikigai under a
variety of deployments. A lot of that is still to be built, but you
can start to get a sense of using [`Python`](polyglot/python.md) or
[`TypeScript`](polyglot/typescript.md) as either the source of
resources or a consumer of the ikigai engine. There will be other
wrappers for other languages, WebAssembly-based jails with WASI,
several scripting languages, and more. There's already support for
Scheme as an embedded language. It can be digitally-signed,
transrepted into RDF, queried, and other things that are likely to
excite and blow your mind. There is also an emacs REPL that includes
generated elisp aliases for the ikigai functionality your host
projects to use within that environment. Additional Lisp variants and
modalities.

## Next steps

For now, if the ideas intrigue you, spend some time thinking through
the concepts and I'll continue unveiling the bigger picture in
time. If you'd like an introductory session, feel free to [request a
chat](https://www.bosatsu.net/contact.html) through the "Request time"
section and I'll try to accommodate. (As a teaser, you'll be using
ikigai in the process of doing so.)

Proceed with an open mind. Cool things are coming.

Regards,  
[Brian Sletten](https://www.bosatsu.net/about.html)
