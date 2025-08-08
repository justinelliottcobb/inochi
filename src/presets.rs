use glam::Vec2;
use rand::Rng;
use crate::particle::{Particle, ParticleSystem};
use crate::config::{SimulationConfig, Preset, ConfigManager};
use crate::forces::{ForceType, InteractionMatrix};

pub struct PresetManager;

impl PresetManager {
    pub fn create_particle_system_from_preset(preset: &Preset, config: &SimulationConfig) -> ParticleSystem {
        let mut system = ParticleSystem::new(config.particles.max_particles);
        
        // Set system properties
        system.spawn_rate = config.particles.spawn_rate;
        system.damping = config.forces.damping_coefficient;
        
        if let Some((min_bounds, max_bounds)) = Self::get_spawn_bounds(config) {
            system.set_bounds(min_bounds, max_bounds);
        }

        // Generate initial particles based on preset
        match preset {
            Preset::ParticleLife => Self::create_particle_life_system(&mut system, config),
            Preset::Flocking => Self::create_flocking_system(&mut system, config),
            Preset::Gravity => Self::create_gravity_system(&mut system, config),
            Preset::Electromagnetic => Self::create_electromagnetic_system(&mut system, config),
            Preset::Brownian => Self::create_brownian_system(&mut system, config),
            Preset::ReactionDiffusion => Self::create_reaction_diffusion_system(&mut system, config),
        }

        system
    }

    fn get_spawn_bounds(config: &SimulationConfig) -> Option<(Vec2, Vec2)> {
        if config.forces.boundary_forces.enable_boundaries {
            Some(config.forces.boundary_forces.bounds)
        } else {
            Some((
                Vec2::new(-400.0, -300.0),
                Vec2::new(400.0, 300.0),
            ))
        }
    }

    fn create_particle_life_system(system: &mut ParticleSystem, config: &SimulationConfig) {
        let mut rng = rand::thread_rng();
        let bounds = Self::get_spawn_bounds(config).unwrap_or((
            Vec2::new(-200.0, -200.0),
            Vec2::new(200.0, 200.0),
        ));

        // Create particles with different species
        let species_weights = &config.particles.species_weights;
        let total_weight: f32 = species_weights.values().sum();
        
        for _ in 0..config.particles.initial_particle_count {
            // Select species based on weights
            let mut species_id = 0;
            let mut weight_sum = 0.0;
            let target_weight = rng.gen::<f32>() * total_weight;
            
            for (&id, &weight) in species_weights {
                weight_sum += weight;
                if target_weight <= weight_sum {
                    species_id = id;
                    break;
                }
            }

            // Random position within bounds
            let position = Vec2::new(
                rng.gen_range(bounds.0.x..bounds.1.x),
                rng.gen_range(bounds.0.y..bounds.1.y),
            );

            // Random initial velocity
            let velocity = Vec2::new(
                rng.gen_range(config.particles.initial_velocity_range.0.x..config.particles.initial_velocity_range.1.x),
                rng.gen_range(config.particles.initial_velocity_range.0.y..config.particles.initial_velocity_range.1.y),
            );

            let color = Self::get_species_color(species_id);
            
            let particle = Particle::new(position)
                .with_velocity(velocity)
                .with_mass(config.particles.default_mass * (1.0 + (rng.gen::<f32>() - 0.5) * config.particles.mass_variation))
                .with_size(config.particles.default_size * (1.0 + (rng.gen::<f32>() - 0.5) * config.particles.size_variation))
                .with_species(species_id)
                .with_color(color)
                .with_lifespan(config.particles.default_lifespan);

            system.add_particle(particle);
        }
    }

