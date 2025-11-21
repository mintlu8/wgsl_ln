use proc_macro2::{Delimiter, Spacing, Span, TokenStream, TokenTree};

use crate::open_close::{close, open};
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
    for token in stream {
        match token {
            TokenTree::Group(g) if first && g.delimiter() == Delimiter::Bracket => (),
            TokenTree::Ident(i) => {
                spans.push((string.len(), i.span()));
                string.push_str(&i.to_string());
                string.push(' ');
            }
            TokenTree::Punct(p) => {
                spans.push((string.len(), p.span()));
                if p.as_char() == ';' {
                    string.push(p.as_char());
                    string.push('\n');
                } else if p.as_char() == '#' {
                    // new line and no spaces for naga_oil
                    string.push('\n');
                    string.push(p.as_char());
                    uses_naga_oil = true;
                } else if p.as_char() == ':' {
                    // bend over backwards for `naga_oil`
                    match string.pop() {
                        Some(' ') => (),
                        Some(c) => string.push(c),
                        None => (),
                    }
                    string.push(p.as_char());
                } else if p.spacing() == Spacing::Alone {
                    string.push(p.as_char());
                    string.push(' ');
                } else {
                    string.push(p.as_char());
                }
            }
            TokenTree::Literal(l) => {
                spans.push((string.len(), l.span()));
                string.push_str(&l.to_string());
                string.push(' ');
            }
            TokenTree::Group(g) => {
                spans.push((string.len(), g.delim_span().open()));
                string.push(open(g.delimiter()));
                if g.delimiter() == Delimiter::Brace {
                    string.push('\n')
                }
                uses_naga_oil |= to_wgsl_string(g.stream(), spans, string);
                spans.push((string.len(), g.delim_span().close()));
                string.push(close(g.delimiter()));
                if g.delimiter() == Delimiter::Brace {
                    string.push('\n')
                }
            }
        }
        first = false;
    }
    uses_naga_oil
}
