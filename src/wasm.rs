use wasm_bindgen::prelude::*;
use nannou::prelude::*;
use crate::{App, config::Preset};
use std::sync::Mutex;

// Global state for the WebAssembly version
static mut GLOBAL_APP: Option<Mutex<App>> = None;

#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub async fn start_simulation() -> Result<(), JsValue> {
    let app = nannou::app(model).build()?;
    
    unsafe {
        GLOBAL_APP = Some(Mutex::new(app));
    }
    
    Ok(())
}

#[wasm_bindgen]
pub fn reset_simulation() {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            app.reset_simulation();
        }
    }
}

#[wasm_bindgen]
pub fn toggle_pause() {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            app.set_paused(!app.is_paused());
        }
    }
}

#[wasm_bindgen]
pub fn change_preset(preset_name: &str) {
    let preset = match preset_name {
        "ParticleLife" => Preset::ParticleLife,
        "Flocking" => Preset::Flocking,
        "Gravity" => Preset::Gravity,
        "Electromagnetic" => Preset::Electromagnetic,
        "Brownian" => Preset::Brownian,
        "ReactionDiffusion" => Preset::ReactionDiffusion,
        _ => return,
    };
    
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            app.apply_preset(preset);
        }
    }
}

#[wasm_bindgen]
pub fn get_particle_count() -> usize {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(app) = app_mutex.lock() {
            return app.get_particle_count();
        }
    }
    0
}

#[wasm_bindgen]
pub fn get_fps() -> f32 {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(app) = app_mutex.lock() {
            return app.get_fps();
        }
    }
    0.0
}

fn model(app: &nannou::App) -> App {
    // Find a canvas element in the document
    let canvas = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.get_element_by_id("nannou-canvas"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok());
    
    let window_id = if let Some(canvas) = canvas {
        app.new_window()
            .title("Inochi - Particle Life System")
            .size(canvas.width(), canvas.height())
            .canvas(canvas)
            .build()
            .unwrap()
    } else {
        app.new_window()
            .title("Inochi - Particle Life System")
            .size(1200, 800)
            .build()
            .unwrap()
    };

    App::new(app, window_id)
}

// Export types for TypeScript
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct WasmParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub mass: f32,
    pub charge: f32,
    pub species_id: u32,
    pub size: f32,
}

#[wasm_bindgen]
impl WasmParticle {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32) -> WasmParticle {
        WasmParticle {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            mass: 1.0,
            charge: 0.0,
            species_id: 0,
            size: 2.0,
        }
    }
    
    #[wasm_bindgen(getter)]
    pub fn position(&self) -> Vec<f32> {
        vec![self.x, self.y]
    }
    
    #[wasm_bindgen(getter)]
    pub fn velocity(&self) -> Vec<f32> {
        vec![self.vx, self.vy]
    }
}

#[wasm_bindgen]
pub fn add_particle(x: f32, y: f32, species_id: u32) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            use crate::particle::Particle;
            use glam::Vec2;
            
            let particle = Particle::new(Vec2::new(x, y))
                .with_species(species_id)
                .with_size(2.0 + (species_id as f32 * 0.5))
                .with_color(crate::presets::PresetManager::get_species_color(species_id));
            
            app.particle_system.add_particle(particle);
        }
    }
}

#[wasm_bindgen]
pub fn get_particles() -> Vec<f32> {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(app) = app_mutex.lock() {
            let mut data = Vec::new();
            
            for particle in &app.particle_system.particles {
                data.push(particle.position.x);
                data.push(particle.position.y);
                data.push(particle.velocity.x);
                data.push(particle.velocity.y);
                data.push(particle.color[0]);
                data.push(particle.color[1]);
                data.push(particle.color[2]);
                data.push(particle.color[3]);
                data.push(particle.size);
                data.push(particle.species_id as f32);
            }
            
            return data;
        }
    }
    Vec::new()
}

#[wasm_bindgen]
pub fn set_camera_position(x: f32, y: f32) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            use glam::Vec2;
            app.renderer.camera.position = Vec2::new(x, y);
        }
    }
}

