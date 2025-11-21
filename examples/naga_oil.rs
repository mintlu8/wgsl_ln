#[allow(unused)]
use wgsl_ln::{wgsl, wgsl_export};

#[wgsl_export(Vertex)]
pub static VERTEX: &str = wgsl!(
    #import awesome_game_engine::Vertex;
);

#[wgsl_export(VertexOutput)]
pub static VERTEX_OUT: &str = wgsl!(
    #import awesome_game_engine::{VertexOut}
);

pub static VERTEX_SHADER: &str = wgsl!(
    @vertex
    fn vertex_shader(vertex: $Vertex) -> $VertexOutput {
        var out: $VertexOutput;
        out.position = vec4(vertex.position, 0.0);
        return out;
    }
);

pub fn main() {
    println!("{}", VERTEX_SHADER);
}

