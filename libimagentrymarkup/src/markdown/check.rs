use hoedown::Html;
use hoedown::Markdown;
use hoedown::renderer::html::Flags;
use hoedown::renderer::Render;

use libimagstore::store::Entry;

pub fn is_markdown(e: &Entry) -> bool {
    let md = Markdown::new(&e.get_content()[..]);
    Html::new(Flags::empty(), 0).render(&md).to_str().is_ok()
}

