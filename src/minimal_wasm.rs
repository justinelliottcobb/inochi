use wasm_bindgen::prelude::*;

// Console logging for WASM
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_log!("Minimal WASM module initialized");
}

#[wasm_bindgen]
pub fn start_simulation() -> bool {
    console_log!("Starting minimal simulation");
    true
}

#[wasm_bindgen]
pub fn get_particle_count() -> usize {
    42
}

#[wasm_bindgen]
pub fn get_fps() -> f32 {
    60.0
}

#[wasm_bindgen]
pub fn reset_simulation() {
    console_log!("Reset simulation called");
}

#[wasm_bindgen]
pub fn toggle_pause() {
    console_log!("Toggle pause called");
}

#[wasm_bindgen]
pub fn change_preset(preset_name: &str) {
    console_log!("Change preset called: {}", preset_name);
}

#[wasm_bindgen]
pub fn update_and_render() {
    // Do nothing for now
}