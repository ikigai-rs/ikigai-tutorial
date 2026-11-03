<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:r="http://www.w3.org/2005/sparql-results#" exclude-result-prefixes="r">
  <!-- SPARQL results (XML) of river.rq, since.rq, in.rq, from.rq or search.rq, as an HTML page.
       The summary is the feed's HTML; it is shown as TEXT (escaped), never spliced as markup.
       The viewport line lets the page reflow on a narrow screen (WCAG 1.4.10). The queries call
       an item with no title "(untitled)"; repeated, that says nothing about where a link goes
       (WCAG 2.4.4), so the link says the item's feed and day instead. -->
  <xsl:output method="html"/>
  <xsl:template match="/">
    <html lang="en">
      <head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/><title>River</title></head>
      <body>
        <h1>River</h1>
        <ol>
          <xsl:for-each select="/r:sparql/r:results/r:result">
            <li>
              <xsl:choose>
                <xsl:when test="r:binding[@name='title']/r:literal = '(untitled)'">
                  <a href="{r:binding[@name='link']/r:literal}"><xsl:value-of select="concat('Untitled item from ', r:binding[@name='feedTitle']/r:literal, ', ', substring(r:binding[@name='date']/r:literal, 1, 10))"/></a>
                </xsl:when>
                <xsl:otherwise>
                  <a href="{r:binding[@name='link']/r:literal}"><xsl:value-of select="r:binding[@name='title']/r:literal"/></a>
                  <xsl:value-of select="concat(' (', r:binding[@name='feedTitle']/r:literal, ', ', substring(r:binding[@name='date']/r:literal, 1, 10), ')')"/>
                </xsl:otherwise>
              </xsl:choose>
            </li>
          </xsl:for-each>
        </ol>
      </body>
    </html>
  </xsl:template>
</xsl:stylesheet>
