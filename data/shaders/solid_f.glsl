#version 420 core

in vec2 uv;
in vec3 normal;

layout (location = 0) out vec4 color;

uniform vec3 ambient_color;
uniform vec3 diffuse_color;
uniform vec3 specular_color;
uniform sampler2D diffuse_texture;
uniform ivec3 opts;
// (texture == & 0b10), (color == & 0b1)
// x = ambient
// y = diffuse
// z = specular

void main() {
    /* vec3 diffuse = (opts.y == 0) ? vec3(0) : (
        (((opts.y & 1) == 1) ? diffuse_color: vec3(1))
        * (((opts.y & 2) == 2) ? texture(diffuse_texture, uv).rgb : vec3(1))
    );*/
    // TODO: change 'true' to opts check for diffuse sampler
    // and change opts to uint or smth
    vec3 diffuse_tx = true ? texture(diffuse_texture, uv).rgb : vec3(1);
    vec3 diffuse = diffuse_color * diffuse_tx;
    // vec3 diffuse = vec3(1);
    // vec3 diffuse = vec3(uv, 0);
    // vec3 diffuse = diffuse_tx;
    color = vec4(diffuse, 1.0f);
}
