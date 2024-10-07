
in vec2 uvs;


uniform mat3 textureTransformation;

uniform sampler2D tex;

in vec4 col;

//layout (location = 0) out vec4 outColor;
out vec4 outColor;

void main()
{

    outColor = texture(tex, (textureTransformation * vec3(uvs, 1.0)).xy);
}








