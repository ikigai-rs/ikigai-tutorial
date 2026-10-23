<?xml version="1.0" encoding="UTF-8"?>
<!-- The topology view: every door of every space, in the order resolution consults them
     (urn:kernel:topology). The first door that matches a name answers it. -->
<xsl:stylesheet version="1.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:ik="https://ikigai-rs.dev/ns#"
    xmlns:insp="urn:iki:tutorial:inspect:term#"
    exclude-result-prefixes="rdf ik insp">
<xsl:output method="html"/>
<xsl:template match="/">
<section id="view" class="inspect">
<h1>Topology</h1>
<p>The arrangement resolution walks: <xsl:value-of select="count(rdf:RDF/ik:Door)"/> doors. A name is answered by the first door, in this order, whose pattern matches it.</p>
<table class="inspect-table">
<caption>Doors, in resolution order</caption>
<thead><tr><th scope="col">Pattern</th><th scope="col">Match</th><th scope="col">Endpoint</th><th scope="col">Space</th></tr></thead>
<tbody>
<xsl:apply-templates select="rdf:RDF/ik:Door">
<xsl:sort select="insp:order"/>
</xsl:apply-templates>
</tbody>
</table>
</section>
</xsl:template>
<xsl:template match="ik:Door">
<tr>
<td><code><xsl:value-of select="ik:pattern"/></code></td>
<td><xsl:value-of select="ik:matchKind"/></td>
<td><xsl:value-of select="ik:endpointName"/></td>
<td><xsl:value-of select="insp:space"/></td>
</tr>
</xsl:template>
</xsl:stylesheet>
