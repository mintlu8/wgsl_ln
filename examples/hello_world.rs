use another_world::{WORLD_2, HELLO_2};
use the_third_world::WORLD_3;
use wgsl_ln::{wgsl, wgsl_export};

#[wgsl_export(hello)]
pub static HELLO: &str = wgsl!(
    fn hello(v: vec2<f32>) -> f32 {
        return (v.x + v.y) + 1.0;
    }
);

pub static WORLD: &str = wgsl!(
    fn world(v: vec4<f32>) -> f32 {
        return #hello(v.xy) + #hello(v.zw);
    }
);

mod another_world {
    use wgsl_ln::{wgsl, wgsl_export};


    #[wgsl_export(hello2)]
    pub static HELLO_2: &str = wgsl!(
        fn hello2(v: vec2<f32>) -> f32 {
            return #hello(v);
        }
    );

    pub static WORLD_2: &str = wgsl!(
        fn world(v: vec4<f32>) -> f32 {
            return #hello(v.xy) + #hello(v.zw);
        }
    );
}


mod the_third_world {
    use wgsl_ln::wgsl;

    pub static WORLD_3: &str = wgsl!(
        fn world(v: vec4<f32>) -> f32 {
            return #hello2(v.xy) + #hello(v.zw);
        }
    );
}

pub fn main() {
    println!("{}", HELLO);
    println!("{}", WORLD);
    println!("{}", HELLO_2);
    println!("{}", WORLD_2);
    println!("{}", WORLD_3);
}