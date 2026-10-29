<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:r="http://www.w3.org/2005/sparql-results#">
  <!-- The same answer as a Markdown digest: one bullet per item, grouped by nothing, newest first. -->
  <xsl:output method="text"/>
  <!-- ⚠ xrust 2.3: see feeds.xsl for why the newline is cut out of a variable. -->
  <xsl:variable name="lf-holder">.
.</xsl:variable>
  <xsl:variable name="lf" select="substring($lf-holder, 2, 1)"/>
  <xsl:template match="/">
    <xsl:value-of select="concat('# Digest', $lf, $lf)"/>
    <xsl:for-each select="/r:sparql/r:results/r:result">
      <xsl:value-of select="concat('- ', substring(r:binding[@name='date']/r:literal, 1, 10), ' [',
        r:binding[@name='title']/r:literal, '](', r:binding[@name='link']/r:literal, ') from ',
        r:binding[@name='feedTitle']/r:literal, $lf)"/>
    </xsl:for-each>
  </xsl:template>
</xsl:stylesheet>
