use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlBuffer, CanvasRenderingContext2d};
use crate::{
    particle::ParticleSystem, 
    forces::PhysicsEngine, 
    config::{ConfigManager, Preset},
    presets::PresetManager
};
use std::cell::RefCell;
use std::rc::Rc;

// Console logging for WASM
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Simplified WASM App state without nannou dependencies
pub struct WasmApp {
    particle_system: ParticleSystem,
    physics_engine: PhysicsEngine,
    config_manager: ConfigManager,
    canvas: Option<HtmlCanvasElement>,
    gl_context: Option<WebGl2RenderingContext>,
    canvas_2d_context: Option<CanvasRenderingContext2d>,
    shader_program: Option<WebGlProgram>,
    vertex_buffer: Option<WebGlBuffer>,
    use_webgl: bool,
    frame_count: u64,
    last_time: f32,
    fps: f32,
    paused: bool,
    current_preset: Option<Preset>,
}

impl WasmApp {
    fn new() -> Self {
        let config_manager = ConfigManager::new();
        let config = config_manager.config();
        
        let particle_system = PresetManager::create_particle_system_from_preset(
            &Preset::ParticleLife, 
            config
        );
        
        let physics_engine = PhysicsEngine::new(config.physics.clone());
        
        console_log!("WasmApp created successfully");
        
        Self {
            particle_system,
            physics_engine,
            config_manager,
            canvas: None,
            gl_context: None,
            canvas_2d_context: None,
            shader_program: None,
            vertex_buffer: None,
            use_webgl: false,
            frame_count: 0,
            last_time: 0.0,
            fps: 0.0,
            paused: false,
            current_preset: Some(Preset::ParticleLife),
        }
    }
    
