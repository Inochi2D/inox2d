/*
    Copyright © 2020, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.

    Authors: Luna Nielsen and Speykious

    Temporary simplified shaders.
    The goal is to smooth out the transition from simplified shaders to official
    shaders while introducing an MVP.
*/
#version 330
uniform mat4 u_mvp;
uniform vec2 u_trans;

layout(location = 0) in vec2 verts;
layout(location = 1) in vec2 uvs;
layout(location = 2) in vec2 deform;

out vec2 texUVs;

void main() {
  gl_Position = u_mvp * vec4(verts + u_trans + deform, 0, 1);
  texUVs = vec2(uvs.x, -uvs.y);
}