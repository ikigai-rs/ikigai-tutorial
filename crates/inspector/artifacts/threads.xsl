<?xml version="1.0" encoding="UTF-8"?>
<!-- The threads view: every golden thread that has been cut, and how often (urn:kernel:threads). -->
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:ik="https://ikigai-rs.dev/ns#"
    xmlns:insp="urn:iki:tutorial:inspect:term#"
    exclude-result-prefixes="rdf ik insp">
<xsl:output method="html"/>
<xsl:template match="/">
<section id="view" class="inspect">
<span hidden="" data-ui-quiet="" hx-get="/inspect/fragment/threads" hx-trigger="ikigai:cut from:body" hx-target="closest section" hx-swap="outerHTML"></span>
<h1>Golden threads</h1>
<p>Every thread that has been cut, and how many times. A cached answer is valid only while every thread it hangs from still stands at the count it was stored under.</p>
<table class="inspect-table">
<caption>Cut threads, by name</caption>
<thead><tr><th scope="col">Thread</th><th scope="col">Cuts</th></tr></thead>
<tbody>
<xsl:apply-templates select="rdf:RDF/ik:GoldenThread">
<xsl:sort select="ik:threadName"/>
</xsl:apply-templates>
</tbody>
</table>
</section>
</xsl:template>
<xsl:template match="ik:GoldenThread">
<tr>
<td><a href="{insp:page}" hx-get="{insp:fragment}" hx-target="#main" hx-push-url="{insp:page}"><xsl:value-of select="ik:threadName"/></a></td>
<td><xsl:value-of select="ik:cutCount"/></td>
</tr>
</xsl:template>
</xsl:stylesheet>
