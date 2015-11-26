#version 330 core

in vec3 position;
in vec3 normal;

out vec3 v_normal;
out vec3 v_position;
out vec3 v_light;


uniform mat4 mvp;
uniform vec3 u_light;

void main()
{
    v_normal = transpose(inverse(mat3(mvp))) * normal;
    gl_Position = mvp * vec4(position, 1);
    v_position = gl_Position.xyz / gl_Position.w;
    vec4 light = mvp * vec4(u_light, 1);
    v_light = light.xyz / light.w;
}
