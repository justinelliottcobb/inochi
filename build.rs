use std::env;

fn main() {
    // Enable WebAssembly SIMD when targeting wasm32
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        println!("cargo:rustc-cfg=web_sys_unstable_apis");
        
        // Set WASM-specific flags
        println!("cargo:rustc-env=CARGO_CFG_TARGET_FEATURE=+simd128");
        
        // Enable rayon for web workers if available
        if env::var("CARGO_FEATURE_WEB_WORKERS").is_ok() {
            println!("cargo:rustc-cfg=web_workers");
        }
    }

    // Set version info
    println!("cargo:rustc-env=CARGO_PKG_VERSION_BUILD={}", env::var("GITHUB_SHA").unwrap_or_else(|_| "dev".to_string()));
}