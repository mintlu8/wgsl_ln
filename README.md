# wgsl_ln

[![Crates.io](https://img.shields.io/crates/v/wgsl_ln.svg)](https://crates.io/crates/wgsl_ln)
[![Docs](https://docs.rs/wgsl_ln/badge.svg)](https://docs.rs/wgsl_ln/latest/wgsl_ln/)

Experimental crate for writing wgsl in rust!

## The `wgsl!` macro

The `wgsl!` macro converts normal rust tokens into a wgsl `&'static str`, similar to [`stringify!`].
This also validates the wgsl string using [`naga`]. Errors will be reported with
the correct span.

```rust
pub static MANHATTAN_DISTANCE: &str = wgsl!(
    fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
        return abs(a.x - b.x) + abs(a.y - b.y);
    }
);
```

Most errors can be caught at compile time.

```rust
pub static MANHATTAN_DISTANCE: &str = wgsl!(
    fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
        // not allowed in wgsl
        abs(a.x - b.x) + abs(a.y - b.y)
    }
);
```

## The `#[wgsl_export(name)]` macro

Export a wgsl item (function, struct, etc)
via `wgsl_export`. Must have the same `name` as the exported item.

```rust
#[wgsl_export(manhattan_distance)]
pub static MANHATTAN_DISTANCE: &str = wgsl!(
    fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
        return abs(a.x - b.x) + abs(a.y - b.y);
    }
);
```

## Using an exported item

```rust
pub static MANHATTAN_DISTANCE_TIMES_FIVE: &str = wgsl!(
    fn manhattan_distance_times_five(a: vec2<f32>, b: vec2<f32>) -> f32 {
        return #manhattan_distance(a, b) * 5.0;
    }
);
```

`#manhattan_distance` copies the `manhattan_distance` function into the module,
making it usable. You can specify multiple instances of `#manhattan_distance`
or omit the `#` in later usages.

## Ok what's actually going on?

`wgsl_export` creates a `macro_rules!` macro that pastes itself into the `wgsl!` macro.
The macro is `#[doc(hidden)]` and available in the crate root,
i.e. `crate::__paste_wgsl_manhattan_distance!`.

You don't need to import anything to use items defined in your crate, for other crates,
you might want to blanket import the crate root.

```rust
mod my_shaders {
    pub use external_shader_defs::*;

    pub static MAGIC: &str = wgsl!(
        fn magic() -> f32 {
            return #magic_number();
        }
    )
}
pub use my_shaders::MAGIC;
```

## `naga_oil` support

Enable the `naga_oil` feature to enable limited `naga_oil` support:

* Treat `#preprocessor_macro_name` as tokens instead of imports.
  * `#define_import_path`
  * `#import`
  * `#if`
  * `#ifdef`
  * `#ifndef`
  * `#else`
  * `#endif`

These values can on longer be imported.

* Checks will be disabled when naga_oil preprocessor macros are detected.

## License

License under either of

Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)
at your option.

## Contribution

Contributions are welcome!

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
