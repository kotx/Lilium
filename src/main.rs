#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;

use std::sync::Mutex;

use assets::TypefaceContainer;
use game_loop::game_loop;
use skulpin::CoordinateSystemHelper;
use skulpin::{rafx::api::RafxExtents2D, skia_bindings::SkTextUtils_Align};
use skulpin::{
    skia_safe::{self, Font},
    Renderer,
};
use winit::{event::Event, window::Window};

#[cfg(debug_assertions)]
fn debug_builtin_cache() {
    assets::builtin().enhance_hot_reloading();
}

#[cfg(not(debug_assertions))]
fn debug_builtin_cache() {}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = winit::dpi::LogicalSize::new(900.0, 600.0);
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: logical_size.width as f32,
        top: 0.0,
        bottom: logical_size.height as f32,
    };
    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Center;

    let window = winit::window::WindowBuilder::new()
        .with_title("Lily")
        .with_inner_size(logical_size)
        .build(&event_loop)
        .expect("Failed to create window");

    let window_size = window.inner_size();
    let window_extents = RafxExtents2D {
        width: window_size.width,
        height: window_size.height,
    };

    let renderer = skulpin::RendererBuilder::new()
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .vsync_enabled(false)
        .build(&window, window_extents);

    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let renderer = renderer.unwrap();

    // Setup the cache
    if let Some(user) = assets::user() {
        user.enhance_hot_reloading();
    }

    // Enable hot reloading for dynamic builtin cache (if present)
    debug_builtin_cache();

    let lily_app = LilyApp::new(renderer);

    game_loop(
        event_loop,
        window,
        lily_app,
        240,
        0.1,
        |g| g.game.update(),
        |g| g.game.render(&g.window),
        |g, event| {
            if !g.game.handle_event(event) {
                g.exit();
            }
        },
    );
}

struct LilyApp {
    tick_count: u64,
    renderer: Mutex<Renderer>,
}

impl LilyApp {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            tick_count: 0,
            renderer: Mutex::new(renderer),
        }
    }

    pub fn update(self: &mut Self) {
        self.tick_count += 1;
    }

    pub fn handle_event(self: &Self, event: Event<()>) -> bool {
        match event {
            winit::event::Event::WindowEvent {
                event:
                    winit::event::WindowEvent::CloseRequested
                    | winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => return false,
            _ => {}
        }

        true
    }

    pub fn render(self: &Self, window: &Window) {
        let window_size = window.inner_size();
        let window_extents = RafxExtents2D {
            width: window_size.width,
            height: window_size.height,
        };

        let mut renderer = self.renderer.lock().unwrap();

        if let Err(e) = renderer.draw(
            window_extents,
            window.scale_factor(),
            |canvas, coordinate_system_helper| {
                self.draw(canvas, coordinate_system_helper, self.tick_count);
            },
        ) {
            println!("Error during draw: {:?}", e);
        }
    }

    fn draw(
        self: &Self,
        canvas: &mut skia_safe::Canvas,
        _coordinate_system_helper: CoordinateSystemHelper,
        tick_count: u64,
    ) {
        canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

        // Floating point value constantly moving between 0..1 to generate some movement
        let f = ((tick_count as f32 / 500.0).sin() + 1.0) / 2.0;

        // Make a color to draw with
        let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(f, 0.0, 1.0 - f, 1.0), None);
        paint.set_anti_alias(false);
        paint.set_style(skia_safe::paint::Style::Stroke);
        paint.set_stroke_width(2.0);

        // Draw a rectangle
        canvas.draw_rect(
            skia_safe::Rect {
                left: 10.0,
                top: 10.0,
                right: 890.0,
                bottom: 590.0,
            },
            &paint,
        );

        let handle = assets::builtin()
            .load::<TypefaceContainer>("PetMe64")
            .unwrap();

        let typeface = &handle.read().0;

        let font = Font::from_typeface(typeface, 50.0);

        canvas.draw_str_align("owo", (450, 300), &font, &paint, SkTextUtils_Align::Center);
    }
}
