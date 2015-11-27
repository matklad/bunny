#version 330 core

uniform vec3 camera_position;

in vec3 position;
in vec3 normal;

out vec3 v_normal;
out vec3 v_position;
out vec3 camera_pos;

uniform mat4 proj;
uniform mat4 view;

void main()
{
    mat4 vp = proj * view;
    gl_Position = vp * vec4(position, 1);

    v_normal = normal;
    v_position = position;
    vec4 t = view * vec4(camera_position, 1);
    camera_pos = t.xyz / t.w;
}
