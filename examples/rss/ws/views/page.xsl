<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:r="http://www.w3.org/2005/sparql-results#" exclude-result-prefixes="r">
  <!-- SPARQL results (XML) of river.rq, since.rq, in.rq, from.rq or search.rq, as an HTML page.
       The summary is the feed's HTML; it is shown as TEXT (escaped), never spliced as markup. -->
  <xsl:output method="html"/>
  <xsl:template match="/">
    <html lang="en">
      <head><meta charset="utf-8"/><title>River</title></head>
      <body>
        <h1>River</h1>
        <ol>
          <xsl:for-each select="/r:sparql/r:results/r:result">
            <li>
              <a href="{r:binding[@name='link']/r:literal}"><xsl:value-of select="r:binding[@name='title']/r:literal"/></a>
              <xsl:value-of select="concat(' (', r:binding[@name='feedTitle']/r:literal, ', ', substring(r:binding[@name='date']/r:literal, 1, 10), ')')"/>
            </li>
          </xsl:for-each>
        </ol>
      </body>
    </html>
  </xsl:template>
</xsl:stylesheet>
