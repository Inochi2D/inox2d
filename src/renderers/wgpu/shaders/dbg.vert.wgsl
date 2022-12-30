struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct Uniform {
    mvp: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> unif: Uniform;

@vertex
fn vs_main(
    @location(0) verts: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = unif.mvp * vec4(verts.x, verts.y, verts.z, 1);
    return out;
}