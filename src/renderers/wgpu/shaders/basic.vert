#version 440
precision mediump float;
uniform vec2 trans;
attribute vec2 pos;
attribute vec2 uvs;
attribute vec2 deform;
varying vec2 texcoord;

void main() {
    vec2 pos2 = pos + trans + deform;
    pos2.y = -pos2.y;
    texcoord = uvs;
    gl_Position = vec4(pos2 / 3072.0, 0.0, 1.0);
}
