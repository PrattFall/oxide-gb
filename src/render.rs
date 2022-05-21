use std::time::{Duration, Instant};

use glium::index::PrimitiveType;
#[allow(unused_imports)]
use glium::{glutin, Surface};

use crate::video::LIGHTEST_GREEN;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

fn build_vertex_buffer(display: &glium::Display) -> glium::VertexBuffer<Vertex> {
    implement_vertex!(Vertex, position, tex_coords);

    glium::VertexBuffer::new(
        display,
        &[
            Vertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
                tex_coords: [1.0, 0.0],
            },
        ],
    )
    .unwrap()
}

pub fn render() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(160, 144i16));
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let vertex_buffer = build_vertex_buffer(&display);

    let index_buffer =
        glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
            .unwrap();

    let program = program!(
        &display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 matrix;

                in vec2 position;
                in vec2 tex_coords;

                out vec2 v_tex_coords;

                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",
            fragment: "
                #version 140

                uniform sampler2D tex;

                in vec2 v_tex_coords;
                out vec4 f_color;

                void main() {
                    f_color = texture(tex, v_tex_coords);
                }
            "
        }
    )
    .unwrap();

    let row = vec![LIGHTEST_GREEN; 160];
    let blank_pixels = vec![row; 144];

    let pixel_buffer = glium::texture::pixel_buffer::PixelBuffer::new_empty(&display, 160 * 144);
    let screen_texture = glium::texture::texture2d::Texture2d::new(&display, blank_pixels.clone()).unwrap();

    event_loop.run(move |event, _, control_flow| {
        let pixels = blank_pixels.clone().concat();

        pixel_buffer.write(&pixels);

        screen_texture
            .main_level()
            .raw_upload_from_pixel_buffer(pixel_buffer.as_slice(), 0 .. 160, 0 .. 144, 0 .. 1);

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex: screen_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let next_frame_time = Instant::now() + Duration::from_nanos(16666667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
    });
}
