<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <!-- The directory as a list of names: one feed URL per line (the map `..` convention). -->
  <xsl:output method="text"/>
  <!-- ⚠ xrust 2.3 turns &#10; in an attribute into a space and drops whitespace-only xsl:text,
       so the newline is cut out of a variable that holds more than whitespace. -->
  <xsl:variable name="lf-holder">.
.</xsl:variable>
  <xsl:variable name="lf" select="substring($lf-holder, 2, 1)"/>
  <xsl:template match="/">
    <xsl:for-each select="/opml/body//outline">
      <xsl:if test="@type = 'rss' and @xmlUrl">
        <xsl:value-of select="concat(@xmlUrl, $lf)"/>
      </xsl:if>
    </xsl:for-each>
  </xsl:template>
</xsl:stylesheet>
