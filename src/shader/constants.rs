pub const SKUGGBOX_CAMERA: &str = "#ifdef USE_SKUGGBOX_CAMERA
    uniform mat4 sbCameraTransform;
    void skuggbox_camera(vec2 uv, inout vec3 ro, inout vec3 rd) {
        ro = sbCameraTransform[3].xyz;
        rd = mat3(sbCameraTransform) * normalize(vec3(uv, 1));
    }
    #else
    void skuggbox_camera(vec2 uv, inout vec3 ro, inout vec3 rd) {
        // empty
    }
    #endif";

pub const VERTEX_SHADER: &str = "#version 330 core
    layout (location = 0) in vec3 position;
    void main() {
        gl_Position = vec4(position, 1.0);
    }
    ";
