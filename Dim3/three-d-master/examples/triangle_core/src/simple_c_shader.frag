
//#define USE_TEXTURE
in vec2 uvs;

//////////////////////////SHARED

//////////////////////////////////FRAG SOURCE


uniform uint ColorMappingType;

vec3 color_mapping(vec3 color) {
    if (ColorMappingType == 1u) {
        vec3 a = vec3(0.055, 0.055, 0.055);
        vec3 ap1 = vec3(1.0, 1.0, 1.0) + a;
        vec3 g = vec3(2.4, 2.4, 2.4);
        vec3 ginv = 1.0 / g;
        vec3 select = step(vec3(0.0031308, 0.0031308, 0.0031308), color);
        vec3 lo = color * 12.92;
        vec3 hi = ap1 * pow(color, ginv) - a;
        color = mix(lo, hi, select);
    } 

    return color;
}




///////////////////////////////////COLOR MATERIAL

uniform vec4 surfaceColor;

//#ifdef USE_TEXTURE
uniform sampler2D tex;
uniform mat3 textureTransformation;
//#endif

in vec4 col;

layout (location = 0) out vec4 outColor;

void main()
{
    // outColor = surfaceColor * col;
    
    // #ifdef USE_TEXTURE
    // outColor *= texture(tex, (textureTransformation * vec3(uvs, 1.0)).xy);
    // #endif

    // outColor.rgb = color_mapping(outColor.rgb);

    outColor = texture2D(tex, uvs.st);
}








