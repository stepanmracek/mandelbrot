extern crate sdl2;
use itertools::Itertools;
use num::complex::Complex;
use rayon::prelude::*;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseState;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};
use sdl2::{event::Event, render::TextureCreator};
use std::time::{Duration, Instant};

fn mandelbrot(c: Complex<f64>, iterations: u32) -> Option<u32> {
    let mut z = Complex::new(0.0, 0.0);
    for i in 0..iterations {
        z = z * z + c;
        if z.re * z.re + z.im * z.im > 4.0 {
            return Some(i);
        }
    }

    None
}

fn x_y_to_complex(
    x: i32,
    y: i32,
    window_size: &(u32, u32),
    view_port: &(Complex<f64>, Complex<f64>),
) -> Complex<f64> {
    let rel_x = x as f64 / window_size.0 as f64;
    let rel_y = y as f64 / window_size.1 as f64;
    let d = view_port.1 - view_port.0;
    let re = view_port.0.re + rel_x * d.re;
    let im = view_port.0.im + rel_y * d.im;
    Complex::new(re, im)
}

pub fn draw_fractal(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    y_x_coords: &[(i32, i32)],
    view_port: &(Complex<f64>, Complex<f64>),
    iterations: u32,
) -> Result<(), String> {
    let window_size = canvas.window().size();
    let (width, height) = window_size;

    let stamp = Instant::now();
    let data = y_x_coords
        .par_iter()
        .map(|(y, x)| {
            let c = x_y_to_complex(*x, *y, &window_size, view_port);
            mandelbrot(c, iterations)
        })
        .collect::<Vec<_>>();
    let elapsed = Instant::now() - stamp;
    println!("Computation time {elapsed:?}");

    let stamp = Instant::now();
    let mut data = data
        .into_iter()
        .flat_map(|i| {
            if let Some(iter) = i {
                let c = (255 * iter / iterations) as u8;
                [c / 2, c, c]
            } else {
                [0, 0, 0]
            }
        })
        .collect::<Vec<_>>();

    let surface = Surface::from_data(&mut data, width, height, width * 3, PixelFormatEnum::RGB24)
        .map_err(|e| e.to_string())?;
    let texture = texture_creator
        .create_texture_from_surface(surface)
        .map_err(|e| e.to_string())?;

    canvas
        .copy(&texture, None, None)
        .map_err(|e| e.to_string())?;
    let elapsed = Instant::now() - stamp;
    println!("Rendering time {elapsed:?}");

    canvas.present();
    Ok(())
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let window = video_subsystem
        .window("Mandelbrot explorer", WIDTH, HEIGHT)
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut view_port = (Complex::new(-2.0, -1.5), Complex::new(2.0, 1.5));
    let mut iterations = 200;
    let y_x_coords = (0..HEIGHT as i32)
        .cartesian_product(0..WIDTH as i32)
        .collect::<Vec<_>>();

    draw_fractal(
        &mut canvas,
        &texture_creator,
        &y_x_coords,
        &view_port,
        iterations,
    )?;

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::KpPlus),
                    ..
                } => {
                    iterations += 100;
                    println!("Increasing iterations count to {iterations}");
                    draw_fractal(
                        &mut canvas,
                        &texture_creator,
                        &y_x_coords,
                        &view_port,
                        iterations,
                    )?;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::KpMinus),
                    ..
                } => {
                    if iterations > 100 {
                        iterations -= 100;
                        println!("Decreasing iterations count to {iterations}");
                        draw_fractal(
                            &mut canvas,
                            &texture_creator,
                            &y_x_coords,
                            &view_port,
                            iterations,
                        )?;
                    }
                }
                _ => {}
            }
        }

        let mouse_state = MouseState::new(&event_pump);
        if mouse_state.left() {
            let d = view_port.1 - view_port.0;
            let click_point = x_y_to_complex(
                mouse_state.x(),
                mouse_state.y(),
                &canvas.window().size(),
                &view_port,
            );
            let rel_click = Complex::new(
                (click_point.re - view_port.0.re) / d.re,
                (click_point.im - view_port.0.im) / d.im,
            );
            view_port.0 = Complex::new(
                view_port.0.re + d.re * 0.1 * (rel_click.re),
                view_port.0.im + d.im * 0.1 * (rel_click.im),
            );
            view_port.1 = Complex::new(
                view_port.1.re - d.re * 0.1 * (1.0 - rel_click.re),
                view_port.1.im - d.im * 0.1 * (1.0 - rel_click.im),
            );
            draw_fractal(
                &mut canvas,
                &texture_creator,
                &y_x_coords,
                &view_port,
                iterations,
            )?;
        } else if mouse_state.right() {
            let d = view_port.1 - view_port.0;
            view_port.0 -= d * 0.1;
            view_port.1 += d * 0.1;
            draw_fractal(
                &mut canvas,
                &texture_creator,
                &y_x_coords,
                &view_port,
                iterations,
            )?;
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
