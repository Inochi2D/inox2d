/*
    Copyright © 2020, Inochi2D Project
    Distributed under the 2-Clause BSD License, see LICENSE file.

    Authors: Luna Nielsen and Speykious

    Temporary simplified shaders.
    The goal is to smooth out the transition from simplified shaders to official
    shaders while introducing an MVP.
*/
#version 330
in vec2 texUVs;

layout(location = 0) out vec4 outAlbedo;

uniform sampler2D u_albedo;

uniform float opacity = 1;
uniform vec3 multColor = vec3(1);
uniform vec3 screenColor = vec3(0);

void main() {
  // Sample texture
  vec4 texColor = texture(u_albedo, texUVs);

  // Screen color math
  vec3 screenOut = vec3(1.0) - ((vec3(1.0) - (texColor.xyz)) *
                                (vec3(1.0) - (screenColor * texColor.a)));

  // Multiply color math + opacity application.
  outAlbedo =
      vec4(screenOut.xyz, texColor.a) * vec4(multColor.xyz, 1) * opacity;
}