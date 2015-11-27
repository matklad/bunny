#version 330 core

in vec3 position;

out vec3 sky_coords;

uniform mat4 mvp;

void main()
{
    gl_Position = mvp * vec4(position, 1);
    sky_coords = -position;
}
