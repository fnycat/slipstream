//! Implements a `tracing` subscriber that logs to a memory buffer.
//!
//! This makes it possible to view the logs within the editor itself.

use parking_lot::Mutex;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{Layer, layer::Context};

#[derive(Default)]
pub struct MemLogLayer(Mutex<Vec<LogEvent>>);

impl MemLogLayer {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
struct LogEvent {
    pub name: &'static str,
    pub verbosity: Level,
    pub target: String,
    pub message: String,
}

impl<S: Subscriber> Layer<S> for MemLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();

        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let mut lock = self.0.lock();
        lock.push(LogEvent {
            name: metadata.name(),
            verbosity: *metadata.level(),
            target: metadata.target().to_owned(),
            message: visitor.0,
        });

        println!("Pushed {:?}", lock.last().unwrap());
    }
}

#[derive(Default)]
struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        dbg!(field.name());
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}
