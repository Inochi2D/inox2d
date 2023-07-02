struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texUVs: vec2<f32>,
};

struct Uniform {
    opacity: f32,
    multColor: vec3<f32>,
    screenColor: vec3<f32>,
    emissionStrength: f32,
    offset: vec2<f32>,
    mvp: mat4x4<f32>,
};

@group(0) @binding(1)
var<uniform> unif: Uniform;

@vertex
fn vs_main(
    @location(0) verts: vec2<f32>,
    @location(1) uvs: vec2<f32>,
    @location(2) deform: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = unif.mvp * vec4(verts + deform - unif.offset, 0.0, 1.0);

    out.texUVs = uvs;
    return out;
}
