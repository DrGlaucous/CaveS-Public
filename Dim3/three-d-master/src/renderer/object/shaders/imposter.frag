uniform sampler2DArray tex; //a bunch of 2d samplers

uniform mat4 view; //what direction we're looking at
uniform int no_views; //count of the 2d samplers
in vec2 uvs; //UVspace of source

layout (location = 0) out vec4 outColor; //final output color for this pixel

void main()
{
    vec3 dir = normalize(vec3(view[0][2], 0.0, view[2][2])); //get cartesian(?) direction from view matrix
    float a = acos(dir.x);
    float angle = (dir.z > 0.0 ? a : 2.0 * PI - a) / (2.0 * PI); //get the angle we're looking at this sprite

    //get the layer index that we should use
    float layer = float(no_views) * clamp(angle, 0.0, 0.999);

    //use this index and index+1 (wrapping)
    float index0 = floor(layer);
    float index1 = float((int(index0) + 1) % no_views);
    float frac = layer - index0; //determine how much of each index we should use

    //get the sample colors at x,y,index
    vec4 color0 = texture(tex, vec3(uvs.x, uvs.y, index0));
    vec4 color1 = texture(tex, vec3(uvs.x, uvs.y, index1));
    //mix them by how much each should show up
    outColor = mix(color0, color1, frac);

    //discard low alphas
    if(outColor.a < 0.5) {
        discard;
    }
    //apply tone and color mapping
    outColor.rgb = tone_mapping(outColor.rgb);
    outColor.rgb = color_mapping(outColor.rgb);
}