    fn initialize_canvas(&mut self, canvas_id: &str) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id)
            .ok_or("Canvas element not found")?
            .dyn_into::<HtmlCanvasElement>()?;
        
        // Set canvas size
        canvas.set_width(800);
        canvas.set_height(600);
        
        // Try WebGL first, fallback to Canvas 2D
        if let Ok(Some(ctx)) = canvas.get_context("webgl2") {
            console_log!("Using WebGL2 context");
            let gl_context = ctx.dyn_into::<WebGl2RenderingContext>()?;
            
            // Configure WebGL
            gl_context.viewport(0, 0, 800, 600);
            gl_context.clear_color(0.0, 0.0, 0.0, 1.0);
            gl_context.enable(WebGl2RenderingContext::BLEND);
            gl_context.blend_func(
                WebGl2RenderingContext::SRC_ALPHA,
                WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            );
            
            self.canvas = Some(canvas);
            self.gl_context = Some(gl_context);
            self.use_webgl = true;
            
            // Initialize shaders
            match self.initialize_shaders() {
                Ok(_) => {
                    console_log!("Canvas and WebGL2 context initialized successfully");
                    return Ok(());
                }
                Err(e) => {
                    console_log!("WebGL shader initialization failed: {:?}", e);
                    console_log!("Falling back to Canvas 2D");
                }
            }
        } else if let Ok(Some(ctx)) = canvas.get_context("webgl") {
            console_log!("WebGL2 not available, trying WebGL1");
            match ctx.dyn_into::<WebGl2RenderingContext>() {
                Ok(gl_context) => {
                    console_log!("Using WebGL1 context as WebGL2");
                    
                    // Configure WebGL
                    gl_context.viewport(0, 0, 800, 600);
                    gl_context.clear_color(0.0, 0.0, 0.0, 1.0);
                    gl_context.enable(WebGl2RenderingContext::BLEND);
                    gl_context.blend_func(
                        WebGl2RenderingContext::SRC_ALPHA,
                        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
                    );
                    
                    self.canvas = Some(canvas.clone());
                    self.gl_context = Some(gl_context);
                    self.use_webgl = true;
                    
                    // Initialize shaders
                    match self.initialize_shaders() {
                        Ok(_) => {
                            console_log!("Canvas and WebGL1 context initialized successfully");
                            return Ok(());
                        }
                        Err(e) => {
                            console_log!("WebGL shader initialization failed: {:?}", e);
                            console_log!("Falling back to Canvas 2D");
                        }
                    }
                }
                Err(_) => {
                    console_log!("Failed to use WebGL1 context, falling back to Canvas 2D");
                }
            }
        }
        
        // Fallback to Canvas 2D
        console_log!("Initializing Canvas 2D fallback renderer");
        let canvas_2d = canvas.get_context("2d")?
            .ok_or("Failed to get 2D context")?
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        self.canvas = Some(canvas);
        self.canvas_2d_context = Some(canvas_2d);
        self.use_webgl = false;
        
        console_log!("Canvas 2D context initialized successfully");
        Ok(())
    }
    
    fn initialize_shaders(&mut self) -> Result<(), JsValue> {
        let gl = self.gl_context.as_ref().unwrap();
        
        let vertex_shader_source = r#"
            attribute vec2 a_position;
            attribute vec4 a_color;
            attribute float a_size;
            
            uniform vec2 u_resolution;
            
            varying vec4 v_color;
            
            void main() {
                // Convert from pixel coordinates to clip space
                vec2 clipspace = ((a_position / u_resolution) * 2.0) - 1.0;
                gl_Position = vec4(clipspace * vec2(1, -1), 0, 1);
                gl_PointSize = a_size;
                v_color = a_color;
            }
        "#;
        
        let fragment_shader_source = r#"
            precision mediump float;
            
            varying vec4 v_color;
            
            void main() {
                // Create circular particles
                vec2 center = gl_PointCoord - vec2(0.5, 0.5);
                float dist = length(center);
                if (dist > 0.5) {
                    discard;
                }
                
                // Soft edges - simpler version for mobile compatibility
                float alpha = 1.0 - (dist * 2.0);
                alpha = max(0.0, alpha);
                gl_FragColor = vec4(v_color.rgb, v_color.a * alpha);
            }
        "#;
        
        let vertex_shader = self.compile_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, vertex_shader_source)?;
        let fragment_shader = self.compile_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, fragment_shader_source)?;
        
        let program = gl.create_program().ok_or("Unable to create shader program")?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);
        
        if gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
            self.shader_program = Some(program);
            
            // Create vertex buffer
            let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
            self.vertex_buffer = Some(buffer);
            
            console_log!("Shaders compiled and linked successfully");
            Ok(())
        } else {
            let info = gl.get_program_info_log(&program)
                .unwrap_or_else(|| "Unknown error creating shader program".into());
            console_log!("Shader program link error: {}", info);
            Err(JsValue::from_str(&format!("Shader program link error: {}", info)))
        }
    }
    
    fn compile_shader(&self, gl: &WebGl2RenderingContext, shader_type: u32, source: &str) -> Result<WebGlShader, JsValue> {
        let shader = gl.create_shader(shader_type).ok_or("Unable to create shader")?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);
        
        if gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
            Ok(shader)
        } else {
            let info = gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error compiling shader".into());
            console_log!("Shader compile error: {}", info);
            Err(JsValue::from_str(&format!("Shader compile error: {}", info)))
        }
    }
    
    fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        
        // Update physics
        self.physics_engine.update(&mut self.particle_system);
        
        // Update particle system
        self.particle_system.update(dt);
        
        // Update FPS
        self.frame_count += 1;
        if self.frame_count % 60 == 0 {
            // Update FPS every 60 frames
            self.fps = 60.0 / dt.max(0.001);
        }
    }
    
    fn render(&self) {
        if self.use_webgl {
            self.render_webgl();
        } else {
            self.render_canvas_2d();
        }
    }
    
    fn render_webgl(&self) {
        let gl = match &self.gl_context {
            Some(gl) => gl,
            None => {
                console_log!("No WebGL context available for rendering");
                return;
            },
        };
        
        let program = match &self.shader_program {
            Some(program) => program,
            None => {
                console_log!("No shader program available for rendering");
                return;
            },
        };
        
        let buffer = match &self.vertex_buffer {
            Some(buffer) => buffer,
            None => {
                console_log!("No vertex buffer available for rendering");
                return;
            },
        };
        
        // Clear the canvas
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        
        // Prepare particle data
        let particles = &self.particle_system.particles;
        if particles.is_empty() {
            if self.frame_count % 60 == 0 {
                console_log!("No particles to render");
            }
            return;
        }
        
        if self.frame_count % 60 == 0 {
            console_log!("Rendering {} particles with WebGL", particles.len());
        }
        
        // Create vertex data: [x, y, r, g, b, a, size] for each particle
        let mut vertex_data = Vec::with_capacity(particles.len() * 7);
        
        for particle in particles {
            // Position (center canvas at 400, 300)
            vertex_data.push(particle.position.x + 400.0);
            vertex_data.push(particle.position.y + 300.0);
            
            // Color
            vertex_data.push(particle.color[0]);
            vertex_data.push(particle.color[1]);
            vertex_data.push(particle.color[2]);
            vertex_data.push(particle.color[3]);
            
            // Size (make particles more visible)
            vertex_data.push(particle.size * 4.0);
        }
        
        // Upload vertex data
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));
        
        // Convert to Float32Array for WebGL
        let vertex_array = js_sys::Float32Array::from(vertex_data.as_slice());
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertex_array,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
        
        // Use shader program
        gl.use_program(Some(program));
        
        // Set uniforms
        let resolution_location = gl.get_uniform_location(program, "u_resolution");
        gl.uniform2f(resolution_location.as_ref(), 800.0, 600.0);
        
        // Set up vertex attributes
        let position_location = gl.get_attrib_location(program, "a_position") as u32;
        let color_location = gl.get_attrib_location(program, "a_color") as u32;
        let size_location = gl.get_attrib_location(program, "a_size") as u32;
        
        // Position attribute (2 floats, offset 0)
        gl.enable_vertex_attrib_array(position_location);
        gl.vertex_attrib_pointer_with_i32(
            position_location,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            7 * 4, // stride: 7 floats * 4 bytes
            0,     // offset
        );
        
        // Color attribute (4 floats, offset 2*4)
        gl.enable_vertex_attrib_array(color_location);
        gl.vertex_attrib_pointer_with_i32(
            color_location,
            4,
            WebGl2RenderingContext::FLOAT,
            false,
            7 * 4, // stride
            2 * 4, // offset
        );
        
        // Size attribute (1 float, offset 6*4)
        gl.enable_vertex_attrib_array(size_location);
        gl.vertex_attrib_pointer_with_i32(
            size_location,
            1,
            WebGl2RenderingContext::FLOAT,
            false,
            7 * 4, // stride
            6 * 4, // offset
        );
        
        // Draw particles as points
        gl.draw_arrays(WebGl2RenderingContext::POINTS, 0, particles.len() as i32);
        
        // Check for WebGL errors
        let error = gl.get_error();
        if error != WebGl2RenderingContext::NO_ERROR && self.frame_count % 60 == 0 {
            console_log!("WebGL error during rendering: {}", error);
        }
        
        // Clean up
        gl.disable_vertex_attrib_array(position_location);
        gl.disable_vertex_attrib_array(color_location);
        gl.disable_vertex_attrib_array(size_location);
    }
    
    fn render_canvas_2d(&self) {
        let ctx = match &self.canvas_2d_context {
            Some(ctx) => ctx,
            None => {
                console_log!("No Canvas 2D context available for rendering");
                return;
            },
        };
        
        // Clear the canvas
        ctx.clear_rect(0.0, 0.0, 800.0, 600.0);
        ctx.set_fill_style(&"black".into());
        ctx.fill_rect(0.0, 0.0, 800.0, 600.0);
        
        // Prepare particle data
        let particles = &self.particle_system.particles;
        if particles.is_empty() {
            if self.frame_count % 60 == 0 {
                console_log!("No particles to render");
            }
            return;
        }
        
        if self.frame_count % 60 == 0 {
            console_log!("Rendering {} particles with Canvas 2D", particles.len());
        }
        
        // Draw each particle as a circle
        for particle in particles {
            let x = particle.position.x + 400.0;
            let y = particle.position.y + 300.0;
            let radius = particle.size * 2.0; // Make particles visible
            
            // Convert color to CSS format
            let color = format!(
                "rgba({}, {}, {}, {})",
                (particle.color[0] * 255.0) as u8,
                (particle.color[1] * 255.0) as u8,
                (particle.color[2] * 255.0) as u8,
                particle.color[3]
            );
            
            ctx.set_fill_style(&color.into());
            ctx.begin_path();
            ctx.arc(x as f64, y as f64, radius as f64, 0.0, 2.0 * std::f64::consts::PI).unwrap();
            ctx.fill();
        }
    }
    
    fn apply_preset(&mut self, preset: Preset) {
        self.config_manager.apply_preset(preset.clone());
        self.current_preset = Some(preset.clone());
        
        // Recreate particle system with new preset
        self.particle_system = PresetManager::create_particle_system_from_preset(
            &preset,
            self.config_manager.config()
        );
        
        // Update physics engine
        self.physics_engine = PhysicsEngine::new(self.config_manager.config().physics.clone());
        
        console_log!("Applied preset: {:?}", preset);
    }
    
    fn reset_simulation(&mut self) {
        if let Some(ref preset) = self.current_preset.clone() {
            self.apply_preset(preset.clone());
        } else {
            self.particle_system.clear();
        }
    }
}

