<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:r="http://www.w3.org/2005/sparql-results#" exclude-result-prefixes="r">
  <!-- The feeds a search matched (search-feeds.rq), as an OPML a feed reader can import. -->
  <xsl:output method="xml"/>
  <xsl:template match="/">
    <opml version="2.0">
      <head><title>Feeds that matched</title></head>
      <body>
        <xsl:for-each select="/r:sparql/r:results/r:result">
          <outline type="rss" version="RSS" text="{r:binding[@name='feedTitle']/r:literal}"
            title="{r:binding[@name='feedTitle']/r:literal}" xmlUrl="{r:binding[@name='feed']/r:uri}"
            htmlUrl="{r:binding[@name='home']/r:literal}"/>
        </xsl:for-each>
      </body>
    </opml>
  </xsl:template>
</xsl:stylesheet>
