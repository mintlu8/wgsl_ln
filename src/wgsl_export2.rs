use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro_error::abort;
use quote::{format_ident, quote};

pub fn wgsl_export2(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let Some(TokenTree::Ident(name)) = attr.into_iter().next() else {
        abort!(Span::call_site(), "Expected #[wgsl_export(name)]");
    };
    let mut wgsl_macro_ident = false;
    let mut exclamation_mark = false;
    let sealed = format_ident!("__sealed_{}", name);
    let mut paste = format_ident!("__wgsl_paste_{}", name);
    paste.set_span(name.span());
    for token in stream.clone() {
        match token {
            TokenTree::Ident(i) if i == "wgsl" => {
                wgsl_macro_ident = true;
                exclamation_mark = false;
            }
            TokenTree::Punct(p) if wgsl_macro_ident && p.as_char() == '!' => {
                exclamation_mark = true;
            }
            TokenTree::Group(g) if wgsl_macro_ident && exclamation_mark => {
                let source = g.stream();
                return quote! {
                    #[allow(non_snake_case)]
                    mod #sealed {
                        #[allow(non_snake_case)]
                        #[doc(hidden)]
                        #[macro_export]
                        macro_rules! #paste {
                            (wgsl!($($tt: tt)*)) => {
                                ::wgsl_ln::__wgsl_paste!(#name {#source} $($tt)*)
                            };
                        }
                    }
                    #stream
                };
            }
            _ => {
                wgsl_macro_ident = false;
                exclamation_mark = false;
            }
        }
    }
    abort!(Span::call_site(), "Expected wgsl! macro.");
}
