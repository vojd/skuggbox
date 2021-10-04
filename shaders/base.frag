#version 330 core

#pragma include(iq.glsl);
in vec2 fragCoord;
out vec4 fragColor;

uniform float iTime;
uniform vec2 iResolution;
// mx, my, zoom_level
uniform vec4 iMouse;

#pragma include(skuggbox_camera)

#define MAXD 200.
#define STEPS 100
#define EPS  0.002
#define PI 3.141592

float smin(float a, float b, float k) {
    float h = max(k-abs(a-b), 0.0)/k;
    return min(a, b) - h*h*k*(1.0/4.0);
}

float map(vec3 p) {
    return sdBox(p, vec3(1.0));
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

float random(vec2 co) {
    float a = 12.9898;
    float b = 78.233;
    float c = 43758.5453;
    float dt= dot(co.xy ,vec2(a,b));
    float sn= mod(dt,3.14);
    return fract(sin(sn) * c);
}

float fog(const float dist, const float density) {
    const float LOG2 = -1.442695;
    float d = density * dist;
    return 1.0 - clamp(exp2(d * d * LOG2), 0.0, 1.0);
}

float intersectPlane(vec3 ro, vec3 rd, vec3 nor, float dist) {
    float denom = dot(rd, nor);
    float t = -(dot(ro, nor) + dist) / denom;
    return t;
}

// Source: http://www.iquilezles.org/www/articles/palettes/palettes.htm
vec3 palette(float t, vec3 a, vec3 b, vec3 c, vec3 d) {
    return a + b*cos( 6.28318*(c*t+d) );
}

vec3 bg(vec3 ro, vec3 rd) {
    vec3 col = 0.1 + (
    palette(clamp((random(rd.xz + sin(iTime * 0.1)) * 0.5 + 0.5) * 0.035 - rd.y * 0.5 + 0.35, -1.0, 1.0)
    , vec3(0.5, 0.45, 0.55)
    , vec3(0.5, 0.5, 0.5)
    , vec3(1.05, 1.0, 1.0)
    , vec3(0.275, 0.2, 0.19)
    )
    );


    float t = intersectPlane(ro, rd, vec3(0, 1, 0), 0.5);

    if (t > 0.0) {
        vec3 p = ro + rd * t;
        float g = (1.0 - pow(abs(sin(p.x) * cos(p.z)), 0.25));

        col += (1.0 - fog
        (t, 0.04)) * g * vec3(5, 4, 2) * 0.075;
        col += (1.0 - fog
        (t, 0.04)) * g * vec3(5, 4, 2) * 0.075;
    }

    return col;
}

mat3 camera(vec3 cameraPos, vec3 lookAtPoint) {
    vec3 cameraDir = normalize(lookAtPoint - cameraPos); // camera direction
    vec3 cameraRight = normalize(cross(vec3(0, 1, 0), cameraDir)); // camera right
    vec3 cameraUp = normalize(cross(cameraDir, cameraRight)); // camera up

    return mat3(-cameraRight, cameraUp, -cameraDir);
}

mat2 rotate2d(float theta) {
    float s = sin(theta), c = cos(theta);
    return mat2(c, -s, s, c);
}

void main(void) {
    vec2 uv = (2.*gl_FragCoord.xy-iResolution.xy)/iResolution.y;
    vec2 mouseUV = iMouse.xy / iResolution.xy;

    vec3 ro = vec3(0,0,-10);
    vec3 rd = mat3(vec3(1,0,0), vec3(0,1,0), vec3(0,0,1)) * normalize(vec3(uv, 1));
    skuggbox_camera(uv, ro, rd);

    vec3 color = bg(ro, rd);

    vec2 hit = intersect(ro, rd);
    if (hit.x < MAXD) {
        vec3 pos = ro + rd * hit.x;
        vec3 n = normal(pos);

        // add simple lighting and color based on reflection
        float light = max(0.5, dot(n, vec3(0., 1., 0.)));
        color = background(reflect(rd, n)) * light;
    }

    // gamma correction
    color = pow(color, vec3(1.0/2.4));
    fragColor = vec4(color, 1.0);
}
