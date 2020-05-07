#version 460

// These should never change: see the mesh module
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec2 texcoord;

out VS_OUT
{
    vec3 color;
} vs_out;

uniform vec2 u_mouse;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

uniform uint u_number_of_beads = 10; // TODO

const float pi = 3.1415926535897932384626433832795;

vec3 hsv_to_rgb(vec3 c)
{
    const vec4 k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    const vec3 p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);
    return c.z * mix(k.xxx, clamp(p - k.xxx, 0.0, 1.0), c.y);
}

vec3 palette(in float t, in vec3 a, in vec3 b, in vec3 c, in vec3 d)
{
    return a + b * cos(2.0 * pi * (c * t + d));
}

void main()
{
    // Generate a color from the position of this vertex (this is pretty arbitrary at the moment)
    vec3 world_space_modified = abs(position / 12.0) * 0.5 + 0.5;
    //world_space_modified.z = sqrt(world_space_modified.z * 24.0);
    vs_out.color = hsv_to_rgb(vec3(world_space_modified.zyx) * vec3(0.89, 0.6, 1.0));

    // Apply MVP matrices
    vec4 m_space = u_model * vec4(position, 1.0);
    vec4 v_space = u_view * m_space;
    vec4 p_space = u_projection * v_space;
    gl_Position = p_space;

    // Set the point size based on this point's z-depth
    const float point_scale_factor = 4.0;
    gl_PointSize = abs(position.z) * point_scale_factor;
}
