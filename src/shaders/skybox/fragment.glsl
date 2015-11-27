#version 330 core

uniform samplerCube skybox;

in vec3 sky_coords;


out vec4 color;

void main()
{
//    color =  color = vec4(sky_coords, 0);
    color = texture(skybox, sky_coords);
}
