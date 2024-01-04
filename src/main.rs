use std::error::Error;

use winit::event_loop::EventLoopBuilder;

mod display;
mod event;
mod point;
mod renderer;
mod window_context;

mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

use crate::event::{Event, Processor};

fn main() -> Result<(), Box<dyn Error>> {
    let window_event_loop = EventLoopBuilder::<Event>::with_user_event().build();

    let mut processor = Processor::new();
    processor.run(window_event_loop)
}
