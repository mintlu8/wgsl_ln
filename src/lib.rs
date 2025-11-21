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
//! # Using an exported item with `$item`
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
//!         return $manhattan_distance(a, b) * 5.0;
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
//!         // missing semicolon
//!         return $manhattan_distance(a, b) * 5.0
//!     }
//! );
//! ```
//!
//! # Ok what's actually going on?
//!
//! `wgsl_export` creates a `macro_rules!` macro that pastes itself into the `wgsl!` macro.
//! The macro is `#[doc(hidden)]` and available in the crate root,
//! i.e. `crate::__wgsl_paste_manhattan_distance!`.
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
//!             return $magic_number();
//!         }
//!     )
//! }
//! pub use my_shaders::MAGIC;
//! # */
//! ```
//!
//! # `naga_oil` support
//!
//! * If a `#` is detected, we will disable certain validations.
//! * All `#` starting statements has to end with either `;` or `}` to force a line break.
//!

use proc_macro::TokenStream as TokenStream1;
use proc_macro_error::{proc_macro_error, set_dummy};
use quote::quote;
mod pasting;
mod open_close;
mod sanitize;
mod to_wgsl_string;
mod wgsl_macro;
mod wgsl_export_macro;

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
/// To import an exported item, use the `$name` syntax. See crate level documentation for details.
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
///         // Using one `$` on the first item is also fine, we will deduplicate items.
///         return $manhattan_distance(a, b) * $manhattan_distance(a, b);
///     }
/// );
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn wgsl(stream: TokenStream1) -> TokenStream1 {
    set_dummy(quote! {""});
    wgsl_macro::wgsl_macro(stream.into()).into()
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
    set_dummy(quote! {""});

    wgsl_export_macro::wgsl_export_macro(attr.into(), stream.into()).into()
}

/// Paste and avoid duplicates.
#[doc(hidden)]
#[proc_macro]
#[proc_macro_error]
pub fn __wgsl_paste(stream: TokenStream1) -> TokenStream1 {
    set_dummy(quote! {""});

    pasting::wgsl_paste(stream.into()).into()
}
