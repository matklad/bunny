#[macro_use] extern crate glium;
extern crate nalgebra;

use std::env;
use std::process;

use glium::{glutin, DisplayBuild, Surface};
use glium::glutin::{Event};

mod obj;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: bunny model.obj");
        process::exit(-1);
    }

    let ref path = args[1];
    let model = obj::load_from_file(path).unwrap();


    let display = build_display();
    let program = glium::Program::from_source(&display,
                                              &include_str!("./shaders/vertex.glsl"),
                                              &include_str!("./shaders/fragment.glsl"),
                                              None).unwrap();

    let points = glium::VertexBuffer::new(&display, &model.vertices).unwrap();
    let normals = glium::VertexBuffer::new(&display, &model.normals).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
                                          &model.indices).unwrap();

    loop {
        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                _ => ()
            }
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        target.draw((&points, &normals), &indices,
                    &program, &glium::uniforms::EmptyUniforms,
                    &Default::default()).unwrap();

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
