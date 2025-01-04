use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use proc_macro_error::abort;
use quote::quote;
pub fn __wgsl_paste2(stream: TokenStream) -> TokenStream {
    let mut iter = stream.into_iter();
    let Some(TokenTree::Ident(definition)) = iter.next() else {
        abort!(
            Span::call_site(),
            "Expected `__wgsl_paste!($definition {to_be_pasted} $([$($defined)*])? $($tt)*)`!"
        )
    };
    let Some(TokenTree::Group(pasted)) = iter.next() else {
        abort!(
            Span::call_site(),
            "Expected `__wgsl_paste!($definition {to_be_pasted} $([$($defined)*])? $($tt)*)`!"
        )
    };
    let pasted = pasted.stream();
    match iter.next() {
        // If some values are defined, check if this item has been defined.
        // If defined, skip, if not defined, paste and define this item.
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
            let tokens: TokenStream = iter.collect();
            let mut found = false;
            let names: Vec<_> = g
                .stream()
                .into_iter()
                .filter(|x| match x {
                    TokenTree::Ident(i) => {
                        if i == &definition {
                            found = true;
                        }
                        true
                    }
                    _ => false,
                })
                .collect();
            if found {
                quote!(::wgsl_ln::wgsl!([#(#names)*] #tokens))
            } else {
                quote!(::wgsl_ln::wgsl!([#(#names)* #definition] #pasted #tokens))
            }
        }
        // If no values defined, paste and define this item.
        other => {
            let tokens: TokenStream = iter.collect();
            quote! {
                ::wgsl_ln::wgsl!([#definition] #pasted #other #tokens)
            }
        }
    }
}
