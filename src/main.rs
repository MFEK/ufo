//! MFEKufo
//! Main author is Fredrick Brennan (@ctrlcctrlv); see AUTHORS.
//! (c) 2021. Apache 2.0 licensed.
#![allow(non_snake_case)] // for our name MFEKglif
#![feature(
    panic_info_message,
    stmt_expr_attributes,
    cell_leak,
)]

// Cargo.toml comments say what crates are used for what.
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate skulpin;
extern crate sdl2;

use sdl2::keyboard::Keycode;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Mod,
    video::Window,
    Sdl,
};
pub use skulpin::skia_safe;
use skulpin::{rafx::api::RafxError, rafx::api::RafxExtents2D, LogicalSize, RendererBuilder};
use imgui_skia_renderer::Renderer;

use std::collections::HashSet;
use std::{cell::RefCell, rc::Rc};

mod system_fonts;

//pub mod renderer;

static WIDTH: u32 = 1500;
static HEIGHT: u32 = 900;

fn main() {
    env_logger::init();

    let (sdl_context, window) = initialize_sdl();

    // Skulpin initialization TODO: proper error handling
    let mut renderer = initialize_skulpin_renderer(&window).unwrap();
    //
    // set up imgui
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    imgui.style_mut().use_light_colors();
    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not create sdl event pump");

    let id = imgui.fonts().add_font(&[
        imgui::FontSource::TtfData {
            data: &crate::system_fonts::SYSTEMSANS.data,
            size_pixels: 14.0 * 1.92,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[
                    0x0020 as u16,
                    0x00FF as u16,
                    0,
                ]),
                ..Default::default()
            }),
        },
    ]);
    FONT_IDS.with(|ids| {
        ids.borrow_mut().push(id);
    });

    let imgui_renderer = Renderer::new(&mut imgui);

    'main_loop: loop {
        // Create a set of pressed Keys.
        let keys_down: HashSet<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let display_idx = window.display_index().expect("Window not on a display?");
        let dpi = sdl_context.video().unwrap().display_dpi(display_idx).unwrap().0;
        let scale_factor = dpi / 133.0; // 133 is the pixel density of a somewhat modern 1080p screen

        // sdl event handling
        for event in event_pump.poll_iter() {

            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) {
                continue;
            };

            match &event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    keymod: km,
                    ..
                } => {
                    if km.contains(Mod::LCTRLMOD) || km.contains(Mod::RCTRLMOD) {
                        break 'main_loop;
                    }
                }
                _ => {}
            }

            //match event {}
        }

        let (window_width, window_height) = window.vulkan_drawable_size();
        let extents = RafxExtents2D {
            width: window_width,
            height: window_height,
        };

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let mut ui = imgui.frame();
        let menu_tok = ui.begin_main_menu_bar().unwrap();
        ui.menu(imgui::im_str!("File"), true, ||{
            imgui::MenuItem::new(imgui::im_str!("Open")).shortcut(imgui::im_str!("Ctrl+O")).build(&ui);
            imgui::MenuItem::new(imgui::im_str!("Reload")).shortcut(imgui::im_str!("F5")).build(&ui);
            imgui::MenuItem::new(imgui::im_str!("Save As")).shortcut(imgui::im_str!("Ctrl+Shift+S")).build(&ui);
            imgui::MenuItem::new(imgui::im_str!("Flatten")).shortcut(imgui::im_str!("Ctrl+U")).build(&ui);
            imgui::MenuItem::new(imgui::im_str!("Quit")).shortcut(imgui::im_str!("Ctrl+Q")).build(&ui);
        });
        ui.menu(imgui::im_str!("Help"), true, ||{
            imgui::MenuItem::new(imgui::im_str!("About")).build(&ui);
        });
        menu_tok.end(&ui);
        //ui.show_demo_window(&mut true);
        imgui_sdl2.prepare_render(&ui, &window);
        let dd = ui.render();

        let drew = renderer.draw(extents, 1.0, |canvas, _coordinate_system_helper| {
            //renderer::render_frame(canvas, Wrapping((elapsed.as_millis() / 45) % 180).0);
            canvas.clear(0xffffffff);
            imgui_renderer.render_imgui(canvas, dd);
        });

        if drew.is_err() {
            warn!("Failed to draw frame. This can happen when resizing due to VkError(ERROR_DEVICE_LOST); if happens otherwise, file an issue.");
        }
    }
}

fn initialize_sdl() -> (Sdl, Window) {
    // SDL initialization
    let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to create sdl video subsystem");

    video_subsystem.text_input().start();

    let logical_size = LogicalSize {
        width: WIDTH,
        height: HEIGHT,
    };

    let window = video_subsystem
        .window(
            &format!("MFEKufo"),
            logical_size.width,
            logical_size.height,
        )
        .position_centered()
        .allow_highdpi()
        .vulkan()
        .resizable()
        .build()
        .expect("Failed to create window");

    /* TODO: Fix icon. 
    let logo = include_bytes!("../doc/logo.png");
    let im = image::load_from_memory_with_format(logo, image::ImageFormat::Png)
        .unwrap()
        .into_rgb8();
    let mut bytes = im.into_vec();
    let surface = Surface::from_data(
        &mut bytes,
        701,
        701,
        701 * 3,
        sdl2::pixels::PixelFormatEnum::RGB888,
    )
    .unwrap();
    window.set_icon(surface);
    */

    (sdl_context, window)
}

fn initialize_skulpin_renderer(sdl_window: &Window) -> Result<skulpin::Renderer, RafxError> {
    let (window_width, window_height) = sdl_window.vulkan_drawable_size();

    let extents = RafxExtents2D {
        width: window_width,
        height: window_height,
    };

    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Start;
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: WIDTH as f32,
        top: 0.0,
        bottom: HEIGHT as f32,
    };

    let renderer = RendererBuilder::new()
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(sdl_window, extents);

    return renderer;
}

thread_local! { pub static FONT_IDS: RefCell<Vec<imgui::FontId>> = RefCell::new(vec!()); }
