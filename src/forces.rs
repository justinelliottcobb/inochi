use glam::Vec2;
use crate::particle::{Particle, ParticleSystem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForceType {
    Gravity {
        strength: f32,
        min_distance: f32,
    },
    ElectroMagnetic {
        strength: f32,
        min_distance: f32,
    },
    LennardJones {
        epsilon: f32,
        sigma: f32,
    },
    Damping {
        coefficient: f32,
    },
    Brownian {
        intensity: f32,
    },
    Attraction {
        strength: f32,
        max_distance: f32,
    },
    Repulsion {
        strength: f32,
        max_distance: f32,
    },
    Vortex {
        center: Vec2,
        strength: f32,
        max_distance: f32,
    },
    Spring {
        rest_length: f32,
        stiffness: f32,
        damping: f32,
    },
    Flocking {
        separation_radius: f32,
        alignment_radius: f32,
        cohesion_radius: f32,
        separation_strength: f32,
        alignment_strength: f32,
        cohesion_strength: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionMatrix {
    pub interactions: HashMap<(u32, u32), Vec<ForceType>>,
    pub default_forces: Vec<ForceType>,
}

impl Default for InteractionMatrix {
    fn default() -> Self {
        Self {
            interactions: HashMap::new(),
            default_forces: vec![
                ForceType::Damping { coefficient: 0.01 },
                ForceType::Brownian { intensity: 0.1 },
            ],
        }
    }
}

impl InteractionMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_interaction(&mut self, species_a: u32, species_b: u32, force: ForceType) {
        let key = if species_a <= species_b {
            (species_a, species_b)
        } else {
            (species_b, species_a)
        };
        
        self.interactions.entry(key).or_insert_with(Vec::new).push(force);
    }

    pub fn get_forces(&self, species_a: u32, species_b: u32) -> &[ForceType] {
        let key = if species_a <= species_b {
            (species_a, species_b)
        } else {
            (species_b, species_a)
        };
        
        self.interactions
            .get(&key)
            .map(|v| v.as_slice())
            .unwrap_or(&self.default_forces)
    }
}

pub struct ForceCalculator {
    pub interaction_matrix: InteractionMatrix,
    pub global_forces: Vec<ForceType>,
    pub dt: f32,
}

impl Default for ForceCalculator {
    fn default() -> Self {
        Self {
            interaction_matrix: InteractionMatrix::default(),
            global_forces: Vec::new(),
            dt: 1.0 / 60.0,
        }
    }
}

impl ForceCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dt(mut self, dt: f32) -> Self {
        self.dt = dt;
        self
    }

    pub fn add_global_force(&mut self, force: ForceType) {
        self.global_forces.push(force);
    }

    pub fn apply_forces(&self, system: &mut ParticleSystem) {
        let particles_copy = system.particles.clone();
        
        for (i, particle) in system.particles.iter_mut().enumerate() {
            self.apply_global_forces(particle);
            
            for (j, other) in particles_copy.iter().enumerate() {
                if i != j {
                    self.apply_pair_forces(particle, other);
                }
            }
            
            self.apply_flocking_forces(particle, i, &particles_copy);
        }
    }

    fn apply_global_forces(&self, particle: &mut Particle) {
        for force in &self.global_forces {
            let force_vec = self.calculate_force(force, particle, None);
            particle.apply_force(force_vec);
        }
    }

    fn apply_pair_forces(&self, particle: &mut Particle, other: &Particle) {
        let forces = self.interaction_matrix.get_forces(particle.species_id, other.species_id);
        
        for force_type in forces {
            let force_vec = self.calculate_force(force_type, particle, Some(other));
            particle.apply_force(force_vec);
        }
    }

    fn apply_flocking_forces(&self, particle: &mut Particle, index: usize, all_particles: &[Particle]) {
        for force_type in &self.global_forces {
            if let ForceType::Flocking { .. } = force_type {
                let force_vec = self.calculate_flocking_force(particle, index, all_particles, force_type);
                particle.apply_force(force_vec);
            }
        }
    }

    fn calculate_force(&self, force_type: &ForceType, particle: &Particle, other: Option<&Particle>) -> Vec2 {
        match force_type {
            ForceType::Gravity { strength, min_distance } => {
                if let Some(other) = other {
                    self.calculate_gravitational_force(particle, other, *strength, *min_distance)
                } else {
                    Vec2::ZERO
                }
            },
            ForceType::ElectroMagnetic { strength, min_distance } => {
                if let Some(other) = other {
                    self.calculate_electromagnetic_force(particle, other, *strength, *min_distance)
                } else {
                    Vec2::ZERO
                }
            },
            ForceType::LennardJones { epsilon, sigma } => {
                if let Some(other) = other {
                    self.calculate_lennard_jones_force(particle, other, *epsilon, *sigma)
                } else {
                    Vec2::ZERO
                }
            },
            ForceType::Damping { coefficient } => {
                -particle.velocity * *coefficient
            },
            ForceType::Brownian { intensity } => {
                Vec2::new(
                    (rand::random::<f32>() - 0.5) * *intensity,
                    (rand::random::<f32>() - 0.5) * *intensity,
                )
            },
            ForceType::Attraction { strength, max_distance } => {
                if let Some(other) = other {
                    self.calculate_attraction_force(particle, other, *strength, *max_distance)
                } else {
                    Vec2::ZERO
                }
            },
            ForceType::Repulsion { strength, max_distance } => {
                if let Some(other) = other {
                    self.calculate_repulsion_force(particle, other, *strength, *max_distance)
                } else {
                    Vec2::ZERO
                }
            },
            ForceType::Vortex { center, strength, max_distance } => {
                self.calculate_vortex_force(particle, *center, *strength, *max_distance)
            },
            ForceType::Spring { rest_length, stiffness, damping } => {
                if let Some(other) = other {
                    self.calculate_spring_force(particle, other, *rest_length, *stiffness, *damping)
                } else {
                    Vec2::ZERO
                }
            },
            ForceType::Flocking { .. } => Vec2::ZERO, // Handled separately
        }
    }

    fn calculate_gravitational_force(&self, particle: &Particle, other: &Particle, strength: f32, min_distance: f32) -> Vec2 {
        let distance_vec = other.position - particle.position;
        let distance = distance_vec.length().max(min_distance);
        let direction = distance_vec.normalize_or_zero();
        
        let force_magnitude = strength * particle.mass * other.mass / (distance * distance);
        direction * force_magnitude
    }

    fn calculate_electromagnetic_force(&self, particle: &Particle, other: &Particle, strength: f32, min_distance: f32) -> Vec2 {
        let distance_vec = other.position - particle.position;
        let distance = distance_vec.length().max(min_distance);
        let direction = distance_vec.normalize_or_zero();
        
        let force_magnitude = strength * particle.charge * other.charge / (distance * distance);
        direction * force_magnitude
    }

    fn calculate_lennard_jones_force(&self, particle: &Particle, other: &Particle, epsilon: f32, sigma: f32) -> Vec2 {
        let distance_vec = other.position - particle.position;
        let distance = distance_vec.length();
        
        if distance == 0.0 {
            return Vec2::ZERO;
        }
        
        let direction = distance_vec.normalize();
        let r_over_sigma = distance / sigma;
        let r6 = r_over_sigma.powi(6);
        let r12 = r6 * r6;
        
        let force_magnitude = 24.0 * epsilon * (2.0 / r12 - 1.0 / r6) / distance;
        direction * force_magnitude
    }

    fn calculate_attraction_force(&self, particle: &Particle, other: &Particle, strength: f32, max_distance: f32) -> Vec2 {
        let distance_vec = other.position - particle.position;
        let distance = distance_vec.length();
        
        if distance > max_distance || distance == 0.0 {
            return Vec2::ZERO;
        }
        
        let direction = distance_vec.normalize();
        let force_magnitude = strength * (1.0 - distance / max_distance);
        direction * force_magnitude
    }

    fn calculate_repulsion_force(&self, particle: &Particle, other: &Particle, strength: f32, max_distance: f32) -> Vec2 {
        let distance_vec = other.position - particle.position;
        let distance = distance_vec.length();
        
        if distance > max_distance || distance == 0.0 {
            return Vec2::ZERO;
        }
        
        let direction = -distance_vec.normalize(); // Repulsion is opposite direction
        let force_magnitude = strength * (1.0 - distance / max_distance);
        direction * force_magnitude
    }

    fn calculate_vortex_force(&self, particle: &Particle, center: Vec2, strength: f32, max_distance: f32) -> Vec2 {
        let distance_vec = particle.position - center;
        let distance = distance_vec.length();
        
        if distance > max_distance || distance == 0.0 {
            return Vec2::ZERO;
        }
        
        let tangent = Vec2::new(-distance_vec.y, distance_vec.x).normalize();
        let force_magnitude = strength * (1.0 - distance / max_distance);
        tangent * force_magnitude
    }

    fn calculate_spring_force(&self, particle: &Particle, other: &Particle, rest_length: f32, stiffness: f32, damping: f32) -> Vec2 {
        let distance_vec = other.position - particle.position;
        let distance = distance_vec.length();
        
        if distance == 0.0 {
            return Vec2::ZERO;
        }
        
        let direction = distance_vec.normalize();
        let displacement = distance - rest_length;
        
        let spring_force = stiffness * displacement;
        let relative_velocity = other.velocity - particle.velocity;
        let damping_force = damping * relative_velocity.dot(direction);
        
        direction * (spring_force + damping_force)
    }

    fn calculate_flocking_force(&self, particle: &Particle, index: usize, all_particles: &[Particle], force_type: &ForceType) -> Vec2 {
        if let ForceType::Flocking {
            separation_radius,
            alignment_radius,
            cohesion_radius,
            separation_strength,
            alignment_strength,
            cohesion_strength,
        } = force_type {
            let mut separation = Vec2::ZERO;
            let mut alignment = Vec2::ZERO;
            let mut cohesion = Vec2::ZERO;
            let mut sep_count = 0;
            let mut align_count = 0;
            let mut coh_count = 0;

            for (i, other) in all_particles.iter().enumerate() {
                if i == index || other.species_id != particle.species_id {
                    continue;
                }

                let distance_vec = other.position - particle.position;
                let distance = distance_vec.length();

                if distance > 0.0 && distance < *separation_radius {
                    separation -= distance_vec.normalize() / distance;
                    sep_count += 1;
                }

                if distance > 0.0 && distance < *alignment_radius {
                    alignment += other.velocity;
                    align_count += 1;
                }

                if distance > 0.0 && distance < *cohesion_radius {
                    cohesion += other.position;
                    coh_count += 1;
                }
            }

            let mut total_force = Vec2::ZERO;

            if sep_count > 0 {
                separation = (separation / sep_count as f32).normalize_or_zero();
                total_force += separation * *separation_strength;
            }

            if align_count > 0 {
                alignment = (alignment / align_count as f32) - particle.velocity;
                alignment = alignment.normalize_or_zero();
                total_force += alignment * *alignment_strength;
            }

            if coh_count > 0 {
                cohesion = (cohesion / coh_count as f32) - particle.position;
                cohesion = cohesion.normalize_or_zero();
                total_force += cohesion * *cohesion_strength;
            }

            total_force
        } else {
            Vec2::ZERO
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub integration_method: IntegrationMethod,
    pub dt: f32,
    pub max_force: f32,
    pub max_velocity: f32,
    pub enable_collisions: bool,
    pub collision_restitution: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationMethod {
    Euler,
    Verlet,
    RungeKutta4,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            integration_method: IntegrationMethod::Verlet,
            dt: 1.0 / 60.0,
            max_force: 1000.0,
            max_velocity: 100.0,
            enable_collisions: false,
            collision_restitution: 0.8,
        }
    }
}

pub struct PhysicsEngine {
    pub config: PhysicsConfig,
    pub force_calculator: ForceCalculator,
    previous_positions: Vec<Vec2>,
}

impl PhysicsEngine {
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            force_calculator: ForceCalculator::new().with_dt(config.dt),
            config,
            previous_positions: Vec::new(),
        }
    }

    pub fn update(&mut self, system: &mut ParticleSystem) {
        self.force_calculator.dt = self.config.dt;
        self.force_calculator.apply_forces(system);

        match self.config.integration_method {
            IntegrationMethod::Euler => self.euler_integration(system),
            IntegrationMethod::Verlet => self.verlet_integration(system),
            IntegrationMethod::RungeKutta4 => self.rk4_integration(system),
        }

        if self.config.enable_collisions {
            self.handle_collisions(system);
        }
    }

    fn euler_integration(&self, system: &mut ParticleSystem) {
        for particle in &mut system.particles {
            let old_velocity = particle.velocity;
            particle.velocity += particle.acceleration * self.config.dt;
            particle.velocity = particle.velocity.clamp_length_max(self.config.max_velocity);
            
            particle.position += old_velocity * self.config.dt;
            particle.age += self.config.dt;
            particle.acceleration = Vec2::ZERO;
        }
    }

    fn verlet_integration(&mut self, system: &mut ParticleSystem) {
        if self.previous_positions.len() != system.particles.len() {
            self.previous_positions = system.particles.iter().map(|p| p.position).collect();
        }

        for (i, particle) in system.particles.iter_mut().enumerate() {
            let new_position = 2.0 * particle.position - self.previous_positions[i] + 
                              particle.acceleration * self.config.dt * self.config.dt;
            
            self.previous_positions[i] = particle.position;
            particle.velocity = (new_position - particle.position) / self.config.dt;
            particle.velocity = particle.velocity.clamp_length_max(self.config.max_velocity);
            particle.position = new_position;
            particle.age += self.config.dt;
            particle.acceleration = Vec2::ZERO;
        }
    }

    fn rk4_integration(&self, system: &mut ParticleSystem) {
        for particle in &mut system.particles {
            let dt = self.config.dt;
            let k1_v = particle.acceleration * dt;
            let k1_x = particle.velocity * dt;
            
            let k2_v = particle.acceleration * dt; // Simplified - should recalculate forces
            let k2_x = (particle.velocity + k1_v * 0.5) * dt;
            
            let k3_v = particle.acceleration * dt; // Simplified
            let k3_x = (particle.velocity + k2_v * 0.5) * dt;
            
            let k4_v = particle.acceleration * dt; // Simplified
            let k4_x = (particle.velocity + k3_v) * dt;
            
            particle.velocity += (k1_v + 2.0 * k2_v + 2.0 * k3_v + k4_v) / 6.0;
            particle.velocity = particle.velocity.clamp_length_max(self.config.max_velocity);
            particle.position += (k1_x + 2.0 * k2_x + 2.0 * k3_x + k4_x) / 6.0;
            particle.age += dt;
            particle.acceleration = Vec2::ZERO;
        }
    }

    fn handle_collisions(&self, system: &mut ParticleSystem) {
        let particles_copy = system.particles.clone();
        
        for (i, particle) in system.particles.iter_mut().enumerate() {
            for (j, other) in particles_copy.iter().enumerate() {
                if i >= j {
                    continue;
                }
                
                let distance = particle.distance_to(other);
                let min_distance = (particle.size + other.size) * 0.5;
                
                if distance < min_distance && distance > 0.0 {
                    let overlap = min_distance - distance;
                    let direction = (particle.position - other.position) / distance;
                    
                    particle.position += direction * overlap * 0.5;
                    
                    let relative_velocity = particle.velocity - other.velocity;
                    let velocity_along_normal = relative_velocity.dot(direction);
                    
                    if velocity_along_normal > 0.0 {
                        continue;
                    }
                    
                    let impulse_magnitude = -(1.0 + self.config.collision_restitution) * velocity_along_normal;
                    let impulse = direction * impulse_magnitude;
                    
                    particle.apply_impulse(impulse);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_matrix() {
        let mut matrix = InteractionMatrix::new();
        matrix.add_interaction(0, 1, ForceType::Attraction { strength: 1.0, max_distance: 10.0 });
        
        let forces = matrix.get_forces(0, 1);
        assert_eq!(forces.len(), 1);
    }

    #[test]
    fn test_gravitational_force() {
        let calculator = ForceCalculator::new();
        let p1 = Particle::new(Vec2::ZERO).with_mass(1.0);
        let p2 = Particle::new(Vec2::new(1.0, 0.0)).with_mass(1.0);
        
        let force = calculator.calculate_gravitational_force(&p1, &p2, 1.0, 0.01);
        assert!(force.x > 0.0);
        assert_eq!(force.y, 0.0);
    }

    #[test]
    fn test_physics_engine() {
        let config = PhysicsConfig::default();
        let mut engine = PhysicsEngine::new(config);
        let mut system = ParticleSystem::new(10);
        
        system.add_particle(Particle::new(Vec2::ZERO));
        engine.update(&mut system);
        
        assert_eq!(system.particle_count(), 1);
    }
}