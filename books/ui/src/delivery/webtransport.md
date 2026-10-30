# WebTransport: server-rendered

*This chapter is not written yet.*

It will deliver the same server-rendered views over WebTransport instead of HTTP requests: one
session per page, the kernel's answers and its cuts on streams of that session. The precedent
is the semantic CMS's reading room, which already serves a kernel to its pages this way.

It waits on that transport being available to a host as a library, the way the HTTP door is.
