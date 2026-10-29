<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:r="http://www.w3.org/2005/sparql-results#" xmlns:dc="http://purl.org/dc/elements/1.1/"
  exclude-result-prefixes="r">
  <!-- The same answer as an RSS 2.0 feed: a search you can subscribe to. Each item keeps its own IRI as
       its guid, so a reader that saw it in the original feed recognizes it. The summary goes back as the
       escaped HTML it arrived as. A stylesheet cannot be told the search terms (no xsl:param reaches it),
       so the channel title is generic. The date goes out as dc:date (W3CDTF), not pubDate: nothing
       here formats an RFC 822 date. -->
  <xsl:output method="xml"/>
  <xsl:template match="/">
    <rss version="2.0">
      <channel>
        <title>River search</title>
        <link>https://example.com/river</link>
        <description>Items from the directory's feeds that matched a saved search.</description>
        <xsl:for-each select="/r:sparql/r:results/r:result">
          <item>
            <title><xsl:value-of select="r:binding[@name='title']/r:literal"/></title>
            <link><xsl:value-of select="r:binding[@name='link']/r:literal"/></link>
            <guid><xsl:value-of select="r:binding[@name='item']/r:uri"/></guid>
            <source url="{r:binding[@name='feed']/r:uri}"><xsl:value-of select="r:binding[@name='feedTitle']/r:literal"/></source>
            <xsl:if test="r:binding[@name='summary']">
              <description><xsl:value-of select="r:binding[@name='summary']/r:literal"/></description>
            </xsl:if>
            <xsl:if test="r:binding[@name='date']">
              <dc:date><xsl:value-of select="r:binding[@name='date']/r:literal"/></dc:date>
            </xsl:if>
          </item>
        </xsl:for-each>
      </channel>
    </rss>
  </xsl:template>
</xsl:stylesheet>
