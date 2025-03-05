//! Support for running activities in background.

mod async_runtime;
mod runtime;
#[cfg(not(target_arch = "wasm32"))]
mod thread;

pub use async_runtime::AsyncRuntime;
pub use runtime::BackgroundRuntime;
#[cfg(not(target_arch = "wasm32"))]
pub use thread::BackgroundThread;
