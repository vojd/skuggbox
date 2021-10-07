#ifdef USE_SKUGGBOX_CAMERA
uniform mat4 sbCameraTransform;
void skuggbox_camera(vec2 uv, inout vec3 ro, inout vec3 rd) {
    ro = sbCameraTransform[3].xyz;
    rd = mat3(sbCameraTransform) * normalize(vec3(uv, 1));
}
#else
void skuggbox_camera(vec2 uv, inout vec3 ro, inout vec3 rd) {
    // empty
}
#endif