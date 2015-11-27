#[macro_use]
extern crate glium;
extern crate nalgebra as na;
extern crate num;
extern crate image;

use std::env;
use std::process;
use std::io::Cursor;

use glium::{glutin, DisplayBuild, Surface, VertexBuffer, IndexBuffer, DrawParameters, GlObject};
use glium::texture::cubemap::Cubemap;
use glium::texture::RawImage2d;
use glium::backend::glutin_backend::GlutinFacade as Display;
use glium::glutin::{Event, ElementState, MouseButton};
use glium::glutin::{Window};

use na::{PerspMat3, Iso3, Pnt3, Vec3, BaseFloat, Mat4, UnitQuat, Rotation};

use num::One;

mod obj;
mod gl;

#[cfg(feature = "dyn_assets")]
macro_rules! load_asset {
    ($x:expr) => (read_asset(concat!("./src/", $x)));
}

#[cfg(not(feature = "dyn_assets"))]
macro_rules! load_asset {
    ($x:expr) => (include_bytes!($x));
}

macro_rules! load_asset_str {
    ($x:expr) => (String::from_utf8_lossy(load_asset!($x).as_ref()));
}

#[cfg(feature = "dyn_assets")]
fn read_asset(path: &str) -> Vec<u8> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path).unwrap();
    let mut result = Vec::new();
    file.read_to_end(&mut result).unwrap();
    result
}

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

fn load_skybox_images() -> Vec<RawImage2d<'static, u8>> {
    let images: Vec<&[u8]> = vec![
        include_bytes!("./skybox/right.jpg"),
        include_bytes!("./skybox/left.jpg"),
        include_bytes!("./skybox/bottom.jpg"),
        include_bytes!("./skybox/top.jpg"),
        include_bytes!("./skybox/back.jpg"),
        include_bytes!("./skybox/front.jpg"),
    ];
    let mut result = Vec::new();
    for im in images {
        let im = image::load(Cursor::new(im), image::JPEG).unwrap().to_rgba();

        let dimension = im.dimensions();
        result.push(RawImage2d::from_raw_rgba_reversed(im.into_raw(), dimension))
    }
    result
}

struct Scene {
    light: Pnt3<f32>,
    camera_position: Pnt3<f32>,
    display: Display,
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
            &load_asset_str!("./shaders/model/vertex.glsl"),
            &load_asset_str!("./shaders/model/fragment.glsl"),
            None,
        ).unwrap();

        let skybox_program = glium::Program::from_source(
            &display,
            &load_asset_str!("./shaders/skybox/vertex.glsl"),
            &load_asset_str!("./shaders/skybox/fragment.glsl"),
            None,
        ).unwrap();


        let model_points = VertexBuffer::new(&display, &model.vertices).unwrap();
        let model_normals = VertexBuffer::new(&display, &model.normals).unwrap();
        let model_indices = IndexBuffer::new(
            &display,
            glium::index::PrimitiveType::TrianglesList,
            &model.indices
        ).unwrap();

        let skybox_images = load_skybox_images();
        let skybox_texture = unsafe {
            let result = Cubemap::empty(&display, 2048).unwrap();
            let id = result.get_id();

            let gl = {
                let w = Window::new().unwrap();
                gl::Gl::load_with(|s| w.get_proc_address(s) as *const _)
            };
            display.exec_in_context(|| {
                gl.ActiveTexture(gl::TEXTURE1);
                gl.BindTexture(gl::TEXTURE_CUBE_MAP, 0);
                gl.BindTexture(gl::TEXTURE_CUBE_MAP, id);
                for (i, im) in skybox_images.into_iter().enumerate() {
                    let bind_point = gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32;
                    gl.TexSubImage2D(bind_point, 0, 0, 0,
                                     im.width as i32, im.height as i32,
                                     gl::RGBA, gl::UNSIGNED_BYTE,
                                     im.data.as_ptr() as *const _);
                }

                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
                gl.BindTexture(gl::TEXTURE_CUBE_MAP, 0);
            });

            result
        };


        let params = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            polygon_mode: glium::draw_parameters::PolygonMode::Fill,
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
            light: Pnt3::new(1.0, -1.0, 1.0f32),
            camera_position: Pnt3::new(-0.03, -0.1, 0.4),
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

    fn draw(&self, view: &na::Mat4<f32>, projection: &na::Mat4<f32>) {
        let mut target = self.display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);
        let vp = *projection * *view;

        let uniforms = uniform! {
            vp: vp,
            light: self.light,
            camera_position: self.camera_position,
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
        transform.look_at_z(&scene.camera_position,
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
        let view = view * rot;

        scene.draw(&view, proj.as_mat());
    }

}

fn build_display() -> Display {
    glutin::WindowBuilder::new()
        .with_dimensions(800, 800)
        .with_depth_buffer(24)
        .with_gl_profile(glutin::GlProfile::Core)
//        .build_glium_debug(glium::debug::DebugCallbackBehavior::PrintAll)
        .build_glium()
        .unwrap()
}
