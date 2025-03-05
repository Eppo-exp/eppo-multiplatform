pub(crate) use inner::*;

#[cfg(not(target_arch = "wasm32"))]
mod inner {
    pub(crate) type FlutterRustBridgeRuntime = tokio::runtime::Handle;

    pub(crate) fn get_runtime() -> FlutterRustBridgeRuntime {
        crate::frb_generated::FLUTTER_RUST_BRIDGE_HANDLER
            .async_runtime()
            .0
             .0
            .handle()
            .clone()
    }
}

// On wasm32, tokio runtime is not available and we run on "flutter rust bridge runtime", which
// internally uses wasm-bindgen.
#[cfg(target_arch = "wasm32")]
mod inner {
    pub struct FlutterRustBridgeRuntime;

    impl eppo_core::background::AsyncRuntime for FlutterRustBridgeRuntime {
        fn spawn<F>(&self, future: F)
        where
            F: std::future::Future + Send + 'static,
            F::Output: Send + 'static,
        {
            flutter_rust_bridge::spawn(future);
        }
    }

    pub fn get_runtime() -> FlutterRustBridgeRuntime {
        FlutterRustBridgeRuntime
    }
}
