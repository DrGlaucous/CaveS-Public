#version 100
//handles texture-filled objects
//see https://webgl2fundamentals.org/webgl/lessons/webgl-qna-how-to-draw-correctly-textured-trapezoid-polygons.html

//float precision
precision mediump float;

//2d texture
uniform sampler2D Texture;

//interlopated textures from the corners
varying vec3 Frag_UV;
varying vec4 Frag_Color;


void main()
{
    //gl_FragColor = Frag_Color * texture2D(Texture, Frag_UV.st); //stock
	gl_FragColor = Frag_Color * texture2DProj(Texture, Frag_UV); //with proj func
    //gl_FragColor = Frag_Color * texture2D(Texture, Frag_UV.xy/Frag_UV.z); //same without proj func
}
