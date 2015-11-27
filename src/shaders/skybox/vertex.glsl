#version 330 core

in vec3 position;

out vec3 sky_coords;

uniform mat4 proj;
uniform mat4 view;

void main()
{
    mat4 vp = proj * view;
    gl_Position = vp * vec4(position, 1);
    sky_coords = -position;
}
