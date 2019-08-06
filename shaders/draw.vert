#version 430

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

out VS_OUT
{
    vec3 color;
} vs_out;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;
uniform float u_number_of_beads = 6.0;// 2916.0; // TODO

vec3 hsv_to_rgb(vec3 c)
{
    const vec4 k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);

    return c.z * mix(k.xxx, clamp(p - k.xxx, 0.0, 1.0), c.y);
}

void main()
{
    vec3 world_space_color = (position / 3.0) * 0.5 + 0.5;

    float h = mod(float(gl_VertexID), 36.0 * 2.0) / (36.0 * 2.0);//float(gl_VertexID / (36.0 * 3.0));// float(gl_VertexID / u_number_of_beads);
    h *= 0.75;
    h += 0.525;

    vec3 rainbow_color = hsv_to_rgb(vec3(h, 1.0, 1.0));

    vs_out.color = rainbow_color;// mix(world_space_color.rbr, rainbow_color, 0.5);

    gl_Position = u_projection * u_view * u_model * vec4(position, 1.0);
}
