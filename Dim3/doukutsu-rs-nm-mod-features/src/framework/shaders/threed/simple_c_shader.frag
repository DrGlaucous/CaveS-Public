
//from vertex shader
in vec2 uvs;
in vec4 col;

//uniforms
uniform mat3 textureTransformation;
uniform sampler2D tex;

//output
layout (location = 0) out vec4 outColor;
//out vec4 outColor;

void main()
{
    outColor = texture(tex, (textureTransformation * vec3(uvs, 1.0)).xy);
    outColor *= col;
}








