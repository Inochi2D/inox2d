//    Copyright © 2020, Inochi2D Project
//  Distributed under the 2-Clause BSD License, see LICENSE file.
//
//  Authors: Luna Nielsen, Fiana Fortressia

struct Uniforms {
    @location(0) albedo: texture_2d<uint>;
    @location(1) emissive: texture_2d<uint>;
    @location(2) bumpmap: texture_2d<uint>;
    opacity: float;
    multColor: vec3;
    screenColor: vec3;
    emissionStrength: float;
}
@group @binding(0) let<uniform> uniforms: Uniforms;

@fragment
fn fs_main() {
   // Sample texture
 // vec4 texColor = texture(albedo, texUVs);

  // Screen color math
//  vec3 screenOut = vec3(1.0) - ((vec3(1.0) - (texColor.xyz)) *
 //                               (vec3(1.0) - (screenColor * texColor.a)));

  // Multiply color math + opacity application.
 // outAlbedo =
 //     vec4(screenOut.xyz, texColor.a) * vec4(multColor.xyz, 1) * opacity;

  // Emissive
//  outEmissive =
 //     vec4(texture(emissive, texUVs).xyz * emissionStrength, 1) * outAlbedo.a;

  // Bumpmap
//  outBump = vec4(texture(bumpmap, texUVs).xyz, 1) * outAlbedo.a;
}

@vertex 
fn vs_main() {
    //  gl_Position = mvp * vec4(verts.x - offset.x + deform.x,
    //                       verts.y - offset.y + deform.y, 0, 1);
 // texUVs = uvs;
}