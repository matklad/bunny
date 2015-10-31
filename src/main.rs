#[macro_use] extern crate glium;
extern crate nalgebra as na;
extern crate num;

use std::env;
use std::process;

use glium::{glutin, DisplayBuild, Surface};
use glium::glutin::{Event};
use na::{PerspMat3, Iso3, Pnt3, Vec3, BaseFloat, Mat4};
use num::One;

mod obj;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: bunny model.obj");
        process::exit(-1);
    }

    let ref path = args[1];
    let model = obj::load_from_file(path).unwrap();
    let light = [1.0, -1.0, 1.0f32];


    let display = build_display();
    let program = glium::Program::from_source(&display,
                                              &include_str!("./shaders/vertex.glsl"),
                                              &include_str!("./shaders/fragment.glsl"),
                                              None).unwrap();

    let points = glium::VertexBuffer::new(&display, &model.vertices).unwrap();
    let normals = glium::VertexBuffer::new(&display, &model.normals).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
                                          &model.indices).unwrap();


    let proj = PerspMat3::<f32>::new(1.0, f32::pi() / 4.0, 0.1, 100.0);
    let view: Mat4<f32> = na::to_homogeneous(& {
        let mut transform = Iso3::one();
        transform.look_at_z(&Pnt3::new(-0.03, -0.1, 0.4),
                            &Pnt3::new(-0.03, -0.1, 0.0),
                            &Vec3::new(0.0, 1.0, 0.0));
        transform
    });

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    loop {
        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                _ => ()
            }
        }

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let mvp = *proj.as_mat() * view;

        let uniforms = uniform! {
            mvp: mvp,
            u_light: light,
        };

        target.draw((&points, &normals), &indices,
                    &program, &uniforms,
                    &params).unwrap();

        target.finish().unwrap();
    }

}

fn build_display() -> glium::backend::glutin_backend::GlutinFacade {
    glutin::WindowBuilder::new()
        .with_dimensions(800, 800)
        .with_depth_buffer(24)
        .with_gl_profile(glutin::GlProfile::Core)
        .build_glium().unwrap()
}
