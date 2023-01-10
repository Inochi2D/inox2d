struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texUVs: vec2<f32>,
};

struct Uniform {
    tex: mat4x4<f32>,
    threshold: f32,
};

@group(0) @binding(0)
var<uniform> unif: Uniform;

@group(1) @binding(0)
var tex : texture_2d<f32>;
@group(1) @binding(1)
var samp : sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, samp, in.texUVs);
    if (color.a <= unif.threshold) {
        discard;
    }
    return vec4(1, 1, 1, 1);
}
