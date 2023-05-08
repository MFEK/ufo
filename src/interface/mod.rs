use sdl2::{
    video::{GLContext, Window as SdlWindow},
    Sdl,
};
use skia_bindings::{GrDirectContext, GrSurfaceOrigin};
use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget},
    ColorType, RCHandle, Surface,
};

mod sdl;

pub struct Interface {
    pub sdl_context: Sdl,
    pub sdl_dpi: f32,
    pub sdl_window: SdlWindow,
    winsize: (f32, f32),

    gl_ctx: GLContext,
    pub gr_context: RCHandle<GrDirectContext>,
    pub fb_info: FramebufferInfo,
}

impl Interface {
    pub fn new(winsize: (f32, f32)) -> Self {
        let (sdl, window, gr_context, gl_ctx) = Self::initialize_sdl(winsize);

        let fb_info = fb_info();

        #[allow(unused_mut)]
        let mut iself = Interface {
            sdl_context: sdl,
            sdl_window: window,
            sdl_dpi: f32::NAN,
            winsize,
            gr_context,
            gl_ctx,
            fb_info,
        };

        iself
    }

    pub fn set_size(&mut self, winsize: (f32, f32)) {
        self.winsize = winsize;
    }

    pub fn get_size(&self) -> (f32, f32) {
        self.winsize
    }

    pub fn create_surface(&mut self) -> skia_safe::Surface {
        let (width, height) = self.sdl_window.drawable_size();

        let backend_render_target =
            BackendRenderTarget::new_gl((width as i32, height as i32), 0, 8, self.fb_info);

        Surface::from_backend_render_target(
            &mut self.gr_context,
            &backend_render_target,
            GrSurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap()
    }
}

fn fb_info() -> FramebufferInfo {
    let mut fboid = 0;
    unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

    FramebufferInfo {
        fboid: fboid.try_into().unwrap(),
        format: skia_safe::gpu::gl::Format::RGBA8.into(),
    }
}
