use naga::valid::{Capabilities, ValidationFlags, Validator};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};

use crate::{sanitize::sanitize, to_wgsl_string::to_wgsl_string};

pub fn wgsl_macro(stream: TokenStream) -> TokenStream {
    let (stream, pastes) = sanitize(stream);
    if let Some(paste) = pastes {
        let paste = format_ident!("__wgsl_paste_{}", paste);
        return quote! {{use crate::*; #paste!(wgsl!(#stream))}};
    }
    let mut spans = Vec::new();
    let mut source = String::new();
    #[allow(unused_variables)]
    let uses_naga_oil = to_wgsl_string(stream, &mut spans, &mut source);
    if uses_naga_oil {
        return quote! {#source};
    }
    match naga::front::wgsl::parse_str(&source) {
        Ok(module) => {
            match Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module) {
                Ok(_) => quote! {#source},
                Err(e) => {
                    if let Some((span, _)) = e.spans().next() {
                        let location = span.location(&source);
                        let pos = match spans
                            .binary_search_by_key(&(location.offset as usize), |x| x.0)
                        {
                            Ok(x) => x,
                            Err(x) => x.saturating_sub(1),
                        };
                        abort!(spans[pos].1, "Wgsl Error: {}", e)
                    }
                    let e_str = e.to_string();
                    quote! {compile_error!(#e_str)}
                }
            }
        }
        Err(e) => {
            if let Some((span, _)) = e.labels().next() {
                let location = span.location(&source);
                let pos = match spans.binary_search_by_key(&(location.offset as usize), |x| x.0) {
                    Ok(x) => x,
                    Err(x) => x.saturating_sub(1),
                };
                abort!(spans[pos].1, "Wgsl Error: {}", e)
            }
            let e_str = e.to_string();
            quote! {compile_error!(#e_str)}
        }
    }
}
