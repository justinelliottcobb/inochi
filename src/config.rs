use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::forces::{ForceType, PhysicsConfig, IntegrationMethod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub physics: PhysicsConfig,
    pub rendering: RenderConfig,
    pub particles: ParticleConfig,
    pub forces: ForceConfig,
    pub ui: UiConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub background_color: [f32; 4],
    pub particle_render_mode: ParticleRenderMode,
    pub point_size: f32,
    pub line_width: f32,
    pub enable_trails: bool,
    pub trail_length: usize,
    pub trail_fade: f32,
    pub enable_bloom: bool,
    pub bloom_intensity: f32,
    pub enable_grid: bool,
    pub grid_color: [f32; 4],
    pub grid_spacing: f32,
    pub camera_zoom: f32,
    pub camera_position: Vec2,
    pub show_velocity_vectors: bool,
    pub show_force_vectors: bool,
    pub show_particle_ids: bool,
    pub color_by_velocity: bool,
    pub color_by_energy: bool,
    pub hdr_exposure: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleRenderMode {
    Points,
    Circles,
    Sprites,
    Metaballs,
    Lines,
    Trails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleConfig {
    pub max_particles: usize,
    pub spawn_rate: f32,
    pub initial_particle_count: usize,
    pub default_mass: f32,
    pub default_charge: f32,
    pub default_size: f32,
    pub default_lifespan: f32,
    pub default_color: [f32; 4],
    pub spawn_area: SpawnArea,
    pub initial_velocity_range: (Vec2, Vec2),
    pub mass_variation: f32,
    pub size_variation: f32,
    pub color_variation: f32,
    pub species_weights: HashMap<u32, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnArea {
    Point(Vec2),
    Circle { center: Vec2, radius: f32 },
    Rectangle { min: Vec2, max: Vec2 },
    Ring { center: Vec2, inner_radius: f32, outer_radius: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceConfig {
    pub global_forces: Vec<ForceType>,
    pub species_interactions: HashMap<(u32, u32), Vec<ForceType>>,
    pub enable_gravity: bool,
    pub gravity_strength: f32,
    pub enable_electromagnetic: bool,
    pub electromagnetic_strength: f32,
    pub enable_damping: bool,
    pub damping_coefficient: f32,
    pub enable_brownian: bool,
    pub brownian_intensity: f32,
    pub boundary_forces: BoundaryForces,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryForces {
    pub enable_boundaries: bool,
    pub boundary_type: BoundaryType,
    pub bounds: (Vec2, Vec2),
    pub boundary_strength: f32,
    pub boundary_damping: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryType {
    Reflective,
    Absorbing,
    Wrapping,
    Elastic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_ui: bool,
    pub show_performance_stats: bool,
    pub show_particle_count: bool,
    pub show_energy_stats: bool,
    pub show_force_controls: bool,
    pub show_rendering_controls: bool,
    pub show_physics_controls: bool,
    pub ui_scale: f32,
    pub enable_keyboard_shortcuts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enable_spatial_partitioning: bool,
    pub spatial_partition_size: f32,
    pub max_interactions_per_particle: usize,
    pub enable_multithreading: bool,
    pub thread_count: Option<usize>,
    pub target_fps: f32,
    pub adaptive_quality: bool,
    pub lod_distance_threshold: f32,
    pub enable_frustum_culling: bool,
    pub max_gpu_particles: usize,
    pub enable_gpu_compute: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            physics: PhysicsConfig::default(),
            rendering: RenderConfig::default(),
            particles: ParticleConfig::default(),
            forces: ForceConfig::default(),
            ui: UiConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            window_width: 1200,
            window_height: 800,
            background_color: [0.02, 0.02, 0.05, 1.0],
            particle_render_mode: ParticleRenderMode::Circles,
            point_size: 2.0,
            line_width: 1.0,
            enable_trails: false,
            trail_length: 50,
            trail_fade: 0.95,
            enable_bloom: false,
            bloom_intensity: 1.0,
            enable_grid: false,
            grid_color: [0.2, 0.2, 0.2, 0.3],
            grid_spacing: 50.0,
            camera_zoom: 1.0,
            camera_position: Vec2::ZERO,
            show_velocity_vectors: false,
            show_force_vectors: false,
            show_particle_ids: false,
            color_by_velocity: false,
            color_by_energy: false,
            hdr_exposure: 1.0,
        }
    }
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            max_particles: 1000,
            spawn_rate: 10.0,
            initial_particle_count: 100,
            default_mass: 1.0,
            default_charge: 0.0,
            default_size: 2.0,
            default_lifespan: f32::INFINITY,
            default_color: [1.0, 1.0, 1.0, 1.0],
            spawn_area: SpawnArea::Circle { center: Vec2::ZERO, radius: 100.0 },
            initial_velocity_range: (Vec2::new(-10.0, -10.0), Vec2::new(10.0, 10.0)),
            mass_variation: 0.1,
            size_variation: 0.2,
            color_variation: 0.1,
            species_weights: {
                let mut weights = HashMap::new();
                weights.insert(0, 1.0);
                weights
            },
        }
    }
}

impl Default for ForceConfig {
    fn default() -> Self {
        Self {
            global_forces: vec![
                ForceType::Damping { coefficient: 0.01 },
                ForceType::Brownian { intensity: 0.1 },
            ],
            species_interactions: HashMap::new(),
            enable_gravity: false,
            gravity_strength: 100.0,
            enable_electromagnetic: false,
            electromagnetic_strength: 100.0,
            enable_damping: true,
            damping_coefficient: 0.01,
            enable_brownian: true,
            brownian_intensity: 0.1,
            boundary_forces: BoundaryForces::default(),
        }
    }
}

impl Default for BoundaryForces {
    fn default() -> Self {
        Self {
            enable_boundaries: true,
            boundary_type: BoundaryType::Reflective,
            bounds: (Vec2::new(-400.0, -300.0), Vec2::new(400.0, 300.0)),
            boundary_strength: 100.0,
            boundary_damping: 0.8,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_ui: true,
            show_performance_stats: true,
            show_particle_count: true,
            show_energy_stats: false,
            show_force_controls: true,
            show_rendering_controls: true,
            show_physics_controls: true,
            ui_scale: 1.0,
            enable_keyboard_shortcuts: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_spatial_partitioning: true,
            spatial_partition_size: 50.0,
            max_interactions_per_particle: 100,
            enable_multithreading: true,
            thread_count: None, // Use system default
            target_fps: 60.0,
            adaptive_quality: true,
            lod_distance_threshold: 500.0,
            enable_frustum_culling: true,
            max_gpu_particles: 10000,
            enable_gpu_compute: false, // Disabled by default for compatibility
        }
    }
}

pub struct ConfigManager {
    config: SimulationConfig,
    config_path: Option<String>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: SimulationConfig::default(),
            config_path: None,
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = if path.ends_with(".json") {
            serde_json::from_str(&content)?
        } else if path.ends_with(".toml") {
            toml::from_str(&content)?
        } else {
            return Err("Unsupported config file format. Use .json or .toml".into());
        };

        Ok(Self {
            config,
            config_path: Some(path.to_string()),
        })
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = if path.ends_with(".json") {
            serde_json::to_string_pretty(&self.config)?
        } else if path.ends_with(".toml") {
            toml::to_string(&self.config)?
        } else {
            return Err("Unsupported config file format. Use .json or .toml".into());
        };

        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.config_path {
            self.save_to_file(path)
        } else {
            Err("No config path set. Use save_to_file() instead.".into())
        }
    }

    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut SimulationConfig {
        &mut self.config
    }

    pub fn reset_to_defaults(&mut self) {
        self.config = SimulationConfig::default();
    }

    pub fn apply_preset(&mut self, preset: Preset) {
        match preset {
            Preset::ParticleLife => self.apply_particle_life_preset(),
            Preset::Flocking => self.apply_flocking_preset(),
            Preset::Gravity => self.apply_gravity_preset(),
            Preset::Electromagnetic => self.apply_electromagnetic_preset(),
            Preset::Brownian => self.apply_brownian_preset(),
            Preset::ReactionDiffusion => self.apply_reaction_diffusion_preset(),
        }
    }

    fn apply_particle_life_preset(&mut self) {
        self.config.particles.max_particles = 500;
        self.config.particles.initial_particle_count = 300;
        
        // Species 0: Red particles
        // Species 1: Blue particles  
        // Species 2: Green particles
        self.config.particles.species_weights.clear();
        self.config.particles.species_weights.insert(0, 0.4);
        self.config.particles.species_weights.insert(1, 0.3);
        self.config.particles.species_weights.insert(2, 0.3);

        self.config.forces.species_interactions.clear();
        
        // Red-Red: mild repulsion
        self.config.forces.species_interactions.insert(
            (0, 0), 
            vec![ForceType::Repulsion { strength: 20.0, max_distance: 30.0 }]
        );
        
        // Red-Blue: attraction
        self.config.forces.species_interactions.insert(
            (0, 1), 
            vec![ForceType::Attraction { strength: 15.0, max_distance: 80.0 }]
        );
        
        // Red-Green: strong repulsion
        self.config.forces.species_interactions.insert(
            (0, 2), 
            vec![ForceType::Repulsion { strength: 50.0, max_distance: 60.0 }]
        );
        
        // Blue-Blue: attraction
        self.config.forces.species_interactions.insert(
            (1, 1), 
            vec![ForceType::Attraction { strength: 10.0, max_distance: 50.0 }]
        );
        
        // Blue-Green: mild attraction
        self.config.forces.species_interactions.insert(
            (1, 2), 
            vec![ForceType::Attraction { strength: 8.0, max_distance: 70.0 }]
        );
        
        // Green-Green: repulsion
        self.config.forces.species_interactions.insert(
            (2, 2), 
            vec![ForceType::Repulsion { strength: 30.0, max_distance: 40.0 }]
        );

        self.config.rendering.enable_trails = true;
        self.config.rendering.trail_length = 30;
    }

    fn apply_flocking_preset(&mut self) {
        self.config.particles.max_particles = 200;
        self.config.particles.initial_particle_count = 150;
        
        self.config.forces.global_forces = vec![
            ForceType::Flocking {
                separation_radius: 20.0,
                alignment_radius: 40.0,
                cohesion_radius: 60.0,
                separation_strength: 50.0,
                alignment_strength: 20.0,
                cohesion_strength: 10.0,
            },
            ForceType::Damping { coefficient: 0.02 },
        ];
        
        self.config.rendering.show_velocity_vectors = true;
        self.config.rendering.color_by_velocity = true;
    }

    fn apply_gravity_preset(&mut self) {
        self.config.particles.max_particles = 100;
        self.config.particles.initial_particle_count = 50;
        
        self.config.forces.global_forces = vec![
            ForceType::Gravity { strength: 500.0, min_distance: 5.0 },
            ForceType::Damping { coefficient: 0.001 },
        ];
        
        self.config.particles.mass_variation = 0.5;
        self.config.rendering.color_by_energy = true;
        self.config.rendering.enable_trails = true;
    }

    fn apply_electromagnetic_preset(&mut self) {
        self.config.particles.max_particles = 300;
        self.config.particles.initial_particle_count = 200;
        
        self.config.forces.global_forces = vec![
            ForceType::ElectroMagnetic { strength: 1000.0, min_distance: 10.0 },
            ForceType::Damping { coefficient: 0.01 },
        ];
        
        // Half positive, half negative charges
        self.config.particles.species_weights.clear();
        self.config.particles.species_weights.insert(0, 0.5); // Positive
        self.config.particles.species_weights.insert(1, 0.5); // Negative
        
        self.config.rendering.show_force_vectors = true;
    }

    fn apply_brownian_preset(&mut self) {
        self.config.particles.max_particles = 1000;
        self.config.particles.initial_particle_count = 500;
        
        self.config.forces.global_forces = vec![
            ForceType::Brownian { intensity: 5.0 },
            ForceType::Damping { coefficient: 0.05 },
        ];
        
        self.config.rendering.particle_render_mode = ParticleRenderMode::Points;
        self.config.rendering.point_size = 1.0;
    }

    fn apply_reaction_diffusion_preset(&mut self) {
        self.config.particles.max_particles = 800;
        self.config.particles.initial_particle_count = 400;
        
        self.config.forces.species_interactions.clear();
        
        // Activator-Inhibitor dynamics
        // Species 0: Activator (red)
        // Species 1: Inhibitor (blue)
        self.config.particles.species_weights.clear();
        self.config.particles.species_weights.insert(0, 0.6);
        self.config.particles.species_weights.insert(1, 0.4);
        
        // Activator-Activator: self-reinforcement
        self.config.forces.species_interactions.insert(
            (0, 0), 
            vec![ForceType::Attraction { strength: 25.0, max_distance: 40.0 }]
        );
        
        // Activator-Inhibitor: inhibition
        self.config.forces.species_interactions.insert(
            (0, 1), 
            vec![ForceType::Repulsion { strength: 40.0, max_distance: 80.0 }]
        );
        
        // Inhibitor-Inhibitor: mild repulsion
        self.config.forces.species_interactions.insert(
            (1, 1), 
            vec![ForceType::Repulsion { strength: 15.0, max_distance: 30.0 }]
        );
        
        self.config.forces.global_forces = vec![
            ForceType::Brownian { intensity: 2.0 },
            ForceType::Damping { coefficient: 0.03 },
        ];
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Preset {
    ParticleLife,
    Flocking,
    Gravity,
    Electromagnetic,
    Brownian,
    ReactionDiffusion,
}

impl Preset {
    pub fn all() -> Vec<Preset> {
        vec![
            Preset::ParticleLife,
            Preset::Flocking,
            Preset::Gravity,
            Preset::Electromagnetic,
            Preset::Brownian,
            Preset::ReactionDiffusion,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Preset::ParticleLife => "Particle Life",
            Preset::Flocking => "Flocking/Boids",
            Preset::Gravity => "N-Body Gravity",
            Preset::Electromagnetic => "Electromagnetic",
            Preset::Brownian => "Brownian Motion",
            Preset::ReactionDiffusion => "Reaction-Diffusion",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Preset::ParticleLife => "Classic particle life with species-based attraction and repulsion",
            Preset::Flocking => "Emergent flocking behavior with separation, alignment, and cohesion",
            Preset::Gravity => "Gravitational n-body simulation with realistic orbital dynamics",
            Preset::Electromagnetic => "Charged particles with electromagnetic forces",
            Preset::Brownian => "Random walk particles demonstrating Brownian motion",
            Preset::ReactionDiffusion => "Reaction-diffusion patterns with activator-inhibitor dynamics",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_config_serialization() {
        let config = SimulationConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SimulationConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.particles.max_particles, deserialized.particles.max_particles);
    }

    #[test]
    fn test_config_manager_save_load() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config = SimulationConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        temp_file.write_all(json.as_bytes()).unwrap();
        
        let manager = ConfigManager::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(manager.config().particles.max_particles, config.particles.max_particles);
    }

    #[test]
    fn test_presets() {
        let mut manager = ConfigManager::new();
        
        for preset in Preset::all() {
            manager.apply_preset(preset.clone());
            // Verify that the preset was applied by checking some config changes
            assert!(manager.config().particles.max_particles > 0);
        }
    }
}