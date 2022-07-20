/*
Made with skuggbox - tazadum
*/

#version 330 core
#define time iTime
#define PI 3.14159265

in vec2 fragCoord;
out vec4 fragColor;

uniform float iTime;
uniform vec2 iResolution;


void mainImage(out vec4 fragColor, in vec2 fragCoord) {

    vec2 uv = fragCoord/iResolution.xy;
    vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0, 2, 4));
    fragColor = vec4(col, 1.0);
}

void main() {

    vec2 uv = fragCoord/iResolution.xy;
    vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0, 2, 4));
    fragColor = vec4(col, 1.0);
}