use crate::event::InputState;
use graphics::renderer::Renderer;
use graphics::types::{CORNFLOWER_BLUE, Color};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowAttributes, WindowId};

mod event;
mod graphics;

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl App {
    async fn initialize(&mut self, event_loop: &ActiveEventLoop) {
        let window_attribs = WindowAttributes::default()
            .with_title("Regnulum")
            .with_inner_size(PhysicalSize::new(1280, 720))
            .with_resizable(false);

        let window = Arc::new(event_loop.create_window(window_attribs).unwrap());
        let renderer = Renderer::new(window.clone()).await;
        let input = InputState::new();

        self.state = Some(State {
            window: window,
            renderer,
            input,
        })
    }

    fn frame_update(&mut self, event_loop: &ActiveEventLoop) {
        let Some(state) = &mut self.state else {
            eprintln!("App State wasn't initialized before first frame update!");
            return;
        };

        if state.input.is_key_down(KeyCode::Escape) {
            event_loop.exit();
        }

        let clear_color = Color::from_hex(CORNFLOWER_BLUE);
        state.renderer.render(clear_color);

        state.input.refresh();
        state.window.request_redraw();
    }

    fn on_resize(&mut self, size: PhysicalSize<u32>) {
        let Some(state) = &mut self.state else {
            return;
        };

        state.renderer.resize(size);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        pollster::block_on(self.initialize(event_loop));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::KeyboardInput { event, .. } => state.input.on_key_event(event),
            WindowEvent::Resized(size) => self.on_resize(size),
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.frame_update(event_loop);
            }
            _ => (),
        }
    }
}

struct State {
    window: Arc<Window>,
    renderer: Renderer,
    input: InputState,
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
