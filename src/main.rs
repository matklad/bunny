#[macro_use]
extern crate glium;
extern crate nalgebra as na;
extern crate num;
extern crate image;

use std::env;
use std::process;

use glium::{glutin, DisplayBuild, Surface, VertexBuffer, IndexBuffer, DrawParameters};
use glium::glutin::{Event, ElementState, MouseButton};
use glium::texture::cubemap::Cubemap;
use glium::backend::glutin_backend::GlutinFacade as Facade;
use na::{PerspMat3, Iso3, Pnt3, Vec3, BaseFloat, Mat4, UnitQuat, Rotation};
use num::One;

mod obj;

trait EventRecorder {
    fn record_event(&mut self, event: &Event);
}

struct MouseTracker {
    mouse_speed: f32,
    previous_position: (i32, i32),
    current_position: (i32, i32),
    is_mouse_down: bool,
}

impl MouseTracker {
    fn new() -> MouseTracker {
        MouseTracker {
            mouse_speed: 0.01,
            previous_position: (0, 0),
            current_position: (0, 0),
            is_mouse_down: false,
        }
    }

    fn drag_amount(&self) -> (f32, f32) {
        if self.is_mouse_down {
            ((self.current_position.0 - self.previous_position.0) as f32 * self.mouse_speed,
             (self.current_position.1 - self.previous_position.1) as f32 * self.mouse_speed)
        } else {
            (0.0, 0.0)
        }
    }
}

impl EventRecorder for MouseTracker {
    fn record_event(&mut self, event: &Event) {
        match *event {
            Event::MouseMoved(new_position) => {
                self.previous_position = self.current_position;
                self.current_position = new_position;
            },
            Event::MouseInput(ElementState::Pressed, MouseButton::Left) =>
                self.is_mouse_down = true,
            Event::MouseInput(ElementState::Released, MouseButton::Left) =>
                self.is_mouse_down = false,
            _ => ()
        }
    }
}

struct Scene {
    light: [f32; 3],
    display: Facade,
    draw_parameters: DrawParameters<'static>,
    model_points: VertexBuffer<obj::Vertex>,
    model_normals: VertexBuffer<obj::Normal>,
    model_indices: IndexBuffer<u16>,
    model_program: glium::Program,
    skybox_points: VertexBuffer<obj::Vertex>,
    skybox_indices: glium::index::NoIndices,
    skybox_texture: Cubemap,
    skybox_program: glium::Program,
}

impl Scene {
    fn new(model: obj::Obj) -> Scene {
        let display = build_display();

        let model_program = glium::Program::from_source(
            &display,
            &include_str!("./shaders/model/vertex.glsl"),
            &include_str!("./shaders/model/fragment.glsl"),
            None,
        ).unwrap();

        let skybox_program = glium::Program::from_source(
            &display,
            &include_str!("./shaders/skybox/vertex.glsl"),
            &include_str!("./shaders/skybox/fragment.glsl"),
            None,
        ).unwrap();


        let model_points = VertexBuffer::new(&display, &model.vertices).unwrap();
        let model_normals = VertexBuffer::new(&display, &model.normals).unwrap();
        let model_indices = IndexBuffer::new(
            &display,
            glium::index::PrimitiveType::TrianglesList,
            &model.indices
        ).unwrap();

        let skybox_texture = Cubemap::empty(&display, 1024).unwrap();

        let params = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            polygon_mode: glium::draw_parameters::PolygonMode::Line,
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let skybox_vertices = vec![
            obj::Vertex::from(Vec3::new(-1.0f32,  1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32, -1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32, -1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32, -1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32,  1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32,  1.0f32, -1.0f32)),

            obj::Vertex::from(Vec3::new(-1.0f32, -1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32, -1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32,  1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32,  1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32,  1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32, -1.0f32,  1.0f32)),

            obj::Vertex::from(Vec3::new(1.0f32, -1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new(1.0f32, -1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new(1.0f32,  1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new(1.0f32,  1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new(1.0f32,  1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new(1.0f32, -1.0f32, -1.0f32)),

            obj::Vertex::from(Vec3::new(-1.0f32, -1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32,  1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32,  1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32,  1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32, -1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32, -1.0f32,  1.0f32)),

            obj::Vertex::from(Vec3::new(-1.0f32,  1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32,  1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32,  1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32,  1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32,  1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32,  1.0f32, -1.0f32)),

            obj::Vertex::from(Vec3::new(-1.0f32, -1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32, -1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32, -1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32, -1.0f32, -1.0f32)),
            obj::Vertex::from(Vec3::new(-1.0f32, -1.0f32,  1.0f32)),
            obj::Vertex::from(Vec3::new( 1.0f32, -1.0f32,  1.0f32)),
        ];
        let skybox_points = VertexBuffer::new(&display, &skybox_vertices).unwrap();
        let skybox_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);



        Scene {
            light: [1.0, -1.0, 1.0f32],
            display: display,
            draw_parameters: params,
            model_points: model_points,
            model_normals: model_normals,
            model_indices: model_indices,
            model_program: model_program,
            skybox_points: skybox_points,
            skybox_indices: skybox_indices,
            skybox_texture: skybox_texture,
            skybox_program: skybox_program,
        }
    }

    fn draw(&self, mvp: na::Mat4<f32>) {
        let mut target = self.display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let uniforms = uniform! {
            mvp: mvp,
            u_light: self.light,
            skybox: &self.skybox_texture,
        };

        target.draw(&self.skybox_points,
                    &self.skybox_indices,
                    &self.skybox_program,
                    &uniforms,
                    &self.draw_parameters).unwrap();

        target.draw((&self.model_points, &self.model_normals),
                    &self.model_indices,
                    &self.model_program,
                    &uniforms,
                    &self.draw_parameters).unwrap();

        target.finish().unwrap();
    }
}


fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: bunny model.obj");
        process::exit(-1);
    }
    let ref path = args[1];

    let model = obj::load_from_file(path).unwrap();

    let scene = Scene::new(model);

    let proj = PerspMat3::<f32>::new(1.0, f32::pi() / 4.0, 0.1, 100.0);
    let view: Mat4<f32> = na::to_homogeneous(&{
        let mut transform = Iso3::one();
        transform.look_at_z(&Pnt3::new(-0.03, -0.1, 0.4),
                            &Pnt3::new(-0.03, -0.1, 0.0),
                            &Vec3::new(0.0, 1.0, 0.0));
        transform
    });

    let mut mouse_tracker = MouseTracker::new();
    let mut rot = UnitQuat::new(Vec3::new(0.0, 0.0, 0.0));
    loop {
        for ev in scene.display.poll_events() {
            mouse_tracker.record_event(&ev);
            match ev {
                Event::Closed => return,
                _ => (),
            }
        }
        let (dx, dy) = mouse_tracker.drag_amount();
        rot = rot.append_rotation(&Vec3::new(dy, dx, 0.0));
        let rot = na::to_homogeneous(&rot.to_rot());
        let mvp: na::Mat4<f32> = *proj.as_mat() * view * rot;

        scene.draw(mvp);
    }

}

fn build_display() -> Facade {
    glutin::WindowBuilder::new()
        .with_dimensions(800, 800)
        .with_depth_buffer(24)
        .with_gl_profile(glutin::GlProfile::Core)
        .build_glium()
        .unwrap()
}
