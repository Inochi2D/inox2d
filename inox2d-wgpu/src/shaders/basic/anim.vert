/*
    Copyright © 2020, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.

    Authors: Luna Nielsen
*/
#version 440

layout(binding = 0) uniform Input {
  mat4 mvp;
  vec2 offset;
  
  uniform vec2 splits;
  uniform float animation;
  uniform float frame;
} uni_in;

layout(location = 0) in vec2 verts;
layout(location = 1) in vec2 uvs;
layout(location = 2) in vec2 deform;

layout(location = 0) out vec2 texUVs;

void main() {
  gl_Position = uni_in.mvp * vec4(verts + uni_in.offset + deform, 0, 1);
  texUVs = vec2((uvs.x / uni_in.splits.x) * uni_in.frame, (uvs.y / uni_in.splits.y) * uni_in.animation);
}