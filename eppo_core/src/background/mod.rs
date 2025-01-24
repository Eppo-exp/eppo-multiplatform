//! Support for running activities in background.

mod runtime;
mod thread;

pub use runtime::BackgroundRuntime;
pub use thread::BackgroundThread;