// Global state for the WebAssembly version
static mut GLOBAL_APP: Option<Rc<RefCell<WasmApp>>> = None;

#[wasm_bindgen(start)]
pub fn wasm_main() {
    // Simplified initialization - remove panic hook that might cause issues
    console_log!("WASM module initialized");
}

#[wasm_bindgen]
pub fn start_simulation() -> bool {
    console_log!("Starting simulation in WASM mode...");
    
    // Initialize the app
    let app = WasmApp::new();
    let app_rc = Rc::new(RefCell::new(app));
    
    // Initialize canvas - handle errors without Result/JsValue
    match app_rc.borrow_mut().initialize_canvas("nannou-canvas") {
        Ok(_) => {
            console_log!("Canvas initialized successfully");
        },
        Err(e) => {
            console_log!("Failed to initialize canvas: {:?}", e);
            return false;
        }
    }
    
    // Store globally
    unsafe {
        GLOBAL_APP = Some(app_rc.clone());
    }
    
    // Start the render loop
    match start_render_loop(app_rc) {
        Ok(_) => {
            console_log!("WASM simulation started successfully");
            true
        },
        Err(e) => {
            console_log!("Failed to start render loop: {:?}", e);
            false
        }
    }
}

fn start_render_loop(app: Rc<RefCell<WasmApp>>) -> Result<(), &'static str> {
    // Extremely simple approach - just start the first frame
    // The update will be triggered manually from JavaScript
    console_log!("Render loop initialization started");
    
    // Do one initial update and render to verify everything works
    if let Ok(mut app_ref) = app.try_borrow_mut() {
        app_ref.update(0.016);
    } else {
        return Err("Failed to borrow app for update");
    }
    
    if let Ok(app_ref) = app.try_borrow() {
        app_ref.render();
    } else {
        return Err("Failed to borrow app for render");
    }
    
    console_log!("Initial render completed");
    Ok(())
}

