pub mod particle;
pub mod forces;
pub mod config;
pub mod spatial;
pub mod renderer;
pub mod presets;

#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub mod wasm;

use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};
use particle::ParticleSystem;
use forces::PhysicsEngine;
use renderer::ParticleRenderer;
use config::{ConfigManager, SimulationConfig, Preset};
use presets::PresetManager;
use spatial::SpatialPartitioning;

pub struct App {
    pub particle_system: ParticleSystem,
    pub physics_engine: PhysicsEngine,
    pub renderer: ParticleRenderer,
    pub config_manager: ConfigManager,
    pub spatial: Option<SpatialPartitioning>,
    pub egui: Egui,
    pub ui_state: UiState,
    pub performance_stats: PerformanceStats,
    pub time_accumulator: f32,
    pub frame_count: u64,
    pub paused: bool,
    pub current_preset: Option<Preset>,
}

#[derive(Default)]
pub struct UiState {
    pub show_settings: bool,
    pub show_performance: bool,
    pub show_force_editor: bool,
    pub selected_preset: usize,
    pub force_strength_slider: f32,
    pub spawn_rate_slider: f32,
    pub particle_count_slider: usize,
}

#[derive(Default)]
pub struct PerformanceStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub update_time_ms: f32,
    pub render_time_ms: f32,
    pub particle_count: usize,
    pub active_forces: usize,
    pub spatial_queries: usize,
}

impl App {
    pub fn new(_app: &nannou::App, window: nannou::window::Id) -> Self {
        let config_manager = ConfigManager::new();
        let config = config_manager.config();
        
        let particle_system = PresetManager::create_particle_system_from_preset(
            &Preset::ParticleLife, 
            config
        );
        
        let physics_engine = PhysicsEngine::new(config.physics.clone());
        let renderer = ParticleRenderer::new(config.rendering.clone());
        
        let spatial = if config.performance.enable_spatial_partitioning {
            Some(SpatialPartitioning::new_quadtree(
                (Vec2::new(-500.0, -500.0), Vec2::new(500.0, 500.0)),
                10,
                8
            ))
        } else {
            None
        };

        let window_ref = _app.window(window).unwrap();
        let egui = Egui::from_window(&window_ref);

        Self {
            particle_system,
            physics_engine,
            renderer,
            config_manager,
            spatial,
            egui,
            ui_state: UiState::default(),
            performance_stats: PerformanceStats::default(),
            time_accumulator: 0.0,
            frame_count: 0,
            paused: false,
            current_preset: Some(Preset::ParticleLife),
        }
    }

    pub fn update(&mut self, _app: &nannou::App, _update: &nannou::event::Update) {
        let dt = _update.since_last.as_secs_f32();
        self.time_accumulator += dt;
        self.frame_count += 1;

        // Update FPS every second
        if self.time_accumulator >= 1.0 {
            self.performance_stats.fps = self.frame_count as f32 / self.time_accumulator;
            self.performance_stats.frame_time_ms = self.time_accumulator * 1000.0 / self.frame_count as f32;
            self.time_accumulator = 0.0;
            self.frame_count = 0;
        }

        if !self.paused {
            let start_time = std::time::Instant::now();
            
            // Update spatial partitioning
            if let Some(ref mut spatial) = self.spatial {
                spatial.update(&self.particle_system.particles);
            }
            
            // Update physics
            self.physics_engine.update(&mut self.particle_system);
            
            // Update particle system
            self.particle_system.update(dt);
            
            self.performance_stats.update_time_ms = start_time.elapsed().as_millis() as f32;
            self.performance_stats.particle_count = self.particle_system.particle_count();
        }

        // Update renderer
        self.renderer.update(&self.particle_system, dt);
        
        // Handle keyboard input
        for key in _app.keys.down.iter() {
            self.handle_key_input(*key);
        }
    }

