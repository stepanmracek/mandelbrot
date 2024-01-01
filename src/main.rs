extern crate sdl2;
use itertools::{self, Itertools};
use num::complex::Complex;
use rayon::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
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
    view_port: &(Complex<f64>, Complex<f64>),
    iterations: u32,
) -> Result<(), String> {
    let window_size = canvas.window().size();
    let (w, h) = window_size;

    let stamp = Instant::now();
    let coords = (0..w as i32)
        .cartesian_product(0..h as i32)
        .collect::<Vec<_>>();
    let set = coords
        .par_iter()
        .map(|(x, y)| {
            let c = x_y_to_complex(*x, *y, &window_size, view_port);
            (x, y, mandelbrot(c, iterations))
        })
        .collect::<Vec<_>>();
    let elapsed = Instant::now() - stamp;
    println!("Computation time {elapsed:?}");

    let stamp = Instant::now();
    for (x, y, i) in set {
        if let Some(iter) = i {
            let c = (255 * iter / iterations) as u8;
            canvas.set_draw_color(Color::RGB(c / 2, c, c));
        } else {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
        }
        canvas.draw_point((*x, *y))?;
    }
    let elapsed = Instant::now() - stamp;
    println!("Rendering time {elapsed:?}");

    canvas.present();
    Ok(())
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Mandelbrot explorer", 800, 600)
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut view_port = (Complex::new(-2.0, -1.5), Complex::new(2.0, 1.5));
    let mut iterations = 100;
    draw_fractal(&mut canvas, &view_port, iterations)?;

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
                    draw_fractal(&mut canvas, &view_port, iterations)?;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::KpMinus),
                    ..
                } => {
                    if iterations > 100 {
                        iterations -= 100;
                        println!("Decreasing iterations count to {iterations}");
                        draw_fractal(&mut canvas, &view_port, iterations)?;
                    }
                }
                _ => {}
            }
        }

        let mouse_state = mouse::MouseState::new(&event_pump);
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
            draw_fractal(&mut canvas, &view_port, iterations)?;
        } else if mouse_state.right() {
            let d = view_port.1 - view_port.0;
            view_port.0 -= d * 0.1;
            view_port.1 += d * 0.1;
            draw_fractal(&mut canvas, &view_port, iterations)?;
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
