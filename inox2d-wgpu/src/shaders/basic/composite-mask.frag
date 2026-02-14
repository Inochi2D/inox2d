/*
    Copyright © 2020, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.

    Authors: Luna Nielsen
*/
#version 440

layout(location = 0) in vec2 texUVs;
layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform sampler2D tex;
layout(binding = 1) uniform Input {
  float threshold;
  float opacity;
} uni_in;

void main() {
  vec4 color = texture(tex, texUVs) * vec4(1, 1, 1, uni_in.opacity);
  if (color.a <= uni_in.threshold)
    discard;
  outColor = vec4(1, 1, 1, 1);
}