# WebTransport: server-rendered

*This chapter is not written yet.*

In [htmx: server-rendered HTML with push](htmx.md), the inspector serves views over HTTP
and sends cuts on a push stream. This chapter will deliver the same server-rendered views
over WebTransport: one session per page, with the kernel's answers and its cuts on streams
of that session. The precedent is the semantic CMS's
[reading room](https://github.com/ikigai-rs/ikigai-cms-web), which already serves a kernel
to its pages this way.

It waits on that transport being available to a host as a library, the way the HTTP door is.
