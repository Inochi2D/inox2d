/*
    Copyright © 2020, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.

    Authors: Luna Nielsen
*/
#version 440
layout(location = 0) in vec2 texUVs;

layout(location = 0) out vec4 outAlbedo;
layout(location = 1) out vec4 outEmissive;
layout(location = 2) out vec4 outBump;

layout(set = 1, binding = 0) uniform Input {
  uniform vec3 ambientLight;
  uniform vec2 fbSize;

  uniform int LOD; // OLD DEFAULT: 2
  uniform int samples; // OLD DEFAULT: 25
} uni_in;

layout(set = 1, binding = 1) uniform texture2D albedo_tex;
layout(set = 1, binding = 2) uniform texture2D emissive_tex;
layout(set = 1, binding = 3) uniform texture2D bumpmap_tex;
layout(set = 1, binding = 4) uniform sampler samp;

// Gaussian
float gaussian(vec2 i, float sigma) {
  return exp(-0.5 * dot(i /= sigma, i)) / (6.28 * sigma * sigma);
}

// Bloom texture by blurring it
vec4 bloom(texture2D tx, sampler smp, vec2 uv, vec2 scale) {
  float sigma = float(uni_in.samples) * 0.25;
  vec4 out_ = vec4(0);
  int sLOD = 1 << uni_in.LOD;
  int s = uni_in.samples / sLOD;

  for (int i = 0; i < s * s; i++) {
    vec2 d = vec2(i % s, i / s) * float(sLOD) - float(uni_in.samples) / 2.0;
    out_ += gaussian(d, sigma) * textureLod(sampler2D(tx, smp), uv + scale * d, uni_in.LOD);
  }

  return out_ / out_.a;
}

void main() {

  // Bloom
  outEmissive = bloom(emissive_tex, samp, texUVs, 1.0 / uni_in.fbSize);

  // Set color to the corrosponding pixel in the FBO
  vec4 light = vec4(uni_in.ambientLight, 1) + outEmissive;

  outAlbedo = (texture(sampler2D(albedo_tex, samp), texUVs) * light);
  outBump = texture(sampler2D(bumpmap_tex, samp), texUVs);
}