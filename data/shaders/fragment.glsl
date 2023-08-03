#version 330 core

in vec2 UV;
in vec3 position_w;
in vec3 normal_c;
in vec3 eyeDirection_c;
in vec3 lightDirection_c;

out vec3 color;

uniform sampler2D sampler;
uniform vec3 lightPosition_w;
uniform float lightPower;
uniform ivec3 lightIntensity;

void main() {
    vec3 lightColor = vec3(1, 1, 1);
    vec3 mDiffuse = texture(sampler, UV).rgb;
    vec3 mAmbient = vec3(0.1, 0.1, 0.1) * mDiffuse;
    vec3 mSpecular = vec3(0.3, 0.3, 0.3);
    float dist = length(lightPosition_w - position_w);
    vec3 norm = normalize(normal_c);
    vec3 light = normalize(lightDirection_c);
    float cos0 = clamp(dot(norm, light), 0, 1);
    vec3 eye = normalize(eyeDirection_c);
    vec3 reflection = reflect(-light, norm);
    float cosA = clamp(dot(eye, reflection), 0, 1);
    float sqr_dist = dist * dist;
    vec3 colorA = lightIntensity.x * mAmbient;
    vec3 colorD = lightIntensity.y * (
            mDiffuse * lightColor * lightPower * cos0 / sqr_dist);
    vec3 colorS = lightIntensity.z * (
            mSpecular * lightColor * lightPower * pow(cosA, 5) / sqr_dist);
    color = colorA + colorD + colorS;
}
