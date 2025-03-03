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

    // Print current working directory for debugging
    println!("cargo:warning=Current working directory: {:?}", std::env::current_dir().unwrap());

    let config_path = Path::new("../flutter_rust_bridge.yaml");
    println!("cargo:warning=Looking for config file at: {:?}", config_path);
    println!("cargo:warning=Config file exists: {}", config_path.exists());

    // Execute code generator with auto-detected config
    let config = match Config::from_config_file("../flutter_rust_bridge.yaml") {
        Ok(Some(config)) => config,
        Ok(None) => {
            println!("cargo:warning=Config file was found but no configuration was loaded");
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
