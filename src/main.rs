use winit::event_loop::EventLoopBuilder;

use crate::event::{Event, Processor};

mod display;
mod event;
mod renderer;

mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

fn main() {
    let window_event_loop = EventLoopBuilder::<Event>::with_user_event().build();
    let processor = Processor::new(window_event_loop);
    processor.run();
}
