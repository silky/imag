## Wiki {#sec:modules:wiki}

The Wiki module implements simple wiki functionality, supporting:

* Pages
* Subpages
* Links to pages or external content
* Rendering to html from
  * markdown (major goal)
    * plain
    * through pandoc
  * textile (consideration)
  * reStructured Text (consideration)
  * including
    * Embedding of
      * images
      * videos
    * TOC-generating

Whereas the commandline interface offers the following features:

* Adding, editing and removing entries
* Adding, editing and removing of tags on entries
* Printing a tree/graph of dependencies between articles
  * on the terminal (tree)
  * optionally via graphviz (graph)
* rendering the complete or parts of the wiki as static html pages
  * start page is the TOC
  * include custom css
  * use custom templates
  * optionally generate
    * link-collection of all links to internal/external content

Maybe the latter point will be implemented as another binary.

