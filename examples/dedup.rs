use wgsl_ln::{wgsl, wgsl_export};

#[wgsl_export(sin_cos)]
pub static SIN_COS: &str = wgsl!(
    fn sin_cos(v: f32) -> vec2<f32> {
        return vec2(sin(v), cos(v));
    }
);

#[wgsl_export(sin_cos2)]
pub static SIN_COS_SQUARED: &str = wgsl!(
    fn sin_cos2(v: f32) -> f32 {
        return #sin_cos(v) * #sin_cos(v);
    }
);

pub static SIN_COS_SQUARED_PLUS_SIN_COS_1: &str = wgsl!(
    fn a(v: f32) -> f32 {
        return #sin_cos2(v) * #sin_cos(v);
    }
);

pub static SIN_COS_SQUARED_PLUS_SIN_COS_2: &str = wgsl!(
    fn a(v: f32) -> f32 {
        return #sin_cos(v) * #sin_cos2(v);
    }
);

pub fn main() {
    println!("{}", SIN_COS_SQUARED_PLUS_SIN_COS_1);
    println!("{}", SIN_COS_SQUARED_PLUS_SIN_COS_2);
}
