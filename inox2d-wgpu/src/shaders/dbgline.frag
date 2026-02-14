/*
    Copyright © 2020, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.
    
    Authors: Luna Nielsen
*/
#version 440
layout(location = 0) out vec4 outColor;

layout(binding=0) uniform Input {
  vec4 color;
} uni_in;

void main() {
  outColor = uni_in.color;
}