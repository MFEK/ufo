use egui_sdl2_event::DpiMode;

mod gui;
mod interface;
mod parsing;
mod ufo_cache;
mod util;
mod viewer;

use interface::Interface;

use gui::fontview::fontview;
use gui::menu::menu;
use libmfekufo::{blocks, glyphs};

use crate::viewer::UFOViewer;

/// This is a mix of the rust-sdl2 opengl example,
/// the skia-safe gl window example: https://github.com/rust-skia/rust-skia/blob/master/skia-safe/examples/gl-window/main.rs
/// and the egui-sdl2-event example: https://github.com/kaphula/egui-sdl2-event-example
fn main() {
    extern crate gl;
    extern crate sdl2;

    use egui_sdl2_event::EguiSDL2State;
    use sdl2::event::{Event, WindowEvent};
    use sdl2::keyboard::Keycode;
    use skia_safe::Color;

    use egui_skia::EguiSkia;

    let mut viewer: UFOViewer = UFOViewer::default();
    let mut interface = Interface::new((800., 600.));

    let mut egui_sdl2_state = EguiSDL2State::new(
        &interface.sdl_window,
        &interface.sdl_context.video().unwrap(),
        DpiMode::Custom(1.),
    );
    let mut egui_skia = EguiSkia::new();
    let mut surface = interface.create_surface();

    'running: loop {
        if viewer.is_requesting_exit() {
            break;
        }

        for event in interface.get_event_pump().poll_iter() {
            match &event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::Window {
                    window_id,
                    win_event:
                        WindowEvent::SizeChanged(_width, _height)
                        | WindowEvent::Resized(_width, _height),
                    ..
                } => {
                    interface.set_size((*_width as f32, *_height as f32));
                    if *window_id == interface.sdl_window.id() {
                        surface = interface.create_surface()
                    }
                }
                _ => {}
            }
            egui_sdl2_state.sdl2_input_to_egui(&interface.sdl_window, &event)
        }

        let (_duration, full_output) = egui_skia.run(
            egui_sdl2_state.take_egui_input(&interface.sdl_window),
            |ctx| {
                menu(ctx, &mut viewer, &mut interface);
                fontview(ctx, &mut viewer, &mut interface);
            },
        );
        egui_sdl2_state.process_output(&interface.sdl_window, &full_output);

        let canvas = surface.canvas();
        canvas.clear(Color::BLACK);
        egui_skia.paint(canvas);
        surface.flush();
        interface.sdl_window.gl_swap_window();
    }
}