    pub fn view(&mut self, _app: &nannou::App, frame: &nannou::Frame) {
        let start_time = std::time::Instant::now();
        
        let draw = _app.draw();
        
        // Render particles
        self.renderer.render(&draw, &self.particle_system);
        
        // Draw to frame
        draw.to_frame(_app, &frame).unwrap();
        
        self.performance_stats.render_time_ms = start_time.elapsed().as_millis() as f32;
        
        // Draw UI
        self.egui.set_elapsed_time(frame.elapsed_frames() as f64 / 60.0);
        let ctx = self.egui.begin_frame();
        self.draw_ui(&ctx);
        self.egui.end_frame_and_draw(&frame.device_queue_pair().queue, frame.resolve_target());
    }

    pub fn raw_window_event(&mut self, _app: &nannou::App, event: &nannou::winit::event::WindowEvent) {
        self.egui.handle_raw_event(event);
        
        // Handle mouse events for camera control
        match event {
            nannou::winit::event::WindowEvent::CursorMoved { position, .. } => {
                let screen_size = Vec2::new(
                    _app.main_window().inner_size_points().0,
                    _app.main_window().inner_size_points().1,
                );
                let mouse_pos = Vec2::new(position.x as f32, position.y as f32);
                self.renderer.handle_mouse_input(mouse_pos, screen_size);
            },
            nannou::winit::event::WindowEvent::MouseWheel { delta, .. } => {
                if let nannou::winit::event::MouseScrollDelta::LineDelta(_, y) = delta {
                    self.renderer.handle_zoom(*y);
                }
            },
            _ => {}
        }
    }

    fn handle_key_input(&mut self, key: nannou::event::Key) {
        match key {
            nannou::event::Key::Space => {
                self.paused = !self.paused;
            },
            nannou::event::Key::R => {
                self.reset_simulation();
            },
            nannou::event::Key::Key1 => {
                self.apply_preset(Preset::ParticleLife);
            },
            nannou::event::Key::Key2 => {
                self.apply_preset(Preset::Flocking);
            },
            nannou::event::Key::Key3 => {
                self.apply_preset(Preset::Gravity);
            },
            nannou::event::Key::Key4 => {
                self.apply_preset(Preset::Electromagnetic);
            },
            nannou::event::Key::Key5 => {
                self.apply_preset(Preset::Brownian);
            },
            nannou::event::Key::Key6 => {
                self.apply_preset(Preset::ReactionDiffusion);
            },
            nannou::event::Key::F1 => {
                self.ui_state.show_settings = !self.ui_state.show_settings;
            },
            nannou::event::Key::F2 => {
                self.ui_state.show_performance = !self.ui_state.show_performance;
            },
            nannou::event::Key::C => {
                self.renderer.reset_camera();
            },
            nannou::event::Key::F => {
                self.renderer.focus_on_particles(&self.particle_system);
            },
            _ => {}
        }
    }

    fn draw_ui(&mut self, ctx: &egui::Context) {
        // Main control panel
        egui::SidePanel::left("control_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("🎆 Inochi Particle Life");
                ui.separator();
                
                self.draw_simulation_controls(ui);
                ui.separator();
                
                self.draw_preset_selector(ui);
                ui.separator();
                
                self.draw_particle_controls(ui);
                ui.separator();
                
                self.draw_force_controls(ui);
                ui.separator();
                
                self.draw_rendering_controls(ui);
            });

        // Performance window
        if self.ui_state.show_performance {
            egui::Window::new("📊 Performance Stats")
                .default_size([300.0, 200.0])
                .show(ctx, |ui| {
                    self.draw_performance_stats(ui);
                });
        }

