use wgsl_ln::{wgsl, wgsl_export};

#[wgsl_export(Vertex)]
pub static VERTEX: &str = wgsl!(
    struct Vertex {
        @builtin(instance_index) instance_index: u32,
        @location(0) position: vec3<f32>,
        @location(1) normal: vec3<f32>,
        @location(2) uv: vec2<f32>,
        @location(3) uv_b: vec2<f32>,
        @location(4) tangent: vec4<f32>,
        @location(5) color: vec4<f32>,
        @location(6) @interpolate(flat) joint_indices: vec4<u32>,
        @location(7) joint_weights: vec4<f32>,
        @builtin(vertex_index) index: u32
    }
);

#[wgsl_export(VertexOutput)]
pub static VERTEX_OUT: &str = wgsl!(
    struct VertexOutput {
        @builtin(position) position: vec4<f32>,
        @location(0) world_position: vec4<f32>,
        @location(1) world_normal: vec3<f32>,
        @location(2) uv: vec2<f32>,
        @location(3) uv_b: vec2<f32>,
        @location(4) world_tangent: vec4<f32>,
        @location(5) color: vec4<f32>,
        @location(6) @interpolate(flat) instance_index: u32
    }
);

pub static VERTEX_SHADER: &str = wgsl!(
    @vertex
    fn vertex_shader(vertex: #Vertex) -> #VertexOutput {
        var out: VertexOutput;
        out.position = vec4(vertex.position, 0.0);
        return out;
    }
);

pub fn main() {
    println!("{}", VERTEX_SHADER);
}
