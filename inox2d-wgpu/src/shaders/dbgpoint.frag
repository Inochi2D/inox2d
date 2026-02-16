/*
    Copyright © 2020, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.

    Authors: Luna Nielsen
*/
#version 440
layout(location = 0) out vec4 outColor;

layout(set = 1, binding = 0) uniform Input {
  vec4 color;
} uni_in;

void main() {
  float r = 0.0;     // radius
  float alpha = 1.0; // alpha

  // r = point in circle compared against circle raidus
  vec2 cxy = 2.0 * gl_PointCoord - 1.0;
  r = dot(cxy, cxy);

  // epsilon width
  float epsilon = fwidth(r) * 0.5;

  // apply delta
  alpha = 1.0 - smoothstep(1.0 - epsilon, 1.0 + epsilon, r);
  outColor = uni_in.color * alpha;
}