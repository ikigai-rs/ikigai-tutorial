<?xml version="1.0" encoding="UTF-8"?>
<!-- One golden thread, and the cache entries hanging from it. graph:thread:{name} keeps only those. -->
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:ik="https://ikigai-rs.dev/ns#"
    xmlns:insp="urn:iki:tutorial:inspect:term#"
    exclude-result-prefixes="rdf ik insp">
<xsl:output method="html"/>
<xsl:template match="/">
<section id="view" class="inspect">
<span hidden="" data-ui-quiet="" hx-get="{rdf:RDF/ik:GoldenThread/insp:fragment}" hx-trigger="ikigai:cut from:body" hx-target="closest section" hx-swap="outerHTML"></span>
<h1>Golden thread <code><xsl:value-of select="rdf:RDF/ik:GoldenThread/ik:threadName"/></code></h1>
<p>Cut <xsl:value-of select="rdf:RDF/ik:GoldenThread/ik:cutCount"/> times. Cutting it again makes every entry below stale.</p>
<h2>The entries hanging from it</h2>
<ul class="inspect-list">
<xsl:apply-templates select="rdf:RDF/ik:CacheEntry">
<xsl:sort select="ik:resolvedFrom/@rdf:resource"/>
</xsl:apply-templates>
</ul>
</section>
</xsl:template>
<xsl:template match="ik:CacheEntry">
<li><a href="{insp:page}" hx-get="{insp:fragment}" hx-target="#main" hx-push-url="{insp:page}"><xsl:value-of select="ik:resolvedFrom/@rdf:resource"/></a> (<span class="state-{ik:entryState}"><xsl:value-of select="ik:entryState"/></span>)</li>
</xsl:template>
</xsl:stylesheet>
