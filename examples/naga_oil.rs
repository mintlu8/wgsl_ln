#[allow(unused)]
use wgsl_ln::{wgsl, wgsl_export};

#[cfg(feature = "naga_oil")]
#[wgsl_export(Vertex)]
pub static VERTEX: &str = wgsl!(
    #import awesome_game_engine::Vertex;
);

#[cfg(feature = "naga_oil")]
#[wgsl_export(VertexOutput)]
pub static VERTEX_OUT: &str = wgsl!(
    #import awesome_game_engine::{VertexOut}
);

#[cfg(feature = "naga_oil")]
pub static VERTEX_SHADER: &str = wgsl!(
    @vertex
    fn vertex_shader(vertex: #Vertex) -> #VertexOutput {
        var out: VertexOutput;
        out.position = vec4(vertex.position, 0.0);
        return out;
    }
);

#[cfg(feature = "naga_oil")]
pub fn main() {
    println!("{}", VERTEX_SHADER);
}

#[cfg(not(feature = "naga_oil"))]
pub fn main() {
    println!("Enable feature `naga_oil` or this will fail to compile.");
}
