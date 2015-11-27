#version 330 core

uniform samplerCube skybox;

in vec3 v_normal;
in vec3 v_position;
in vec3 camera_pos;

out vec4 color;


void main()
{
    vec3 view = normalize(v_position - camera_pos);
    vec3 refl = reflect(view, normalize(v_normal));

    color = texture(skybox, -refl);
}
