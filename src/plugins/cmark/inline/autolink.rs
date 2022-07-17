//! Autolinks
//!
//! `<https://example.org>`
//!
//! <https://spec.commonmark.org/0.30/#autolinks>
use once_cell::sync::Lazy;
use regex::Regex;
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::inline::{InlineRule, InlineState, Text};

#[derive(Debug)]
pub struct Autolink {
    pub url: String,
}

impl NodeValue for Autolink {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs = node.attrs.clone();
        attrs.push(("href", self.url.clone()));

        fmt.open("a", &attrs);
        fmt.contents(&node.children);
        fmt.close("a");
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<AutolinkScanner>();
}

static AUTOLINK_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z][a-zA-Z0-9+.\-]{1,31}):([^<>\x00-\x20]*)$").unwrap()
});

static EMAIL_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)$").unwrap()
});

#[doc(hidden)]
pub struct AutolinkScanner;
impl InlineRule for AutolinkScanner {
    const MARKER: char = '&';

    fn run(state: &mut InlineState, silent: bool) -> bool {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next().unwrap() != '<' { return false; }

        let mut pos = state.pos + 2;

        loop {
            match chars.next() {
                Some('<') | None => return false,
                Some('>') => break,
                Some(x) => pos += x.len_utf8(),
            }
        }

        let url = &state.src[state.pos+1..pos-1];
        let is_autolink = AUTOLINK_RE.is_match(url);
        let is_email = EMAIL_RE.is_match(url);

        if !is_autolink && !is_email { return false; }

        let full_url = if is_autolink {
            (state.md.normalize_link)(url)
        } else {
            (state.md.normalize_link)(&("mailto:".to_owned() + url))
        };

        if !(state.md.validate_link)(&full_url) { return false; }

        if !silent {
            let content = (state.md.normalize_link_text)(url);

            let mut node = Node::new(Autolink { url: full_url });
            node.srcmap = state.get_map(state.pos, pos);

            let mut inner_node = Node::new(Text { content });
            inner_node.srcmap = state.get_map(state.pos + 1, pos - 1);

            node.children.push(inner_node);
            state.push(node);
        }

        state.pos = pos;
        true
    }
}
