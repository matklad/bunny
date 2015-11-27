#version 330 core

uniform samplerCube skybox;
uniform vec3 camera_position;

in vec3 v_normal;
in vec3 v_position;
in vec3 v_light;

out vec4 color;


const vec3 ambient_color = vec3(0.1, 0.1, 0.1);
const vec3 diffuse_color = vec3(0.3, 0.3, 0.3);
const vec3 specular_color = vec3(1.0, 1.0, 1.0);

void main()
{
    float diffuse = max(dot(normalize(v_normal), normalize(v_light)),
                        0.0);

    vec3 camera_dir = normalize(-v_position);
    vec3 half_direction = normalize(normalize(v_light) + camera_dir);
    float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0),
                         16.0);

    /*color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color,
                 1.0);*/
    vec3 view = normalize(v_position - camera_position);
    vec3 refl = reflect(view, normalize(v_normal));
    color = texture(skybox, -refl);
}
