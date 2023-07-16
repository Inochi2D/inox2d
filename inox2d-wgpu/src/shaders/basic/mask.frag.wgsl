struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texUVs: vec2<f32>,
};

@group(1) @binding(0)
var albedo : texture_2d<f32>;
@group(1) @binding(1)
var albedoSamp : sampler;

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let texColor = textureSample(albedo, albedoSamp, in.texUVs);
    if (texColor.a <= 0.5) {
        discard;
    }

    return vec4(0.0, 0.0, 0.0, 1.0);
}