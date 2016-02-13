use crossbeam;

use libimagstore::store::Entry;

use asciidoc::check::is_asciidoc;
use bbcode::check::is_bbcode;
use commonmark::check::is_commonmark;
use latex::check::is_latex;
use markdown::check::is_markdown;
use restructuredtext::check::is_restructuredtext;
use textile::check::is_textile;

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

