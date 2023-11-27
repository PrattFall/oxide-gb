use std::{
    fs::File,
    time::{Duration, Instant},
};

#[allow(unused_imports)]
use glium::{glutin, Surface};
use glium::{
    index::PrimitiveType,
    pixel_buffer::PixelBuffer,
    uniforms::{EmptyUniforms, Sampler, UniformsStorage},
    Program,
};
use winit::window::WindowBuilder;

use crate::{
    cartridge::Cartridge,
    cpu::{Cpu, CLOCK_MHZ},
    mbc::MBC,
    pixel::Pixel,
    video::{Video, Frame, SCREEN_HEIGHT, SCREEN_WIDTH},
};

type PixelColor = (u8, u8, u8, u8);

const DARKEST_GREEN: PixelColor = (15, 56, 15, 0);
const DARK_GREEN: PixelColor = (48, 98, 48, 0);
const LIGHT_GREEN: PixelColor = (139, 172, 15, 0);
const LIGHTEST_GREEN: PixelColor = (155, 188, 15, 0);

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

type Uniforms<'a> = UniformsStorage<
    'a,
    Sampler<'a, glium::texture::Texture2d>,
    UniformsStorage<'a, [[f32; 4]; 4], EmptyUniforms>,
>;

const VERTEX_SHADER_140: &'static str = "
#version 140

uniform mat4 matrix;

in vec2 position;
in vec2 tex_coords;

out vec2 v_tex_coords;

void main() {
    gl_Position = matrix * vec4(position, 0.0, 1.0);
    v_tex_coords = tex_coords;
}
";

const FRAGMENT_SHADER_140: &'static str = "
#version 140

uniform sampler2D tex;

in vec2 v_tex_coords;
out vec4 f_color;

void main() {
    f_color = texture(tex, v_tex_coords);
}
";

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

fn build_index_buffer(display: &glium::Display) -> glium::IndexBuffer<u16> {
    glium::IndexBuffer::new(display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]).unwrap()
}

fn build_window_builder() -> WindowBuilder {
    glutin::window::WindowBuilder::new().with_inner_size(
        glium::glutin::dpi::LogicalSize::<i16>::new(SCREEN_WIDTH.into(), SCREEN_HEIGHT.into()),
    )
}

fn build_pixel_buffer(display: &glium::Display) -> PixelBuffer<PixelColor> {
    glium::texture::pixel_buffer::PixelBuffer::<PixelColor>::new_empty(
        display,
        Into::<usize>::into(SCREEN_WIDTH) * Into::<usize>::into(SCREEN_HEIGHT),
    )
}

fn get_uniforms<'a>(screen_texture: &'a glium::texture::Texture2d) -> Uniforms<'a> {
    uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ],
        tex: screen_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
    }
}

fn build_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let wb = build_window_builder();
    let cb = glutin::ContextBuilder::new();
    glium::Display::new(wb, cb, &event_loop).unwrap()
}

fn build_program(display: &glium::Display) -> Program {
    program!(
        display,
        140 => {
            vertex: VERTEX_SHADER_140,
            fragment: FRAGMENT_SHADER_140
        }
    )
    .unwrap()
}

type FrameColors = Vec<Vec<PixelColor>>;

fn frame_to_colors(frame: Frame) -> FrameColors {
    fn frame_row_to_colors(row: &Vec<Pixel>) -> Vec<PixelColor> {
        row.iter().map(pixel_to_color).collect()
    }

    frame.iter().map(frame_row_to_colors).collect()
}

fn init_texture(display: &glium::Display) -> glium::texture::Texture2d {
    glium::texture::texture2d::Texture2d::new(display, frame_to_colors(Video::blank_frame())).unwrap()
}

fn pixel_to_color(pixel: &Pixel) -> PixelColor {
    match pixel {
        Pixel::Lightest => LIGHTEST_GREEN,
        Pixel::Light => LIGHT_GREEN,
        Pixel::Dark => DARK_GREEN,
        Pixel::Darkest => DARKEST_GREEN,
    }
}

pub fn render<'a>(input_file: File) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let display = build_display(&event_loop);
    let vertex_buffer = build_vertex_buffer(&display);
    let index_buffer = build_index_buffer(&display);
    let pixel_buffer = build_pixel_buffer(&display);
    let program = build_program(&display);
    let screen_texture = init_texture(&display);

    let cartridge = Cartridge::from(input_file);
    let mut memory = MBC::from(cartridge);
    let mut cpu = Cpu::default();

    // Skip over the Boot Rom
    cpu.program_counter = 0x100;

    event_loop.run(move |event, _, control_flow| {
        cpu.apply_operation(&mut memory);

        // Draw
        let pixels = Video::blank_frame()
            .concat()
            .iter()
            .map(pixel_to_color)
            .collect::<Vec<PixelColor>>();

        pixel_buffer.write(&pixels);

        screen_texture.main_level().raw_upload_from_pixel_buffer(
            pixel_buffer.as_slice(),
            0..SCREEN_WIDTH.into(),
            0..SCREEN_HEIGHT.into(),
            0..1,
        );

        let uniforms = get_uniforms(&screen_texture);

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

        let next_frame_time = Instant::now() + Duration::from_nanos(CLOCK_MHZ.into());
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
    });
}
