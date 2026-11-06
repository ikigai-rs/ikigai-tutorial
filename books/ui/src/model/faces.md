# One model, many faces

*This chapter is not written yet.*

It will cover **transreption** for user interfaces: converting a resource's representation
into another form. The same model can have a `text/html` face for a browser, a `text/plain`
face for a terminal, and a Turtle or JSON-LD face for an agent or a dashboard.

In [Graph to page with XSLT](xslt.md), the inspector already reads the kernel's Turtle graphs.
The cache, threads and scheduler readouts also have a text face (the one The ikigai Book prints).
The chapter will make the inspector's own views answer the same way, and show a page linking
its other faces with `rel=alternate`.

It waits on the inspector's text faces, which are the terminal target's first step.
