/*
    Copyright © 2020, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.
    
    Authors: Luna Nielsen
*/
#version 440

layout(set = 0, binding = 0) uniform Input {
  mat4 mvp;
} uni_in;

layout(location = 0) in vec3 verts;

layout(location = 0) out vec2 texUVs;

void main() {
  gl_Position = uni_in.mvp * vec4(verts.x, verts.y, verts.z, 1);
}