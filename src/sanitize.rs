use proc_macro2::{token_stream::IntoIter, Delimiter, Group, Ident, TokenStream, TokenTree};
/// Find the first instance of `#ident` and rewrite the macro as `__paste!(wgsl!())`.
pub fn sanitize(stream: TokenStream) -> (TokenStream, Option<Ident>) {
    let mut result = Vec::new();
    let mut last_is_hash = false;
    let mut iter = stream.into_iter();
    let mut first = true;
    while let Some(tt) = iter.next() {
        match tt {
            // ifndef
            TokenTree::Group(g) if first && g.delimiter() == Delimiter::Bracket => {
                result.push(TokenTree::Group(g));
            }
            TokenTree::Punct(p) if p.as_char() == '#' => {
                last_is_hash = true;
            }
            // if is a naga_oil definition, write `#def`
            #[cfg(feature = "naga_oil")]
            TokenTree::Ident(ident) if last_is_hash && is_naga_oil_name(&ident) => {
                last_is_hash = false;
                result.push(TokenTree::Punct(proc_macro2::Punct::new(
                    '#',
                    Spacing::Joint,
                )));
                result.push(TokenTree::Ident(ident.clone()));
            }
            // If # ident, import it and remove duplicated `#`s.
            TokenTree::Ident(ident) if last_is_hash => {
                result.push(TokenTree::Ident(ident.clone()));
                sanitize_remaining(iter, &ident, &mut result);
                return (TokenStream::from_iter(result), Some(ident));
            }
            // Recursively look for `#`s.
            TokenTree::Group(g) => {
                let delim = g.delimiter();
                let (stream, ident) = sanitize(g.stream());
                result.push(TokenTree::Group(Group::new(delim, stream)));
                if let Some(ident) = ident {
                    sanitize_remaining(iter, &ident, &mut result);
                    return (TokenStream::from_iter(result), Some(ident));
                }
            }
            tt => {
                last_is_hash = false;
                result.push(tt)
            }
        }
        first = false
    }
    (TokenStream::from_iter(result), None)
}

/// Remove duplicated `#`s from `# ident`s.
pub fn sanitize_remaining(stream: IntoIter, ident: &Ident, items: &mut Vec<TokenTree>) {
    let mut last_is_hash = false;
    for tt in stream {
        match &tt {
            // ifndef
            TokenTree::Punct(p) if p.as_char() == '#' => {
                last_is_hash = true;
                items.push(tt)
            }
            TokenTree::Ident(i) if last_is_hash && i == ident => {
                last_is_hash = false;
                let _ = items.pop();
                items.push(tt)
            }
            TokenTree::Group(g) => {
                last_is_hash = false;
                let mut stream = Vec::new();
                sanitize_remaining(g.stream().into_iter(), ident, &mut stream);
                items.push(TokenTree::Group(Group::new(
                    g.delimiter(),
                    TokenStream::from_iter(stream),
                )))
            }
            _ => {
                last_is_hash = false;
                items.push(tt)
            }
        }
    }
}

#[allow(dead_code)]
fn is_naga_oil_name(name: &Ident) -> bool {
    name == "define_import_path"
        || name == "import"
        || name == "if"
        || name == "ifdef"
        || name == "ifndef"
        || name == "else"
        || name == "endif"
}
