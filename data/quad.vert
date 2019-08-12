#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 a_pos;
layout(location = 1) in vec2 a_uv;
layout(location = 2) in vec4 a_color;
layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec4 v_color;

layout(set = 0, binding = 0) uniform Locals {
    mat4 mvpmat;
};

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    v_uv = a_uv;
    v_color = a_color;
    gl_Position = mvpmat * a_pos;
    gl_Position.y *= -1;
}
