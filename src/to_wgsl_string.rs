use proc_macro2::{Delimiter, Spacing, Span, TokenStream, TokenTree};

use crate::open_close::{close, open};

trait StringMutExt {
    fn trim_space(&mut self);
    fn consume_prev(&mut self, c: char);
}

impl StringMutExt for String {
    fn trim_space(&mut self) {
        if self.ends_with(' ') {
            self.pop();
        }
    }

    fn consume_prev(&mut self, c: char) {
        if matches!(c, ':' | ',' | '.' | ';') {
            self.trim_space();
        }
    }
}

fn consume_post(c: char) -> bool {
    matches!(c, ':' | '.' | '@')
}

/// Convert to `wgsl` and return if we think this uses `naga_oil` or not.
/// This has to format in a certain way to make `naga_oil` work:
///
/// * Linebreaks after `;` and `}`.
/// * Linebreaks before `#`.
/// * No space after `#`.
/// * No spaces before and after `:`.
pub fn to_wgsl_string(
    stream: TokenStream,
    spans: &mut Vec<(usize, Span)>,
    string: &mut String,
) -> bool {
    let mut first = true;
    let mut uses_naga_oil = false;
    let mut iter = stream.into_iter().peekable();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Group(g) if first && g.delimiter() == Delimiter::Bracket => (),
            TokenTree::Ident(i) => {
                spans.push((string.len(), i.span()));
                string.push_str(&i.to_string());
                string.push(' ');
            }
            TokenTree::Punct(p) => {
                spans.push((string.len(), p.span()));
                string.consume_prev(p.as_char());
                if p.as_char() == ';' {
                    string.push(p.as_char());
                    string.push('\n');
                } else if p.as_char() == '#' {
                    uses_naga_oil = true;
                    match iter.peek() {
                        // Make sure `#{MATERIAL_BIND_GROUP}` stays in one line.
                        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                            string.push_str("#{");
                            to_wgsl_string(g.stream(), spans, string);
                            iter.next();
                            string.trim_space();
                            string.push_str("} ");
                        },
                        _ => {
                            // new line and no spaces for naga_oil
                            string.push('\n');
                            string.push(p.as_char());
                        }
                    }
                } else if consume_post(p.as_char()) || p.spacing() == Spacing::Joint {
                    string.push(p.as_char());
                } else {
                    string.push(p.as_char());
                    string.push(' ');
                }
            }
            TokenTree::Literal(l) => {
                spans.push((string.len(), l.span()));
                string.push_str(&l.to_string());
                string.push(' ');
            }
            TokenTree::Group(g) => {
                if g.delimiter() == Delimiter::Bracket || g.delimiter() == Delimiter::Parenthesis {
                    string.trim_space();
                }
                spans.push((string.len(), g.delim_span().open()));
                string.push(open(g.delimiter()));
                if g.delimiter() == Delimiter::Brace {
                    string.push('\n')
                }
                uses_naga_oil |= to_wgsl_string(g.stream(), spans, string);
                if string.ends_with(' ') {
                    string.pop();
                }
                spans.push((string.len(), g.delim_span().close()));
                string.push(close(g.delimiter()));
                if g.delimiter() == Delimiter::Brace {
                    string.push('\n')
                } else {
                    string.push(' ');
                }
            }
        }
        first = false;
    }
    uses_naga_oil
}
