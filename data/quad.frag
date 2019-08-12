#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec4 v_color;
layout(location = 0) out vec4 target0;

void main() {
    vec2 d = abs(v_uv - 0.5);
    float intensity = clamp(1 - 2 * sqrt(dot(d, d)), 0, 1);
    target0 = v_color * intensity;
}
