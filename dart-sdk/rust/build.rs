use lib_flutter_rust_bridge_codegen::codegen;
use lib_flutter_rust_bridge_codegen::codegen::Config;
use lib_flutter_rust_bridge_codegen::utils::logs::configure_opinionated_logging;
use std::path::Path;

fn main() {
    // Uncomment the line below, if you only want to generate bindings on api directory change.
    //
    // NOTE: This accelerates the build process, but you will need to manually trigger binding
    // generation whenever there are changes to definitions outside of the api directory that it
    // depends on.
    //
    // println!("cargo:rerun-if-changed=src/api");

    // If you want to see logs
    // Alternatively, use `cargo build -vvv` (instead of `cargo build`) to see logs on screen
    configure_opinionated_logging("./logs/", true).unwrap();
    let current_dir = std::env::current_dir().unwrap();
    // Skip code generation when cross-compiling
    if std::env::var("CROSS_COMPILE").is_ok() || std::env::var("TARGET").unwrap_or_default().contains("android") {
        println!("cargo:warning=Cross-compiling detected, skipping code generation");
        return;
    }

    let config_path = Path::new("flutter_rust_bridge.yaml");
    // Execute code generator with auto-detected config
    let config = match Config::from_config_file(config_path.to_str().unwrap()) {
        Ok(Some(config)) => config,
        Ok(None) => {
            panic!("Failed to load configuration from flutter_rust_bridge.yaml");
        }
        Err(e) => {
            println!("cargo:warning=Error loading config file: {}", e);
            panic!("Failed to load configuration: {}", e);
        }
    };

    codegen::generate(config, Default::default())
        .unwrap_or_else(|e| {
            println!("cargo:warning=Error generating code: {}", e);
            panic!("Failed to generate code: {}", e);
        });
}
