#version 330 core

#pragma include(iq.glsl);
in vec2 fragCoord;
out vec4 fragColor;

uniform float iTime;
uniform vec2 iResolution;
// mx, my, zoom_level
uniform vec3 iMouse;

uniform vec2 mouseCursor;

uniform vec3 cameraPosition;
uniform vec3 cameraTarget;

uniform mat4 projection;
uniform mat4 view;

uniform vec3 camDir;
uniform vec3 camPos;

#define MAXD    200.
#define STEPS   100
#define EPS     0.002

float smin(float a, float b, float k) {
    float h = max(k-abs(a-b), 0.0)/k;
    return min(a, b) - h*h*k*(1.0/4.0);
}

float map(vec3 p) {
    return sdBox(p, vec3(0.5));

}

vec3 normal(vec3 pos) {
    float eps=0.002, d=map(pos);
    return normalize(vec3(map(pos+vec3(eps, 0, 0))-d, map(pos+vec3(0, eps, 0))-d, map(pos+vec3(0, 0, eps))-d));
}

vec2 intersect(vec3 ro, vec3 rd) {
    float function_sign=(map(ro)<0.)?-1.:1.;
    float h=EPS*2.;
    float t=0.;
    for (int i=0;i<STEPS;i++) {
        if (abs(h)>EPS||t<MAXD) {
            h = function_sign*map(ro+rd*t);
            t += h;
        }
    }
    return vec2(t, h);
}

vec3 background(vec3 rd) {
    return vec3(0.3, 0.2, 0.1) + rd.z * 0.5;
}

void main(void) {
    vec2 uv = (2.*gl_FragCoord.xy-iResolution.xy)/iResolution.y;

    // camera
    vec2 mouse = iMouse.xy / iResolution.xy;

    vec3 cameraAt 	= camPos;

    float angleX =  6.28 * mouse.x;
    float angleY = 6.28 * mouse.y;
    vec3 cameraPos	= (vec3(sin(angleX)*cos(angleY), sin(angleY), cos(angleX)*cos(angleY))) * 3.0;

    vec3 cameraFwd = normalize(cameraAt - cameraPos);
    vec3 cameraLeft = normalize(cross(normalize(cameraAt - cameraPos), vec3(0.0,sign(cos(angleY)),0.0)));
    vec3 cameraUp = normalize(cross(cameraLeft, cameraFwd));

    float cameraViewWidth	= 6.0;
    float cameraViewHeight	= cameraViewWidth * iResolution.y / iResolution.x;
    float cameraDistance	= 6.0 + iMouse.z;

    vec2 rawPercent = (gl_FragCoord.xy / iResolution.xy);
    vec2 percent = rawPercent - vec2(0.5,0.5);

    vec3 rayTarget = (cameraFwd * vec3(cameraDistance,cameraDistance,cameraDistance))
    - (cameraLeft * percent.x * cameraViewWidth)
    + (cameraUp * percent.y * cameraViewHeight);

    vec3 rd = normalize(rayTarget);
    vec3 ro = cameraPos;

    vec3 color = vec3(0.1, 0.2, 0.3);

    vec2 hit = intersect(ro, rd);
    if (hit.x < MAXD) {
        vec3 pos = hit.x * rd + ro;
        vec3 n = normal(pos);

        // add simple lighting and color based on reflection
        float light = max(0.5, dot(n, vec3(0., 1., 0.)));
        color = background(reflect(rd, n)) * light;
    }

    // gamma correction
    color = pow(color, vec3(1.0/2.4));

    fragColor = vec4(color, 1.0);
}
