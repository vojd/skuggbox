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

pub const VERTEX_SHADER: &str = r#"#version 330 core
                const vec2 verts[3] = vec2[3](
                vec2(-1.0f, -1.0f),
                vec2(3.0f, -1.0f),
                vec2(-1.0f, 3.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert, 0.0, 1.0);
            }"#;
