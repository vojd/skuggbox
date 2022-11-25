#version 330 core

in vec2 fragCoord;
out vec4 fragColor;

uniform float iTime;
uniform vec2 iResolution;
uniform vec4 iMouse;

void main(void) {

    vec2 uv = (2.*fragCoord.xy-iResolution.xy)/iResolution.y;
    fragColor = vec4(uv.xy, 0.0,1.0);
//    fragColor = vec4(sin(iTime), 0.0, 1.0, 1.0);
}
