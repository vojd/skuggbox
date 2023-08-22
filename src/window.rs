use glow::Context;
use glutin::config::{Config, ConfigTemplateBuilder, GlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentContext, NotCurrentGlContextSurfaceAccessor,
    PossiblyCurrentContext, Version,
};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, WindowSurface};
use std::ffi::CString;
use std::sync::Arc;

use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::{AppConfig, AppState};

/// Encapsulates everything needed for setting up the window and gl_context
pub struct AppWindow {
    // glutin
    pub not_current_context: Option<NotCurrentContext>,
    pub gl_config: Config,

    gl_context: Option<PossiblyCurrentContext>,
    gl_surface: Option<Surface<WindowSurface>>,
    // glow
    // pub gl: Option<Arc<glow::Context>>,

    // winit
    pub window: Option<Window>,
}

impl AppWindow {
    /// Setup the required bits for a winit Window
    /// Returns Self and the winit event loop
    pub fn new(_config: AppConfig, app_state: &AppState) -> (Self, EventLoop<()>) {
        // TODO: Move event loop out of AppWindow
        let event_loop = EventLoop::new();

        // Let winit create a window builder
        let window_builder = WindowBuilder::new()
            .with_title("Skuggbox")
            .with_inner_size(LogicalSize::new(app_state.width, app_state.height));

        let template = ConfigTemplateBuilder::new();

        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

        let (window, gl_config) = display_builder
            .build(&event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        let transparency_check = config.supports_transparency().unwrap_or(false)
                            & !accum.supports_transparency().unwrap_or(false);

                        if transparency_check || config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap(); // TODO(mathias): ? operator instead

        let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

        let fallback_context_attributes = ContextAttributesBuilder::new()
            // When GlProfile::Core is used at least 3.3 will be used
            .with_context_api(ContextApi::OpenGl(None))
            .build(raw_window_handle);

        let legacy_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
            .build(raw_window_handle);

        // This gl context has not been initialized yet. It will be set up in the Event::Resume handler
        let not_current_context = Some(unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .unwrap_or_else(|_| {
                            gl_display
                                .create_context(&gl_config, &legacy_context_attributes)
                                .expect("failed to create gl context")
                        })
                })
        });

        // Create and return self and return gl and event_loop
        (
            Self {
                not_current_context,
                gl_config,
                gl_context: None,
                gl_surface: None,
                window,
            },
            // gl,
            event_loop,
        )
    }

    /// Forge a gl_context (PossiblyCurrentContext) out of the not_current_context
    /// NOTE: This should only be called during the Event::Resume part of the event loop as per this doc
    /// https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs#L16
    pub fn create_window_context(&mut self) -> Arc<Context> {
        let window = self.window.as_ref().unwrap();
        let attrs = window.build_surface_attributes(<_>::default());
        let gl_config = &self.gl_config;
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(gl_config, &attrs)
                .unwrap()
        };

        let gl_context = self
            .not_current_context
            .take()
            .unwrap()
            .make_current(&gl_surface)
            .unwrap();

        let gl_display = gl_config.display();
        let gl = unsafe {
            Context::from_loader_function(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            })
        };
        self.gl_context = Some(gl_context);
        // Return the gl context and the WindowSurface which is used to swap buffers
        self.gl_surface = Some(gl_surface);
        Arc::new(gl)
    }

    /// Only call when you know that the gl context is initialized or you'll have a panic
    pub fn swap_buffers(&self) {
        let surface = self.gl_surface.as_ref().unwrap();
        let gl_context = self.gl_context.as_ref().unwrap();
        if let Err(err) = surface.swap_buffers(gl_context) {
            log::error!("Failed to swap buffers {:?}", err);
        };
    }
}
