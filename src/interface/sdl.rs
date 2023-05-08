use std::ffi::NulError;

use sdl2::{
    pixels::PixelFormatEnum,
    surface::Surface,
    video::{GLContext, GLProfile, Window},
    EventPump, Sdl,
};

use skia_bindings::{GrDirectContext, GrSurfaceOrigin};
use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget},
    ColorType, RCHandle,
};

use super::Interface;

impl Interface {
    // for macOS, we may mutate viewport.winsize. other OS don't (normally?) mutate viewport
    pub fn initialize_sdl(
        winsize: (f32, f32),
    ) -> (Sdl, Window, RCHandle<GrDirectContext>, GLContext) {
        const WL_ENV: &'static str = "WAYLAND_DISPLAY";
        if let Some(_) = std::env::var_os(WL_ENV) {
            let (k, v) = ("SDL_VIDEODRIVER", "wayland");
            std::env::set_var(k, v);
            log::info!(
                "Setting {k} to {v} as we see in env {}={}. If this fails, set {k} to `x11`!",
                WL_ENV,
                std::env::var(WL_ENV).unwrap()
            );
        }

        // SDL initialization
        let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
        let video_subsystem = sdl_context
            .video()
            .expect("Failed to create sdl video subsystem");

        let mut window = video_subsystem
            .window(&format!("MFEKufo"), winsize.0 as u32, winsize.1 as u32)
            .opengl()
            .position_centered()
            .allow_highdpi()
            .resizable()
            .build()
            .expect("Failed to create SDL Window");

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 3));

        let gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
        let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }
            video_subsystem.gl_get_proc_address(name) as *const _
        })
        .expect("Could not create interface");

        let gr_context = skia_safe::gpu::DirectContext::new_gl(Some(interface), None).unwrap();
        video_subsystem.text_input().start();

        let logo = include_bytes!("../../resources/icon.png");

        let mut im = image::load_from_memory_with_format(logo, image::ImageFormat::Png)
            .unwrap()
            .into_rgba8();

        // SDL2's pixel formats are not byte-by-byte, but rather word-by-word, where the words are each
        // 32 bits long. So RGBA8888 means a 32-bit word where 8 bits are R, G, B and A. However,
        // SDL2's words are not big endian, they are little endian, so we need to reverse them.
        im.chunks_exact_mut(4).for_each(|pixel: &mut _| {
            let oldpixel: [u8; 4] = [pixel[0], pixel[1], pixel[2], pixel[3]];
            pixel[0] = oldpixel[3];
            pixel[1] = oldpixel[2];
            pixel[2] = oldpixel[1];
            pixel[3] = oldpixel[0];
        });

        let surface = Surface::from_data(&mut im, 512, 512, 512 * 4, PixelFormatEnum::RGBA8888)
            .expect("Failed to create SDL2 Surface");

        window.set_icon(surface);

        (sdl_context, window, gr_context, gl_ctx)
    }

    pub fn set_window_title(&mut self, title: &str) -> Result<(), NulError> {
        self.sdl_window.set_title(title)
    }

    pub fn get_event_pump(&self) -> EventPump {
        self.sdl_context
            .event_pump()
            .expect("Could not create sdl event pump")
    }
}
