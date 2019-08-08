#version 430

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

out VS_OUT
{
    vec3 color;
} vs_out;

uniform vec2 u_mouse;
uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;
uniform float u_number_of_beads = 2916.0; // TODO

const float pi = 3.1415926535897932384626433832795;

vec3 hsv_to_rgb(vec3 c)
{
    const vec4 k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);

    return c.z * mix(k.xxx, clamp(p - k.xxx, 0.0, 1.0), c.y);
}

vec3 palette(in float t, in vec3 a, in vec3 b, in vec3 c, in vec3 d)
{
    return a + b * cos(6.28318 * (c * t + d));
}

void main()
{
    vec3 world_space_modified = abs(position / 4.0) * 0.5 + 0.5;

    world_space_modified.z = sqrt(world_space_modified.z);

    float x = abs(position.z);
    x = pow(min(cos(pi * x / 2.0), 1.0 - abs(x)), 3.0);

    const float hue = float(gl_VertexID / u_number_of_beads);
    const vec3 color = hsv_to_rgb(vec3(world_space_modified.zyx) * vec3(1.0, 0.6, 1.0));

    vs_out.color = color;

    gl_Position = u_projection * u_view * u_model * vec4(position, 1.0);
    gl_PointSize = abs(position.z) * 8.0;
}
