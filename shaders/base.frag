#version 330 core

in vec2 Position;
out vec4 Color;

uniform float iTime;
vec2 iResolution = vec2(1024, 768);

uniform vec2 value;
uniform float hello;

float map(vec3 p) {
    vec3 q = fract(p)*2.0-1.0;
    return length(q)-0.25;
}

float sdBox( vec3 p, vec3 b ) {
    vec3 q = fract(p)*2.0-1.0;
    vec3 d =  abs(vec3(q.x,p.y-0.5,q.z))-b;
    return min(max(d.x,max(d.y,d.z)),0.0) + length(max(d,0.0));
}

float trace(vec3 ro, vec3 rd) {
    float t = 0.0;
    for(int i=0;i<64;i++){

        float precis = 0.0005*t;
        vec3 p = ro + rd * t;
        float d = sdBox(p,vec3(0.6,1.5,0.6));
        if( d < precis || t>200. ) break;
        t+=d*0.5;
    }
    return t;
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    uv = uv * 2.0 - 1.0;
    uv.x *= iResolution.x / iResolution.y;

    vec3 r = normalize(vec3(uv,1.));

    float theta = iTime * 0.2;
    r.xz *= mat2(cos(theta),-sin(theta),sin(theta),cos(theta));

    vec3 o = vec3(2.0,sin(iTime*0.1)*1.5+0.9,iTime*0.25);
    float t = trace(o,r);
    float fog = 0.5/(1.0+t*t*0.25)*(0.25+t*t*0.5);
    vec3 fc = clamp(vec3(fog),-0.,1.);

    Color = vec4(fc.x*0.75,fc.y*0.95,fc.z*1.,0.0);
}