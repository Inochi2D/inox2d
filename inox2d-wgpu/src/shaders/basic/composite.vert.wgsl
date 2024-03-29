struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texUVs: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) verts: vec2<f32>,
    @location(1) uvs: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4(verts, 0.0, 1.0);
    out.position.y = -out.position.y;

    out.texUVs = uvs;
    return out;
}
