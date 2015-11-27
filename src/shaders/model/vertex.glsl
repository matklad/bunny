#version 330 core

in vec3 position;
in vec3 normal;

out vec3 v_normal;
out vec3 v_position;
out vec3 v_light;

uniform mat4 vp;
uniform vec3 light;

void main()
{
    v_normal = normal;
    gl_Position = vp * vec4(position, 1);
    v_position = position;
    v_light = light;
}
