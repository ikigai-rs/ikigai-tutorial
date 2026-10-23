<?xml version="1.0" encoding="UTF-8"?>
<!-- One cache entry, and the golden threads it hangs from. graph:entry:{id} keeps only those. -->
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:ik="https://ikigai-rs.dev/ns#"
    xmlns:insp="urn:iki:tutorial:inspect:term#"
    exclude-result-prefixes="rdf ik insp">
<xsl:output method="html"/>
<xsl:template match="/">
<section id="view" class="inspect">
<span hidden="" data-ui-quiet="" hx-get="{rdf:RDF/ik:CacheEntry/insp:fragment}" hx-trigger="ikigai:cut from:body" hx-target="closest section" hx-swap="outerHTML"></span>
<xsl:apply-templates select="rdf:RDF/ik:CacheEntry"/>
<h2>The threads it hangs from</h2>
<p>Cut any of these and the entry is stale: the next read recomputes it.</p>
<ul class="inspect-list">
<xsl:apply-templates select="rdf:RDF/ik:GoldenThread">
<xsl:sort select="ik:threadName"/>
</xsl:apply-templates>
</ul>
</section>
</xsl:template>
<xsl:template match="ik:CacheEntry">
<h1>Cache entry for <code><xsl:value-of select="ik:resolvedFrom/@rdf:resource"/></code></h1>
<dl class="inspect-facts">
<dt>State</dt><dd class="state-{ik:entryState}"><xsl:value-of select="ik:entryState"/></dd>
<dt>Type</dt><dd><xsl:value-of select="ik:mediaType"/></dd>
<dt>Bytes</dt><dd><xsl:value-of select="ik:sizeBytes"/></dd>
<dt>Chain</dt><dd><code><xsl:value-of select="ik:chain/@rdf:resource"/></code></dd>
</dl>
</xsl:template>
<xsl:template match="ik:GoldenThread">
<li><a href="{insp:page}" hx-get="{insp:fragment}" hx-target="#main" hx-push-url="{insp:page}"><xsl:value-of select="ik:threadName"/></a> (cut <xsl:value-of select="ik:cutCount"/> times)</li>
</xsl:template>
</xsl:stylesheet>