        // Settings window
        if self.ui_state.show_settings {
            egui::Window::new("⚙️ Advanced Settings")
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    self.draw_advanced_settings(ui);
                });
        }

        // Force editor window
        if self.ui_state.show_force_editor {
            egui::Window::new("🔧 Force Editor")
                .default_size([350.0, 400.0])
                .show(ctx, |ui| {
                    self.draw_force_editor(ui);
                });
        }
    }

    fn draw_simulation_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Simulation");
        
        ui.horizontal(|ui| {
            if ui.button(if self.paused { "▶️ Play" } else { "⏸️ Pause" }).clicked() {
                self.paused = !self.paused;
            }
            
            if ui.button("🔄 Reset").clicked() {
                self.reset_simulation();
            }
        });
        
        ui.label(format!("Particles: {}", self.particle_system.particle_count()));
        ui.label(format!("FPS: {:.1}", self.performance_stats.fps));
        
        if ui.button("📊 Performance").clicked() {
            self.ui_state.show_performance = !self.ui_state.show_performance;
        }
    }

    fn draw_preset_selector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Presets");
        
        let presets = Preset::all();
        let preset_names: Vec<&str> = presets.iter().map(|p| p.name()).collect();
        
        if ui.combo_box_with_label("Preset", &preset_names[self.ui_state.selected_preset]).changed() {
            // Combo box selection changed
        }
        
        for (i, preset) in presets.iter().enumerate() {
            if ui.button(preset.name()).clicked() {
                self.ui_state.selected_preset = i;
                self.apply_preset(preset.clone());
            }
        }
    }

    fn draw_particle_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Particles");
        
        let config = self.config_manager.config_mut();
        
        ui.add(egui::Slider::new(&mut config.particles.spawn_rate, 0.0..=100.0)
            .text("Spawn Rate"));
        
        ui.add(egui::Slider::new(&mut config.particles.max_particles, 10..=2000)
            .text("Max Particles"));
        
        ui.add(egui::Slider::new(&mut config.particles.default_size, 0.5..=10.0)
            .text("Default Size"));
    }

    fn draw_force_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Forces");
        
        let config = self.config_manager.config_mut();
        
        ui.checkbox(&mut config.forces.enable_gravity, "Gravity");
        if config.forces.enable_gravity {
            ui.add(egui::Slider::new(&mut config.forces.gravity_strength, 0.0..=1000.0)
                .text("Gravity Strength"));
        }
        
        ui.checkbox(&mut config.forces.enable_damping, "Damping");
        if config.forces.enable_damping {
            ui.add(egui::Slider::new(&mut config.forces.damping_coefficient, 0.0..=0.1)
                .text("Damping"));
        }
        
        ui.checkbox(&mut config.forces.enable_brownian, "Brownian Motion");
        if config.forces.enable_brownian {
            ui.add(egui::Slider::new(&mut config.forces.brownian_intensity, 0.0..=10.0)
                .text("Brownian Intensity"));
        }
        
        if ui.button("🔧 Force Editor").clicked() {
            self.ui_state.show_force_editor = !self.ui_state.show_force_editor;
        }
    }

    fn draw_rendering_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Rendering");
        
        let config = self.config_manager.config_mut();
        
        ui.checkbox(&mut config.rendering.enable_trails, "Particle Trails");
        if config.rendering.enable_trails {
            ui.add(egui::Slider::new(&mut config.rendering.trail_length, 5..=200)
                .text("Trail Length"));
        }
        
        ui.checkbox(&mut config.rendering.show_velocity_vectors, "Velocity Vectors");
        ui.checkbox(&mut config.rendering.show_force_vectors, "Force Vectors");
        ui.checkbox(&mut config.rendering.color_by_velocity, "Color by Velocity");
        ui.checkbox(&mut config.rendering.color_by_energy, "Color by Energy");
        
        ui.add(egui::Slider::new(&mut config.rendering.point_size, 0.1..=10.0)
            .text("Point Size"));
        
        // Update renderer config when changed
        self.renderer.update_config(config.rendering.clone());
    }

    fn draw_performance_stats(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("FPS: {:.1}", self.performance_stats.fps));
        ui.label(format!("Frame Time: {:.2} ms", self.performance_stats.frame_time_ms));
        ui.label(format!("Update Time: {:.2} ms", self.performance_stats.update_time_ms));
        ui.label(format!("Render Time: {:.2} ms", self.performance_stats.render_time_ms));
        ui.separator();
        ui.label(format!("Particles: {}", self.performance_stats.particle_count));
        ui.label(format!("Active Forces: {}", self.performance_stats.active_forces));
        
        if let Some(ref spatial) = self.spatial {
            match spatial {
                SpatialPartitioning::Grid(grid) => {
                    ui.label(format!("Grid Cells: {}", grid.get_cell_count()));
                    ui.label(format!("Max Particles/Cell: {}", grid.get_max_particles_per_cell()));
                },
                SpatialPartitioning::QuadTree(qt) => {
                    let stats = qt.get_statistics();
                    ui.label(format!("QuadTree Nodes: {}", stats.node_count));
                    ui.label(format!("Max Depth: {}", stats.max_depth));
                }
            }
        }
    }

    fn draw_advanced_settings(&mut self, ui: &mut egui::Ui) {
        let config = self.config_manager.config_mut();
        
        ui.collapsing("Physics", |ui| {
            ui.add(egui::Slider::new(&mut config.physics.dt, 0.001..=0.1)
                .text("Time Step"));
            ui.add(egui::Slider::new(&mut config.physics.max_velocity, 10.0..=1000.0)
                .text("Max Velocity"));
            ui.checkbox(&mut config.physics.enable_collisions, "Enable Collisions");
        });
        
        ui.collapsing("Performance", |ui| {
            ui.checkbox(&mut config.performance.enable_spatial_partitioning, "Spatial Partitioning");
            ui.checkbox(&mut config.performance.enable_multithreading, "Multithreading");
            ui.add(egui::Slider::new(&mut config.performance.target_fps, 30.0..=120.0)
                .text("Target FPS"));
        });
        
        ui.separator();
        
        if ui.button("💾 Save Config").clicked() {
            if let Err(e) = self.config_manager.save_to_file("config.json") {
                eprintln!("Failed to save config: {}", e);
            }
        }
        
        if ui.button("📁 Load Config").clicked() {
            match ConfigManager::from_file("config.json") {
                Ok(manager) => {
                    self.config_manager = manager;
                    self.apply_current_config();
                },
                Err(e) => {
                    eprintln!("Failed to load config: {}", e);
                }
            }
        }
        
        if ui.button("🔄 Reset to Defaults").clicked() {
            self.config_manager.reset_to_defaults();
            self.apply_current_config();
        }
    }

    fn draw_force_editor(&mut self, _ui: &mut egui::Ui) {
        // Advanced force editor - placeholder for now
        _ui.label("Force Editor - Coming Soon!");
        _ui.label("This will allow fine-tuning of individual force parameters");
        _ui.label("and creation of custom force interactions.");
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
        
        // Update renderer
        self.renderer.update_config(self.config_manager.config().rendering.clone());
        
        // Update spatial partitioning
        if self.config_manager.config().performance.enable_spatial_partitioning {
            self.spatial = Some(SpatialPartitioning::new_quadtree(
                (Vec2::new(-500.0, -500.0), Vec2::new(500.0, 500.0)),
                10,
                8
            ));
        } else {
            self.spatial = None;
        }
    }

    fn reset_simulation(&mut self) {
        if let Some(ref preset) = self.current_preset.clone() {
            self.apply_preset(preset);
        } else {
            self.particle_system.clear();
        }
        self.renderer.reset_camera();
    }

    fn apply_current_config(&mut self) {
        let config = self.config_manager.config().clone();
        
        // Update physics engine
        self.physics_engine = PhysicsEngine::new(config.physics);
        
        // Update renderer
        self.renderer.update_config(config.rendering);
        
        // Update spatial partitioning
        if config.performance.enable_spatial_partitioning {
            self.spatial = Some(SpatialPartitioning::new_quadtree(
                (Vec2::new(-500.0, -500.0), Vec2::new(500.0, 500.0)),
                10,
                8
            ));
        } else {
            self.spatial = None;
        }
    }

    pub fn get_particle_count(&self) -> usize {
        self.particle_system.particle_count()
    }

    pub fn get_fps(&self) -> f32 {
        self.performance_stats.fps
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}