    fn create_flocking_system(system: &mut ParticleSystem, config: &SimulationConfig) {
        let mut rng = rand::thread_rng();
        let bounds = Self::get_spawn_bounds(config).unwrap_or((
            Vec2::new(-300.0, -300.0),
            Vec2::new(300.0, 300.0),
        ));

        for _ in 0..config.particles.initial_particle_count {
            // Create small clusters of particles
            let cluster_center = Vec2::new(
                rng.gen_range(bounds.0.x..bounds.1.x),
                rng.gen_range(bounds.0.y..bounds.1.y),
            );
            
            let position = cluster_center + Vec2::new(
                rng.gen_range(-30.0..30.0),
                rng.gen_range(-30.0..30.0),
            );

            // Initial velocity pointing in a common direction with some variation
            let base_direction = Vec2::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5).normalize_or_zero();
            let velocity = base_direction * rng.gen_range(20.0..50.0);

            let particle = Particle::new(position)
                .with_velocity(velocity)
                .with_mass(1.0)
                .with_size(3.0)
                .with_species(0)
                .with_color([0.8, 0.8, 1.0, 1.0])
                .with_lifespan(f32::INFINITY);

            system.add_particle(particle);
        }
    }

    fn create_gravity_system(system: &mut ParticleSystem, config: &SimulationConfig) {
        let mut rng = rand::thread_rng();
        
        // Create a central massive body
        let central_mass = Particle::new(Vec2::ZERO)
            .with_mass(100.0)
            .with_size(10.0)
            .with_velocity(Vec2::ZERO)
            .with_species(0)
            .with_color([1.0, 1.0, 0.0, 1.0])
            .with_lifespan(f32::INFINITY);
        
        system.add_particle(central_mass);

        // Create orbiting bodies
        for _ in 1..config.particles.initial_particle_count {
            let distance = rng.gen_range(50.0..300.0);
            let angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
            
            let position = Vec2::new(
                distance * angle.cos(),
                distance * angle.sin(),
            );

            // Calculate orbital velocity (simplified)
            let orbital_speed = (100.0 / distance).sqrt() * 20.0; // G*M/r approximation
            let velocity = Vec2::new(-orbital_speed * angle.sin(), orbital_speed * angle.cos());

            let mass = rng.gen_range(0.5..3.0);
            let size = 2.0 + mass;

            let particle = Particle::new(position)
                .with_velocity(velocity)
                .with_mass(mass)
                .with_size(size)
                .with_species(1)
                .with_color([0.8, 0.6, 1.0, 1.0])
                .with_lifespan(f32::INFINITY);

            system.add_particle(particle);
        }
    }

    fn create_electromagnetic_system(system: &mut ParticleSystem, config: &SimulationConfig) {
        let mut rng = rand::thread_rng();
        let bounds = Self::get_spawn_bounds(config).unwrap_or((
            Vec2::new(-200.0, -200.0),
            Vec2::new(200.0, 200.0),
        ));

        for _ in 0..config.particles.initial_particle_count {
            let position = Vec2::new(
                rng.gen_range(bounds.0.x..bounds.1.x),
                rng.gen_range(bounds.0.y..bounds.1.y),
            );

            let velocity = Vec2::new(
                rng.gen_range(-20.0..20.0),
                rng.gen_range(-20.0..20.0),
            );

            // Randomly assign positive or negative charge
            let charge = if rng.gen::<bool>() { 1.0 } else { -1.0 };
            let species_id = if charge > 0.0 { 0 } else { 1 };
            let color = if charge > 0.0 { [1.0, 0.3, 0.3, 1.0] } else { [0.3, 0.3, 1.0, 1.0] };

            let particle = Particle::new(position)
                .with_velocity(velocity)
                .with_mass(1.0)
                .with_charge(charge)
                .with_size(3.0)
                .with_species(species_id)
                .with_color(color)
                .with_lifespan(f32::INFINITY);

            system.add_particle(particle);
        }
    }

    fn create_brownian_system(system: &mut ParticleSystem, config: &SimulationConfig) {
        let mut rng = rand::thread_rng();
        let bounds = Self::get_spawn_bounds(config).unwrap_or((
            Vec2::new(-400.0, -300.0),
            Vec2::new(400.0, 300.0),
        ));

        for _ in 0..config.particles.initial_particle_count {
            let position = Vec2::new(
                rng.gen_range(bounds.0.x..bounds.1.x),
                rng.gen_range(bounds.0.y..bounds.1.y),
            );

            let particle = Particle::new(position)
                .with_velocity(Vec2::ZERO)
                .with_mass(1.0)
                .with_size(rng.gen_range(1.0..3.0))
                .with_species(0)
                .with_color([
                    rng.gen_range(0.5..1.0),
                    rng.gen_range(0.5..1.0),
                    rng.gen_range(0.5..1.0),
                    0.8,
                ])
                .with_lifespan(f32::INFINITY);

            system.add_particle(particle);
        }
    }

    fn create_reaction_diffusion_system(system: &mut ParticleSystem, config: &SimulationConfig) {
        let mut rng = rand::thread_rng();
        let bounds = Self::get_spawn_bounds(config).unwrap_or((
            Vec2::new(-200.0, -200.0),
            Vec2::new(200.0, 200.0),
        ));

        // Create activator particles (species 0) in small clusters
        let num_clusters = 5;
        for _ in 0..num_clusters {
            let cluster_center = Vec2::new(
                rng.gen_range(bounds.0.x..bounds.1.x),
                rng.gen_range(bounds.0.y..bounds.1.y),
            );

            let particles_per_cluster = config.particles.initial_particle_count * 6 / (10 * num_clusters);
            for _ in 0..particles_per_cluster {
                let position = cluster_center + Vec2::new(
                    rng.gen_range(-20.0..20.0),
                    rng.gen_range(-20.0..20.0),
                );

                let particle = Particle::new(position)
                    .with_velocity(Vec2::ZERO)
                    .with_mass(1.0)
                    .with_size(2.5)
                    .with_species(0) // Activator
                    .with_color([1.0, 0.3, 0.3, 1.0]) // Red
                    .with_lifespan(f32::INFINITY);

                system.add_particle(particle);
            }
        }

        // Fill the rest of the space with inhibitor particles (species 1)
        let remaining_particles = config.particles.initial_particle_count - system.particle_count();
        for _ in 0..remaining_particles {
            let position = Vec2::new(
                rng.gen_range(bounds.0.x..bounds.1.x),
                rng.gen_range(bounds.0.y..bounds.1.y),
            );

            let particle = Particle::new(position)
                .with_velocity(Vec2::ZERO)
                .with_mass(1.0)
                .with_size(2.0)
                .with_species(1) // Inhibitor
                .with_color([0.3, 0.3, 1.0, 1.0]) // Blue
                .with_lifespan(f32::INFINITY);

            system.add_particle(particle);
        }
    }

    pub fn create_interaction_matrix(preset: &Preset) -> InteractionMatrix {
        let mut matrix = InteractionMatrix::new();

        match preset {
            Preset::ParticleLife => {
                // Red-Red: mild repulsion
                matrix.add_interaction(0, 0, ForceType::Repulsion { strength: 20.0, max_distance: 30.0 });
                
                // Red-Blue: attraction
                matrix.add_interaction(0, 1, ForceType::Attraction { strength: 15.0, max_distance: 80.0 });
                
                // Red-Green: strong repulsion
                matrix.add_interaction(0, 2, ForceType::Repulsion { strength: 50.0, max_distance: 60.0 });
                
                // Blue-Blue: attraction
                matrix.add_interaction(1, 1, ForceType::Attraction { strength: 10.0, max_distance: 50.0 });
                
                // Blue-Green: mild attraction
                matrix.add_interaction(1, 2, ForceType::Attraction { strength: 8.0, max_distance: 70.0 });
                
                // Green-Green: repulsion
                matrix.add_interaction(2, 2, ForceType::Repulsion { strength: 30.0, max_distance: 40.0 });
            },
            Preset::Electromagnetic => {
                // Positive-Positive: repulsion
                matrix.add_interaction(0, 0, ForceType::ElectroMagnetic { strength: 1000.0, min_distance: 5.0 });
                
                // Negative-Negative: repulsion
                matrix.add_interaction(1, 1, ForceType::ElectroMagnetic { strength: 1000.0, min_distance: 5.0 });
                
                // Positive-Negative: attraction
                matrix.add_interaction(0, 1, ForceType::ElectroMagnetic { strength: -1000.0, min_distance: 5.0 });
            },
            Preset::ReactionDiffusion => {
                // Activator-Activator: self-reinforcement
                matrix.add_interaction(0, 0, ForceType::Attraction { strength: 25.0, max_distance: 40.0 });
                
                // Activator-Inhibitor: inhibition
                matrix.add_interaction(0, 1, ForceType::Repulsion { strength: 40.0, max_distance: 80.0 });
                
                // Inhibitor-Inhibitor: mild repulsion
                matrix.add_interaction(1, 1, ForceType::Repulsion { strength: 15.0, max_distance: 30.0 });
            },
            _ => {
                // Default forces for other presets
                matrix.default_forces = vec![
                    ForceType::Damping { coefficient: 0.01 },
                ];
            }
        }

        matrix
    }

    pub fn get_species_color(species_id: u32) -> [f32; 4] {
        match species_id {
            0 => [1.0, 0.3, 0.3, 1.0], // Red
            1 => [0.3, 0.3, 1.0, 1.0], // Blue
            2 => [0.3, 1.0, 0.3, 1.0], // Green
            3 => [1.0, 1.0, 0.3, 1.0], // Yellow
            4 => [1.0, 0.3, 1.0, 1.0], // Magenta
            5 => [0.3, 1.0, 1.0, 1.0], // Cyan
            6 => [1.0, 0.6, 0.2, 1.0], // Orange
            7 => [0.6, 0.2, 1.0, 1.0], // Purple
            _ => [0.8, 0.8, 0.8, 1.0], // Light gray for unknown species
        }
    }

    pub fn create_test_scenario(scenario_name: &str) -> (ParticleSystem, InteractionMatrix) {
        let mut system = ParticleSystem::new(100);
        let matrix = InteractionMatrix::new();

        match scenario_name {
            "chase" => {
                // Create a predator-prey chase scenario
                let prey = Particle::new(Vec2::new(-50.0, 0.0))
                    .with_velocity(Vec2::new(30.0, 0.0))
                    .with_species(0)
                    .with_color([0.3, 1.0, 0.3, 1.0])
                    .with_size(3.0);

                let predator = Particle::new(Vec2::new(-100.0, 0.0))
                    .with_velocity(Vec2::new(40.0, 0.0))
                    .with_species(1)
                    .with_color([1.0, 0.3, 0.3, 1.0])
                    .with_size(4.0);

                system.add_particle(prey);
                system.add_particle(predator);
            },
            "orbit" => {
                // Create a simple orbital system
                let center = Particle::new(Vec2::ZERO)
                    .with_mass(50.0)
                    .with_species(0)
                    .with_color([1.0, 1.0, 0.3, 1.0])
                    .with_size(8.0);

                let orbiter = Particle::new(Vec2::new(80.0, 0.0))
                    .with_velocity(Vec2::new(0.0, 25.0))
                    .with_mass(1.0)
                    .with_species(1)
                    .with_color([0.3, 0.8, 1.0, 1.0])
                    .with_size(3.0);

                system.add_particle(center);
                system.add_particle(orbiter);
            },
            "collision" => {
                // Create particles on collision course
                let p1 = Particle::new(Vec2::new(-100.0, 0.0))
                    .with_velocity(Vec2::new(50.0, 0.0))
                    .with_mass(2.0)
                    .with_species(0)
                    .with_color([1.0, 0.3, 0.3, 1.0])
                    .with_size(5.0);

                let p2 = Particle::new(Vec2::new(100.0, 0.0))
                    .with_velocity(Vec2::new(-50.0, 0.0))
                    .with_mass(2.0)
                    .with_species(1)
                    .with_color([0.3, 0.3, 1.0, 1.0])
                    .with_size(5.0);

                system.add_particle(p1);
                system.add_particle(p2);
            },
            _ => {
                // Default: random particles
                let mut rng = rand::thread_rng();
                for _ in 0..20 {
                    let position = Vec2::new(
                        rng.gen_range(-100.0..100.0),
                        rng.gen_range(-100.0..100.0),
                    );
                    
                    let particle = Particle::new(position)
                        .with_velocity(Vec2::new(
                            rng.gen_range(-20.0..20.0),
                            rng.gen_range(-20.0..20.0),
                        ))
                        .with_species(rng.gen_range(0..3))
                        .with_color(Self::get_species_color(rng.gen_range(0..3)))
                        .with_size(rng.gen_range(2.0..4.0));

                    system.add_particle(particle);
                }
            }
        }

        (system, matrix)
    }

    pub fn save_custom_preset(name: &str, config: &SimulationConfig, system: &ParticleSystem) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("presets/{}.json", name);
        std::fs::create_dir_all("presets")?;
        
        let preset_data = CustomPresetData {
            name: name.to_string(),
            description: format!("Custom preset with {} particles", system.particle_count()),
            config: config.clone(),
            initial_particles: system.particles.clone(),
        };
        
        let json = serde_json::to_string_pretty(&preset_data)?;
        std::fs::write(filename, json)?;
        
        Ok(())
    }

    pub fn load_custom_preset(name: &str) -> Result<CustomPresetData, Box<dyn std::error::Error>> {
        let filename = format!("presets/{}.json", name);
        let content = std::fs::read_to_string(filename)?;
        let preset_data = serde_json::from_str(&content)?;
        Ok(preset_data)
    }

    pub fn list_custom_presets() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut presets = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir("presets") {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".json") {
                            presets.push(name.trim_end_matches(".json").to_string());
                        }
                    }
                }
            }
        }
        
        Ok(presets)
    }
}

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPresetData {
    pub name: String,
    pub description: String,
    pub config: SimulationConfig,
    pub initial_particles: Vec<Particle>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_life_creation() {
        let config = SimulationConfig::default();
        let system = PresetManager::create_particle_system_from_preset(&Preset::ParticleLife, &config);
        assert!(system.particle_count() > 0);
    }

    #[test]
    fn test_interaction_matrix_creation() {
        let matrix = PresetManager::create_interaction_matrix(&Preset::ParticleLife);
        let forces = matrix.get_forces(0, 1);
        assert!(!forces.is_empty());
    }

    #[test]
    fn test_species_colors() {
        let red = PresetManager::get_species_color(0);
        assert_eq!(red, [1.0, 0.3, 0.3, 1.0]);
        
        let blue = PresetManager::get_species_color(1);
        assert_eq!(blue, [0.3, 0.3, 1.0, 1.0]);
    }

    #[test]
    fn test_test_scenarios() {
        let (system, _) = PresetManager::create_test_scenario("chase");
        assert_eq!(system.particle_count(), 2);
        
        let (system, _) = PresetManager::create_test_scenario("orbit");
        assert_eq!(system.particle_count(), 2);
    }
}