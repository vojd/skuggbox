use glow::Context;
use glutin::config::{Config, ConfigTemplateBuilder, GlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentContext, NotCurrentGlContextSurfaceAccessor,
    PossiblyCurrentContext, Version,
};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::WindowSurface;

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
    pub gl_context: Option<PossiblyCurrentContext>,
    pub gl_config: Config,

    // winit
    pub window: Option<Window>,
}

impl AppWindow {
    /// Setup the required bits for a winit Window
    /// Returns Self and the winit event loop
    pub fn new(_config: AppConfig, app_state: &AppState) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();
        // Let winit create a window builder
        let window_builder = WindowBuilder::new()
            .with_title("Skuggbox")
            .with_inner_size(LogicalSize::new(app_state.width, app_state.height));

        let template = ConfigTemplateBuilder::new();

        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

        let (mut window, gl_config) = display_builder
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
        let mut not_current_context = Some(unsafe {
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

        {
            let window = window.as_ref().unwrap();
            let attrs = window.build_surface_attributes(<_>::default());
            let gl_config = &gl_config;
            let gl_surface = unsafe {
                gl_config
                    .display()
                    .create_window_surface(&gl_config, &attrs)
                    .unwrap()
            };

            let ctx = not_current_context
                .take()
                .unwrap()
                .make_current(&gl_surface)
                .unwrap();
        }

        // Forge a gl_context out of the not_current_gl_context
        // NOTE: This might fail when / if skuggbox needs to recreate the context later
        // TODO: Add the context creation within the event::resumed part of the event loop as in the url below
        // https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs#L16

        // Create and return self and return gl and event_loop
        (
            Self {
                not_current_context,
                gl_context: None,
                gl_config,
                window,
            },
            // gl,
            event_loop,
        )
    }

    pub fn create_window_context(&mut self) {
        let window = self.window.as_ref().unwrap();
        let attrs = window.build_surface_attributes(<_>::default());
        let gl_config = &self.gl_config;
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let ctx = self
            .not_current_context
            .take()
            .unwrap()
            .make_current(&gl_surface)
            .unwrap();
    }
    // let gl_surface = unsafe {
    //     *self
    //         .gl_config
    //         .take()
    //         .display()
    //         .create_window_surface(&self.gl_config, &attrs)
    //         .unwrap()
    // };
    //
    // let ctx = self.not_current_context.take();
    // let gl_context: PossiblyCurrentContext = ctx.unwrap().make_current(&gl_surface).unwrap();
}

// pub fn create_window_context(app_window: &mut AppWindow) {
//     let window = app_window.window.as_ref().unwrap();
//     let attrs = window.build_surface_attributes(<_>::default());
//     let gl_config = &app_window.gl_config;
//     let gl_surface = unsafe {
//         gl_config
//             .display()
//             .create_window_surface(&gl_config, &attrs)
//             .unwrap()
//     };
//
//     // TODO(mathias) gah! ownership problem
//     let ctx = app_window.not_current_context.as_mut().unwrap();
//
//     let gl_context = ctx.make_current(&gl_surface).unwrap();
//
//     // app_window.gl_context = Option::from(gl_context);
// }
