use crossbeam;

use libimagstore::store::Entry;

use asciidoc::AsciiDoc;
use bbcode::BBCode;
use commonmark::CommonMark;
use latex::Latex;
use markdown::Markdown;
use restructuredtext::RestructuredText;
use textile::Textile;

use markup::IsMarkupChecker;

pub fn is_parsable(e: &Entry) -> bool {
    crossbeam::scope(|sc| {
        let is_markdown         = sc.spawn(|| Markdown::is_markup(e));
        let is_commonmark       = sc.spawn(|| CommonMark::is_markup(e));
        let is_textile          = sc.spawn(|| Textile::is_markup(e));
        let is_latex            = sc.spawn(|| Latex::is_markup(e));
        let is_restructuredtext = sc.spawn(|| RestructuredText::is_markup(e));
        let is_asciidoc         = sc.spawn(|| AsciiDoc::is_markup(e));
        let is_bbcode           = sc.spawn(|| BBCode::is_markup(e));

        is_markdown.join()         ||
        is_commonmark.join()       ||
        is_textile.join()          ||
        is_latex.join()            ||
        is_restructuredtext.join() ||
        is_asciidoc.join()         ||
        is_bbcode.join()
    })
}

