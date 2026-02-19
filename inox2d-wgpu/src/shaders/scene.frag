/*
    Copyright © 2020, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.

    Authors: Luna Nielsen
*/
#version 440

layout(location = 0) in vec2 texUVs;
layout(location = 0) out vec4 outColor;

layout(set = 1, binding = 0) uniform texture2D fbo;
layout(set = 1, binding = 1) uniform sampler samp;

void main() {
  // Set color to the corrosponding pixel in the FBO
  vec4 color = texture(sampler2D(fbo, samp), texUVs);
  outColor =
      vec4(color.r * color.a, color.g * color.a, color.b * color.a, color.a);
}