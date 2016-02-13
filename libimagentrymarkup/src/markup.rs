use std::io::Write;

use libimagstore::store::Entry;

use result::Result;

pub type HTML = String;
pub type Link = String;

pub trait IntoHtml {

    fn into_html(self) -> Result<HTML>;
    fn write_html<W: Write>(self, w: W) -> Result<()>;

}

pub trait LinkExtractor {

    fn links(&self) -> Vec<Link>;

    fn has_external_links(&self) -> bool;
    fn has_internal_links(&self) -> bool;

    fn has_link(&self, link: Link) -> bool {
        self.links().into_iter().any(|l| l == link)
    }

}

pub trait IsMarkupChecker {

    fn is_markup(e: &Entry) -> bool;

}

pub trait MarkupProcessor : IntoHtml + LinkExtractor + IsMarkupChecker {

    fn for_entry(e: &Entry) -> Result<Self>;

}

