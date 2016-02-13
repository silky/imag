use std::io::Write;

use libimagstore::store::Entry;

use markup::{IntoHtml, LinkExtractor, IsMarkupChecker, MarkupProcessor, Link, HTML};
use result::Result;

pub struct Textile {
    entry_content: String,
}

impl IntoHtml for Textile {

    fn into_html(self) -> Result<HTML> {
        unimplemented!()
    }

    fn write_html<W: Write>(self, w: W) -> Result<()> {
        unimplemented!()
    }

}

impl LinkExtractor for Textile {

    fn links(&self) -> Vec<Link> {
        unimplemented!()
    }

    fn has_external_links(&self) -> bool {
        unimplemented!()
    }

    fn has_internal_links(&self) -> bool {
        unimplemented!()
    }

}

impl IsMarkupChecker for Textile {

    fn is_markup(e: &Entry) -> bool {
        unimplemented!()
    }

}

impl MarkupProcessor for Textile {

    fn for_entry(e: &Entry) -> Result<Textile> {
        unimplemented!()
    }

}

