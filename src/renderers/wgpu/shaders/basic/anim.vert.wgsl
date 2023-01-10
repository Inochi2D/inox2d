struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texUVs: vec2<f32>,
};

struct Uniform {
    mvp: mat4x4<f32>,
    offset: vec2<f32>,
    splits: vec2<f32>,
    animation: f32,
    frame: f32,
};

@group(0) @binding(0)
var<uniform> unif: Uniform;

@vertex
fn vs_main(
    @location(0) verts: vec2<f32>,
    @location(1) uvs: vec2<f32>,
    @location(2) deform: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = unif.mvp * vec4(verts.x - unif.offset.x + deform.x,
                            verts.y - unif.offset.y + deform.y, 0, 1);
    out.texUVs = vec2((uvs.x / unif.splits.x) * unif.frame, (uvs.y / unif.splits.y) * unif.animation);
    return out;
}
