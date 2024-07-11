//! Experimental crate for writing wgsl in rust!
//!
//! # The `wgsl!` macro
//!
//! The `wgsl!` macro converts normal rust tokens into a wgsl `&'static str`, similar to [`stringify!`].
//! This also validates the wgsl string using [`naga`]. Errors will be reported with
//! the correct span.
//!
//! ```
//! # use wgsl_ln::wgsl;
//! pub static MANHATTAN_DISTANCE: &str = wgsl!(
//!     fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
//!         return abs(a.x - b.x) + abs(a.y - b.y);
//!     }
//! );
//! ```
//!
//! Most errors can be caught at compile time.
//!
//! ```compile_fail
//! # use wgsl_ln::wgsl;
//! pub static MANHATTAN_DISTANCE: &str = wgsl!(
//!     fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
//!         // not allowed in wgsl
//!         abs(a.x - b.x) + abs(a.y - b.y)
//!     }
//! );
//! ```
//!
//! # The `#[wgsl_export(name)]` macro
//!
//! Export a wgsl item (function, struct, etc)
//! via `wgsl_export`. Must have the same `name` as the exported item.
//!
//! ```
//! # use wgsl_ln::{wgsl, wgsl_export};
//! #[wgsl_export(manhattan_distance)]
//! pub static MANHATTAN_DISTANCE: &str = wgsl!(
//!     fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
//!         return abs(a.x - b.x) + abs(a.y - b.y);
//!     }
//! );
//! ```
//!
//! # Using an exported item
//!
//! ```
//! # use wgsl_ln::{wgsl, wgsl_export};
//! # #[wgsl_export(manhattan_distance)]
//! # pub static MANHATTAN_DISTANCE: &str = wgsl!(
//! #     fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
//! #         return abs(a.x - b.x) + abs(a.y - b.y);
//! #     }
//! # );
//! pub static MANHATTAN_DISTANCE_TIMES_FIVE: &str = wgsl!(
//!     fn manhattan_distance_times_five(a: vec2<f32>, b: vec2<f32>) -> f32 {
//!         return #manhattan_distance(a, b) * 5.0;
//!     }
//! );
//! ```
//!
//! `#manhattan_distance` copies the `manhattan_distance` function into the module,
//! making it usable. You can specify multiple instances of `#manhattan_distance`
//! or omit the `#` in later usages.
//!
//! * Note compile time checks still work.
//!
//! ```compile_fail
//! # use wgsl_ln::{wgsl, wgsl_export};
//! # #[wgsl_export(manhattan_distance)]
//! # pub static MANHATTAN_DISTANCE: &str = wgsl!(
//! #     fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
//! #         return abs(a.x - b.x) + abs(a.y - b.y);
//! #     }
//! # );
//! pub static MANHATTAN_DISTANCE_TIMES_FIVE: &str = wgsl!(
//!     fn manhattan_distance_times_five(a: vec2<f32>, b: vec2<f32>) -> f32 {
//!         // missing comma
//!         return #manhattan_distance(a, b) * 5.0
//!     }
//! );
//! ```
//!
//! # Ok what's actually going on?
//!
//! `wgsl_export` creates a `macro_rules!` macro that pastes itself into the `wgsl!` macro.
//! The macro is `#[doc(hidden)]` and available in the crate root,
//! i.e. `crate::__paste_wgsl_manhattan_distance!`.
//!
//! You don't need to import anything to use items defined in your crate, for other crates,
//! you might want to blanket import the crate root.
//!
//! ```
//! # /*
//! mod my_shaders {
//!     pub use external_shader_defs::*;
//!
//!     pub static MAGIC: &str = wgsl!(
//!         fn magic() -> f32 {
//!             return #magic_number();
//!         }
//!     )
//! }
//! pub use my_shaders::MAGIC;
//! # */
//! ```
//!
//! # `naga_oil` support
//!
//! Enable the `naga_oil` feature to enable limited `naga_oil` support:
//!
//! * Treat `#preprocessor_macro_name` as tokens instead of imports.
//!     * `#define_import_path`
//!     * `#import`
//!     * `#if`
//!     * `#ifdef`
//!     * `#ifndef`
//!     * `#else`
//!     * `#endif`
//!
//! These values can no longer be imported.
//!
//! * Checks will be disabled when naga_oil preprocessor macros are detected.
//!

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{
    token_stream::IntoIter, Delimiter, Group, Ident, Spacing, Span, TokenStream, TokenTree,
};
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};

