#version 100
//handles solid-fill objects
precision mediump float;

varying vec3 Frag_UV;
varying vec4 Frag_Color;

void main()
{
    gl_FragColor = Frag_Color;
}
