//! Implements a `tracing` subscriber that logs to a memory buffer.
//!
//! This makes it possible to view the logs within the editor itself.

use std::sync::LazyLock;

use parking_lot::Mutex;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{Layer, layer::Context};

pub static GLOBAL_MEM_LOGS: LazyLock<Mutex<Vec<LogEvent>>> = LazyLock::new(|| Mutex::default());

#[derive(Default)]
pub struct GlobalMemLogLayer;

impl GlobalMemLogLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear() {
        GLOBAL_MEM_LOGS.lock().clear();
    }
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub name: &'static str,
    pub verbosity: Level,
    pub target: String,
    pub message: String,
}

impl<S: Subscriber> Layer<S> for GlobalMemLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();

        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let mut lock = GLOBAL_MEM_LOGS.lock();
        lock.push(LogEvent {
            name: metadata.name(),
            verbosity: *metadata.level(),
            target: metadata.target().to_owned(),
            message: visitor.0,
        });
    }
}

#[derive(Default)]
struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}