#[wasm_bindgen]
pub fn update_and_render() {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        // Update simulation
        if let Ok(mut app_ref) = app_rc.try_borrow_mut() {
            app_ref.update(0.016); // ~60 FPS
        }
        // Render
        if let Ok(app_ref) = app_rc.try_borrow() {
            app_ref.render();
        }
    }
}

#[wasm_bindgen]
pub fn reset_simulation() {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        app_rc.borrow_mut().reset_simulation();
        console_log!("Simulation reset");
    }
}

#[wasm_bindgen]
pub fn toggle_pause() {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        let mut app = app_rc.borrow_mut();
        app.paused = !app.paused;
        console_log!("Simulation pause toggled: {}", app.paused);
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
        _ => {
            console_log!("Unknown preset: {}", preset_name);
            return;
        }
    };
    
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        app_rc.borrow_mut().apply_preset(preset);
    }
}

#[wasm_bindgen]
pub fn get_particle_count() -> usize {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        return app_rc.borrow().particle_system.particles.len();
    }
    0
}

#[wasm_bindgen]
pub fn get_fps() -> f32 {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        return app_rc.borrow().fps;
    }
    0.0
}

// Note: Full nannou WASM integration would require a proper model function
// This is commented out for now as we're doing a simpler test
// fn model(app: &nannou::App) -> App {
//     let window_id = app.main_window().id();
//     App::new(app, window_id)
// }


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
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        use crate::particle::Particle;
        use glam::Vec2;
        
        let particle = Particle::new(Vec2::new(x, y))
            .with_species(species_id)
            .with_size(2.0 + (species_id as f32 * 0.5))
            .with_color(crate::presets::PresetManager::get_species_color(species_id));
        
        app_rc.borrow_mut().particle_system.add_particle(particle);
        console_log!("Added particle at ({}, {})", x, y);
    }
}

