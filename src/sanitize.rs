#[allow(unused)]
use proc_macro2::{
    token_stream::IntoIter, Delimiter, Group, Ident, Spacing, TokenStream, TokenTree,
};
/// Find the first instance of `$ident` and rewrite the macro as `__paste!(wgsl!())`.
pub fn sanitize(stream: TokenStream) -> (TokenStream, Option<Ident>) {
    let mut result = Vec::new();
    let mut external_ident = false;
    let mut iter = stream.into_iter();
    let mut first = true;
    while let Some(tt) = iter.next() {
        match tt {
            // ifndef
            TokenTree::Group(g) if first && g.delimiter() == Delimiter::Bracket => {
                result.push(TokenTree::Group(g));
            }
            TokenTree::Punct(p) if p.as_char() == '$' => {
                external_ident = true;
            }
            // If $ident, import it and remove duplicated `$`s.
            TokenTree::Ident(ident) if external_ident => {
                result.push(TokenTree::Ident(ident.clone()));
                sanitize_remaining(iter, &ident, &mut result);
                return (TokenStream::from_iter(result), Some(ident));
            }
            // Recursively look for `$`s.
            TokenTree::Group(g) => {
                external_ident = false;
                let delim = g.delimiter();
                let (stream, ident) = sanitize(g.stream());
                result.push(TokenTree::Group(Group::new(delim, stream)));
                if let Some(ident) = ident {
                    sanitize_remaining(iter, &ident, &mut result);
                    return (TokenStream::from_iter(result), Some(ident));
                }
            }
            tt => {
                external_ident = false;
                result.push(tt)
            }
        }
        first = false
    }
    (TokenStream::from_iter(result), None)
}

/// Remove duplicated `$`s from `$ident`s.
pub fn sanitize_remaining(stream: IntoIter, ident: &Ident, items: &mut Vec<TokenTree>) {
    let mut last_is_symbol = false;
    for tt in stream {
        match &tt {
            // ifndef
            TokenTree::Punct(p) if p.as_char() == '$' => {
                last_is_symbol = true;
                items.push(tt)
            }
            TokenTree::Ident(i) if last_is_symbol && i == ident => {
                last_is_symbol = false;
                let _ = items.pop();
                items.push(tt)
            }
            TokenTree::Group(g) => {
                last_is_symbol = false;
                let mut stream = Vec::new();
                sanitize_remaining(g.stream().into_iter(), ident, &mut stream);
                items.push(TokenTree::Group(Group::new(
                    g.delimiter(),
                    TokenStream::from_iter(stream),
                )))
            }
            _ => {
                last_is_symbol = false;
                items.push(tt)
            }
        }
    }
}