/// Converts normal rust tokens into a wgsl `&'static str`, similar to [`stringify!`].
/// This also validates the wgsl string using [`naga`]. Errors will be reported with
/// the correct span.
///
/// ```
/// # use wgsl_ln::wgsl;
/// pub static MANHATTAN_DISTANCE: &str = wgsl!(
///     fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
///         return abs(a.x - b.x) + abs(a.y - b.y);
///     }
/// );
/// ```
///
/// To import an exported item, use the `#name` syntax. See crate level documentation for details.
///
/// ```
/// # use wgsl_ln::{wgsl, wgsl_export};
/// # #[wgsl_export(manhattan_distance)]
/// # pub static MANHATTAN_DISTANCE: &str = wgsl!(
/// #     fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
/// #         return abs(a.x - b.x) + abs(a.y - b.y);
/// #     }
/// # );
/// pub static MANHATTAN_DISTANCE_SQUARED: &str = wgsl!(
///     fn manhattan_distance_squared(a: vec2<f32>, b: vec2<f32>) -> f32 {
///         return #manhattan_distance(a, b) * manhattan_distance(a, b);
///     }
/// );
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn wgsl(stream: TokenStream1) -> TokenStream1 {
    wgsl2(stream.into()).into()
}

fn open(d: Delimiter) -> char {
    match d {
        Delimiter::Parenthesis => '(',
        Delimiter::Brace => '{',
        Delimiter::Bracket => '[',
        Delimiter::None => ' ',
    }
}

fn close(d: Delimiter) -> char {
    match d {
        Delimiter::Parenthesis => ')',
        Delimiter::Brace => '}',
        Delimiter::Bracket => ']',
        Delimiter::None => ' ',
    }
}

fn to_wgsl_string(
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
                    // bend over backwards for `naga_oil` :p
                    match string.pop() {
                        Some(' ') => (),
                        Some(c) => string.push(c),
                        None => (),
                    }
                    string.push(p.as_char());
                    uses_naga_oil = true;
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

fn sanitize_remaining(stream: IntoIter, ident: &Ident, items: &mut Vec<TokenTree>) {
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

fn is_naga_oil_name(name: &Ident) -> bool {
    name == "define_import_path"
        || name == "import"
        || name == "if"
        || name == "ifdef"
        || name == "ifndef"
        || name == "else"
        || name == "endif"
}

/// Find the first instance of `#ident` and rewrite the macro as `__paste!(wgsl!())`.
fn sanitize(stream: TokenStream) -> (TokenStream, Option<Ident>) {
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
            #[cfg(feature = "naga_oil")]
            TokenTree::Ident(ident) if last_is_hash && is_naga_oil_name(&ident) => {
                last_is_hash = false;
                result.push(TokenTree::Punct(proc_macro2::Punct::new(
                    '#',
                    Spacing::Joint,
                )));
                result.push(TokenTree::Ident(ident.clone()));
            }
            TokenTree::Ident(ident) if last_is_hash => {
                result.push(TokenTree::Ident(ident.clone()));
                sanitize_remaining(iter, &ident, &mut result);
                return (TokenStream::from_iter(result), Some(ident));
            }
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

fn wgsl2(stream: TokenStream) -> TokenStream {
    let (stream, pastes) = sanitize(stream);
    if let Some(paste) = pastes {
        let paste = format_ident!("__wgsl_paste_{}", paste);
        return quote! {{use crate::*; #paste!(wgsl!(#stream))}};
    }
    let mut spans = Vec::new();
    let mut source = String::new();
    #[allow(unused_variables)]
    let uses_naga_oil = to_wgsl_string(stream, &mut spans, &mut source);
    #[cfg(feature = "naga_oil")]
    if uses_naga_oil {
        return quote! {#source};
    }
    match naga::front::wgsl::parse_str(&source) {
        Ok(_) => quote! {#source},
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

/// Export a wgsl item (function, struct, etc).
///
/// Must have the same `name` as the exported item.
///
/// ```
/// # use wgsl_ln::{wgsl, wgsl_export};
/// #[wgsl_export(manhattan_distance)]
/// pub static MANHATTAN_DISTANCE: &str = wgsl!(
///     fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
///         return abs(a.x - b.x) + abs(a.y - b.y);
///     }
/// );
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn wgsl_export(attr: TokenStream1, stream: TokenStream1) -> TokenStream1 {
    wgsl_export2(attr.into(), stream.into()).into()
}

fn wgsl_export2(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

/// Paste and avoid duplicates.
#[doc(hidden)]
#[proc_macro]
#[proc_macro_error]
pub fn __wgsl_paste(stream: TokenStream1) -> TokenStream1 {
    __wgsl_paste2(stream.into()).into()
}

fn __wgsl_paste2(stream: TokenStream) -> TokenStream {
    let mut iter = stream.into_iter();
    let Some(TokenTree::Ident(definition)) = iter.next() else {
        abort!(
            Span::call_site(),
            "Expected `__wgsl_paste!($definition [$($defined)*] {to_be_pasted} $($tt)*)`!"
        )
    };
    let Some(TokenTree::Group(pasted)) = iter.next() else {
        abort!(
            Span::call_site(),
            "Expected `__wgsl_paste!($definition [$($defined)*] {to_be_pasted} $($tt)*)`!"
        )
    };
    let pasted = pasted.stream();
    match iter.next() {
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
        other => {
            let tokens: TokenStream = iter.collect();
            quote! {
                ::wgsl_ln::wgsl!([#definition] #pasted #other #tokens)
            }
        }
    }
}