#[wasm_bindgen]
pub fn set_camera_zoom(zoom: f32) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            app.renderer.camera.zoom = zoom.max(0.1).min(10.0);
        }
    }
}

#[wasm_bindgen]
pub fn handle_mouse_drag(dx: f32, dy: f32) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            app.renderer.handle_pan(nannou::geom::Vec2::new(dx, dy));
        }
    }
}

#[wasm_bindgen]
pub fn handle_mouse_wheel(delta: f32) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            app.renderer.handle_zoom(delta);
        }
    }
}

#[wasm_bindgen]
pub fn set_force_strength(force_type: &str, strength: f32) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            let config = app.config_manager.config_mut();
            
            match force_type {
                "gravity" => {
                    config.forces.gravity_strength = strength;
                    config.forces.enable_gravity = strength > 0.0;
                },
                "damping" => {
                    config.forces.damping_coefficient = strength;
                    config.forces.enable_damping = strength > 0.0;
                },
                "brownian" => {
                    config.forces.brownian_intensity = strength;
                    config.forces.enable_brownian = strength > 0.0;
                },
                _ => {}
            }
            
            // Apply the updated configuration
            app.apply_current_config();
        }
    }
}

#[wasm_bindgen]
pub fn set_spawn_rate(rate: f32) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            app.particle_system.spawn_rate = rate.max(0.0);
        }
    }
}

#[wasm_bindgen]
pub fn enable_trails(enable: bool) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            let config = app.config_manager.config_mut();
            config.rendering.enable_trails = enable;
            app.renderer.update_config(config.rendering.clone());
        }
    }
}

#[wasm_bindgen]
pub fn set_background_color(r: f32, g: f32, b: f32, a: f32) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            let config = app.config_manager.config_mut();
            config.rendering.background_color = [r, g, b, a];
            app.renderer.update_config(config.rendering.clone());
        }
    }
}

// Performance monitoring functions
#[wasm_bindgen]
pub fn get_performance_stats() -> Vec<f32> {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(app) = app_mutex.lock() {
            vec![
                app.performance_stats.fps,
                app.performance_stats.frame_time_ms,
                app.performance_stats.update_time_ms,
                app.performance_stats.render_time_ms,
                app.performance_stats.particle_count as f32,
            ]
        } else {
            vec![0.0; 5]
        }
    } else {
        vec![0.0; 5]
    }
}

// Configuration export/import
#[wasm_bindgen]
pub fn export_config() -> String {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(app) = app_mutex.lock() {
            match serde_json::to_string_pretty(app.config_manager.config()) {
                Ok(json) => json,
                Err(_) => "{}".to_string(),
            }
        } else {
            "{}".to_string()
        }
    } else {
        "{}".to_string()
    }
}

#[wasm_bindgen]
pub fn import_config(config_json: &str) -> bool {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(mut app) = app_mutex.lock() {
            match serde_json::from_str(config_json) {
                Ok(config) => {
                    *app.config_manager.config_mut() = config;
                    app.apply_current_config();
                    true
                },
                Err(_) => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

// Utility functions for debugging
#[wasm_bindgen]
pub fn log_particle_info(index: usize) {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(app) = app_mutex.lock() {
            if let Some(particle) = app.particle_system.particles.get(index) {
                console_log!("Particle {}: pos=({:.2}, {:.2}), vel=({:.2}, {:.2}), species={}", 
                    index, 
                    particle.position.x, particle.position.y,
                    particle.velocity.x, particle.velocity.y,
                    particle.species_id
                );
            }
        }
    }
}

#[wasm_bindgen]
pub fn get_system_info() -> String {
    if let Some(ref app_mutex) = unsafe { &GLOBAL_APP } {
        if let Ok(app) = app_mutex.lock() {
            format!(
                "Particles: {}, Energy: {:.2}, Center of Mass: ({:.2}, {:.2})",
                app.particle_system.particle_count(),
                app.particle_system.total_energy(),
                app.particle_system.center_of_mass().x,
                app.particle_system.center_of_mass().y
            )
        } else {
            "System not available".to_string()
        }
    } else {
        "System not initialized".to_string()
    }
}