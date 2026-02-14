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

layout(binding = 0) uniform sampler2D albedo;
layout(binding = 1) uniform sampler2D emissive;
layout(binding = 2) uniform sampler2D bumpmap;

layout(binding = 3) uniform Input {
    float opacity;
    vec3 multColor;
    vec3 screenColor;
} uni_in;

void main() {
  // Sample texture
  vec4 texColor = texture(albedo, texUVs);

  // Screen color math
  vec3 screenOut = vec3(1.0) - ((vec3(1.0) - (texColor.xyz)) *
                                (vec3(1.0) - (uni_in.screenColor * texColor.a)));

  // Multiply color math + opacity application.
  outAlbedo =
      vec4(screenOut.xyz, texColor.a) * vec4(uni_in.multColor.xyz, 1) * uni_in.opacity;

  // Emissive
  outEmissive = texture(emissive, texUVs) * outAlbedo.a;

  // Bumpmap
  outBump = texture(bumpmap, texUVs) * outAlbedo.a;
}