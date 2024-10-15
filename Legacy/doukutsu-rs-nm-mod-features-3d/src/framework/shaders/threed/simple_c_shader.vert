in vec2 position;
in vec4 color;
in vec2 uv;
//uniform mat4 viewProjection;


out vec2 uvs;
out vec4 col;


void main() {

    //gl_Position = viewProjection * vec4(position, 0.0, 1.0);
    gl_Position = vec4(position, 0.0, 1.0);

    col = color;
    uvs = uv;
}
