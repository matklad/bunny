#version 330 core

uniform samplerCube skybox;

in vec3 sky_coords;


out vec4 color;

void main()
{
    color = vec4(1.0, 0.5, 0.1, 1.0);
}
