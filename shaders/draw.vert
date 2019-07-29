#version 430

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

out VS_OUT
{
    vec3 color;
} vs_out;

vec3 hsv_to_rgb(vec3 c) {
    const vec4 k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);

    return c.z * mix(k.xxx, clamp(p - k.xxx, 0.0, 1.0), c.y);
}

void main()
{
    vs_out.color = hsv_to_rgb(vec3(color.r, 1.0, 1.0));

    gl_Position = vec4(position, 1.0);
}
