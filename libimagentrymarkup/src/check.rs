use crossbeam;

use libimagstore::store::Entry;

pub fn is_parsable(e: &Entry) -> bool {
    crossbeam::scope(|sc| {
        let is_markdown         = sc.spawn(|| is_markdown(e));
        let is_commonmark       = sc.spawn(|| is_commonmark(e));
        let is_textile          = sc.spawn(|| is_textile(e));
        let is_latex            = sc.spawn(|| is_latex(e));
        let is_restructuredtext = sc.spawn(|| is_restructuredtext(e));
        let is_asciidoc         = sc.spawn(|| is_asciidoc(e));
        let is_bbcode           = sc.spawn(|| is_bbcode(e));

        is_markdown.join()         ||
        is_commonmark.join()       ||
        is_textile.join()          ||
        is_latex.join()            ||
        is_restructuredtext.join() ||
        is_asciidoc.join()         ||
        is_bbcode.join()
    })
}

pub fn is_markdown(e: &Entry) -> bool {
    use hoedown::Html;
    use hoedown::Markdown;
    use hoedown::renderer::html::Flags;
    use hoedown::renderer::Render;

    let md = Markdown::new(&e.get_content()[..]);
    Html::new(Flags::empty(), 0).render(&md).to_str().is_ok()
}

pub fn is_commonmark(e: &Entry) -> bool {
    false
}

pub fn is_textile(e: &Entry) -> bool {
    false
}

pub fn is_latex(e: &Entry) -> bool {
    false
}

pub fn is_restructuredtext(e: &Entry) -> bool {
    false
}

pub fn is_asciidoc(e: &Entry) -> bool {
    false
}

pub fn is_bbcode(e: &Entry) -> bool {
    false
}

