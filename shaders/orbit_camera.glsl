#define PI 3.1415926535

// A simple and naive orbital camera
// Should probably use some modifier value to set mouse sensitivity
#define R(p, a) p=cos(a)*p+sin(a)*vec2(p.y, -p.x)
void camera(inout vec3 ro, inout vec3 rd, vec4 mouse) {
    R(rd.yz, -mouse.y*0.01*PI*2.);
    R(rd.xz, mouse.x*0.01*PI*2.);
    R(ro.yz, -mouse.y*0.01*PI*2.);
    R(ro.xz, mouse.x*0.01*PI*2.);
}
