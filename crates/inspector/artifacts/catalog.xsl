<?xml version="1.0" encoding="UTF-8"?>
<!-- The catalog view: every endpoint's own description (urn:kernel:catalog). -->
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:ik="https://ikigai-rs.dev/ns#"
    xmlns:insp="urn:iki:tutorial:inspect:term#"
    exclude-result-prefixes="rdf ik insp">
<xsl:output method="html"/>
<xsl:template match="/">
<section id="view" class="inspect">
<h1>Catalog</h1>
<p>Every endpoint this kernel binds, as it describes itself: <xsl:value-of select="count(rdf:RDF/ik:Endpoint)"/> endpoints.</p>
<table class="inspect-table">
<caption>Endpoints, by id</caption>
<thead><tr><th scope="col">Id</th><th scope="col">Title</th><th scope="col">Verbs</th><th scope="col">Summary</th></tr></thead>
<tbody>
<xsl:apply-templates select="rdf:RDF/ik:Endpoint">
<xsl:sort select="ik:id"/>
</xsl:apply-templates>
</tbody>
</table>
</section>
</xsl:template>
<xsl:template match="ik:Endpoint">
<tr>
<td><code><xsl:value-of select="ik:id"/></code></td>
<td><xsl:value-of select="ik:title"/></td>
<td><xsl:for-each select="ik:verb"><xsl:value-of select="concat(., ' ')"/></xsl:for-each></td>
<td><xsl:value-of select="ik:summary"/></td>
</tr>
</xsl:template>
</xsl:stylesheet>
