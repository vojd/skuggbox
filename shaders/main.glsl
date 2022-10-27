#version 330 core

#pragma include(orbit_camera.glsl);

in vec2 fragCoord;
out vec4 fragColor;

// default uniforms passed in from skuggbox
uniform float iTime;
uniform vec2 iResolution;
uniform vec4 iMouse;

#define MAX_DISTANCE 200.
#define STEPS 100
#define EPS  0.002

float sdBox(vec3 p, vec3 b) {
    vec3 d = abs(p) - b;
    return min(max(d.x, max(d.y, d.z)), 0.0) + length(max(d, 0.0));
}

float map(vec3 p) {
    return sdBox(p, vec3(1.));
}

vec2 intersect(vec3 ro, vec3 rd) {
    float function_sign=(map(ro)<0.)?-1.:1.;
    float h=EPS*2.;
    float t=0.;
    for (int i=0;i<STEPS;i++) {
        if (abs(h)>EPS||t<MAX_DISTANCE) {
            h = function_sign*map(ro+rd*t);
            t += h;
        }
    }
    return vec2(t, h);
}

void main(void) {
    vec2 uv = (2.*gl_FragCoord.xy-iResolution.xy)/iResolution.y;

    vec3 rd = normalize(vec3(uv.x, uv.y, 1.));
    vec3 ro = vec3(0., 0., -4.);

    camera(ro, rd, iMouse);

    vec2 hit = intersect(ro, rd);
    vec3 color = vec3(.4 + 2.0 * sin(iTime + uv.x * pow(uv.x, -10.)) + 2.0, 0.2, 0.1);

    if (hit.x < MAX_DISTANCE) {
        color = vec3(0.);
    }

    fragColor = vec4(color, 1.0);
}
