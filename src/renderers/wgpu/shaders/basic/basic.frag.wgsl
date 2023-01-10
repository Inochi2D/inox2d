struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texUVs: vec2<f32>,
};

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) emissive: vec4<f32>,
    @location(2) bump: vec4<f32>,
};

struct Uniform {
    opacity: f32,
    multColor: vec3<f32>,
    screenColor: vec3<f32>,
    emissionStrength: f32,
};

@group(0) @binding(0)
var<uniform> unif: Uniform;

@group(1) @binding(0)
var albedo : texture_2d<f32>;
@group(1) @binding(1)
var albedoSamp : sampler;

@group(2) @binding(0)
var emissive : texture_2d<f32>;
@group(2) @binding(1)
var emissiveSamp : sampler;

@group(3) @binding(0)
var bump : texture_2d<f32>;
@group(3) @binding(1)
var bumpSamp : sampler;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    // Sample texture
    let texColor = textureSample(albedo, albedoSamp, in.texUVs);

    // Screen color math
    let screenOut = vec3(1.0) - ((vec3(1.0) - (texColor.xyz)) *
                            (vec3(1.0) - (unif.screenColor * texColor.a)));

    // Multiply color math + opacity application.
    out.albedo = vec4(screenOut.xyz, texColor.a) * vec4(unif.multColor.xyz, 1) * unif.opacity;

    // Emissive
    out.emissive = vec4(textureSample(emissive, emissiveSamp, in.texUVs).xyz * unif.emissionStrength, 1) * out.albedo.a;

    // Bumpmap
    out.bump = vec4(textureSample(bump, bumpSamp, in.texUVs).xyz, 1) * out.albedo.a;

    return out;
}