#[wasm_bindgen]
pub fn get_particles() -> Vec<f32> {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        let app = app_rc.borrow();
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
    Vec::new()
}

// Camera and interaction functions - simplified for WASM
#[wasm_bindgen]
pub fn set_camera_position(x: f32, y: f32) {
    // TODO: Implement camera controls for WASM renderer
    console_log!("Camera position: ({}, {})", x, y);
}

#[wasm_bindgen]
pub fn set_camera_zoom(zoom: f32) {
    // TODO: Implement zoom for WASM renderer
    console_log!("Camera zoom: {}", zoom);
}

#[wasm_bindgen]
pub fn handle_mouse_drag(dx: f32, dy: f32) {
    // TODO: Implement pan for WASM renderer
    console_log!("Mouse drag: ({}, {})", dx, dy);
}

#[wasm_bindgen]
pub fn handle_mouse_wheel(delta: f32) {
    // TODO: Implement wheel zoom for WASM renderer
    console_log!("Mouse wheel: {}", delta);
}

#[wasm_bindgen]
pub fn set_force_strength(force_type: &str, strength: f32) {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        let mut app = app_rc.borrow_mut();
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
        
        // Update physics engine with new config
        app.physics_engine = PhysicsEngine::new(config.physics.clone());
        console_log!("Updated {} force to strength {}", force_type, strength);
    }
}

#[wasm_bindgen]
pub fn set_spawn_rate(rate: f32) {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        app_rc.borrow_mut().particle_system.spawn_rate = rate.max(0.0);
        console_log!("Set spawn rate to {}", rate);
    }
}

#[wasm_bindgen]
pub fn enable_trails(enable: bool) {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        let mut app = app_rc.borrow_mut();
        let config = app.config_manager.config_mut();
        config.rendering.enable_trails = enable;
        console_log!("Trails enabled: {}", enable);
    }
}

#[wasm_bindgen]
pub fn set_background_color(r: f32, g: f32, b: f32, a: f32) {
    console_log!("Background color set to ({}, {}, {}, {}) - WebGL not enabled", r, g, b, a);
}

// Performance monitoring functions
#[wasm_bindgen]
pub fn get_performance_stats() -> Vec<f32> {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        let app = app_rc.borrow();
        vec![
            app.fps,
            16.67, // Approximate frame time for 60fps
            1.0,   // Approximate update time
            1.0,   // Approximate render time  
            app.particle_system.particles.len() as f32,
        ]
    } else {
        vec![0.0; 5]
    }
}

// Configuration export/import
#[wasm_bindgen]
pub fn export_config() -> String {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        let app = app_rc.borrow();
        match serde_json::to_string_pretty(app.config_manager.config()) {
            Ok(json) => json,
            Err(_) => "{}".to_string(),
        }
    } else {
        "{}".to_string()
    }
}

#[wasm_bindgen]
pub fn import_config(config_json: &str) -> bool {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        let mut app = app_rc.borrow_mut();
        match serde_json::from_str(config_json) {
            Ok(config) => {
                *app.config_manager.config_mut() = config;
                // Update physics engine with new config
                app.physics_engine = PhysicsEngine::new(app.config_manager.config().physics.clone());
                console_log!("Config imported successfully");
                true
            },
            Err(e) => {
                console_log!("Failed to import config: {:?}", e);
                false
            }
        }
    } else {
        false
    }
}

// Utility functions for debugging
#[wasm_bindgen]
pub fn log_particle_info(index: usize) {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        let app = app_rc.borrow();
        if let Some(particle) = app.particle_system.particles.get(index) {
            console_log!("Particle {}: pos=({:.2}, {:.2}), vel=({:.2}, {:.2}), species={}", 
                index, 
                particle.position.x, particle.position.y,
                particle.velocity.x, particle.velocity.y,
                particle.species_id
            );
        } else {
            console_log!("Particle {} not found", index);
        }
    }
}

#[wasm_bindgen]
pub fn get_system_info() -> String {
    if let Some(ref app_rc) = unsafe { &GLOBAL_APP } {
        let app = app_rc.borrow();
        format!(
            "Particles: {}, Energy: {:.2}, Center of Mass: ({:.2}, {:.2})",
            app.particle_system.particle_count(),
            app.particle_system.total_energy(),
            app.particle_system.center_of_mass().x,
            app.particle_system.center_of_mass().y
        )
    } else {
        "System not initialized".to_string()
    }
}