<?xml version="1.0" encoding="UTF-8"?>
<!-- The cache view: one row per entry of urn:kernel:cache, as the viewer may see it.
     Values go out through xsl:value-of and attribute value templates only (UI conventions D2). -->
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:ik="https://ikigai-rs.dev/ns#"
    xmlns:insp="urn:iki:tutorial:inspect:term#"
    exclude-result-prefixes="rdf ik insp">
<xsl:output method="html"/>
<xsl:template match="/">
<section id="view" class="inspect">
<span hidden="" data-ui-quiet="" hx-get="/inspect/fragment/cache" hx-trigger="ikigai:cut from:body" hx-target="closest section" hx-swap="outerHTML"></span>
<h1>Cache</h1>
<p>What the representation cache holds for you: <xsl:value-of select="count(rdf:RDF/ik:CacheEntry)"/> entries. A <strong>live</strong> entry would be served from the cache; a <strong>cut</strong> one is resident but stale, and the next read recomputes it.</p>
<table class="inspect-table">
<caption>Cache entries, by the name each was resolved from</caption>
<thead><tr><th scope="col">Resolved from</th><th scope="col">State</th><th scope="col">Type</th><th scope="col">Bytes</th></tr></thead>
<tbody>
<xsl:apply-templates select="rdf:RDF/ik:CacheEntry">
<xsl:sort select="ik:resolvedFrom/@rdf:resource"/>
</xsl:apply-templates>
</tbody>
</table>
</section>
</xsl:template>
<xsl:template match="ik:CacheEntry">
<tr class="state-{ik:entryState}">
<td><a href="{insp:page}" hx-get="{insp:fragment}" hx-target="#main" hx-push-url="{insp:page}"><xsl:value-of select="ik:resolvedFrom/@rdf:resource"/></a></td>
<td><xsl:value-of select="ik:entryState"/></td>
<td><xsl:value-of select="ik:mediaType"/></td>
<td><xsl:value-of select="ik:sizeBytes"/></td>
</tr>
</xsl:template>
</xsl:stylesheet>
