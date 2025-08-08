use glam::Vec2;
use serde::{Deserialize, Serialize};
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub mass: f32,
    pub charge: f32,
    pub age: f32,
    pub lifespan: f32,
    pub color: [f32; 4],
    pub species_id: u32,
    pub energy: f32,
    pub size: f32,
    pub temperature: f32,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            mass: 1.0,
            charge: 0.0,
            age: 0.0,
            lifespan: f32::INFINITY,
            color: [1.0, 1.0, 1.0, 1.0],
            species_id: 0,
            energy: 1.0,
            size: 1.0,
            temperature: 1.0,
        }
    }
}

impl Particle {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn with_charge(mut self, charge: f32) -> Self {
        self.charge = charge;
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_species(mut self, species_id: u32) -> Self {
        self.species_id = species_id;
        self
    }

    pub fn with_lifespan(mut self, lifespan: f32) -> Self {
        self.lifespan = lifespan;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn is_alive(&self) -> bool {
        self.age < self.lifespan
    }

    pub fn life_ratio(&self) -> f32 {
        if self.lifespan.is_infinite() {
            1.0
        } else {
            1.0 - (self.age / self.lifespan).clamp(0.0, 1.0)
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.velocity += self.acceleration * dt;
        self.position += self.velocity * dt;
        self.age += dt;
        self.acceleration = Vec2::ZERO;
        
        self.energy = self.velocity.length_squared() * 0.5 * self.mass;
        
        let life_factor = self.life_ratio();
        self.color[3] = life_factor;
    }

    pub fn apply_force(&mut self, force: Vec2) {
        if self.mass > 0.0 {
            self.acceleration += force / self.mass;
        }
    }

    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if self.mass > 0.0 {
            self.velocity += impulse / self.mass;
        }
    }

    pub fn distance_to(&self, other: &Particle) -> f32 {
        self.position.distance(other.position)
    }

    pub fn distance_squared_to(&self, other: &Particle) -> f32 {
        self.position.distance_squared(other.position)
    }

    pub fn direction_to(&self, other: &Particle) -> Vec2 {
        let diff = other.position - self.position;
        let len = diff.length();
        if len > 0.0 {
            diff / len
        } else {
            Vec2::ZERO
        }
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.velocity.length_squared()
    }

    pub fn momentum(&self) -> Vec2 {
        self.velocity * self.mass
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub max_particles: usize,
    pub spawn_rate: f32,
    pub spawn_timer: f32,
    pub bounds: Option<(Vec2, Vec2)>,
    pub wrap_boundaries: bool,
    pub damping: f32,
}

impl ParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            max_particles,
            spawn_rate: 10.0,
            spawn_timer: 0.0,
            bounds: None,
            wrap_boundaries: false,
            damping: 0.99,
        }
    }

    pub fn add_particle(&mut self, particle: Particle) {
        if self.particles.len() < self.max_particles {
            self.particles.push(particle);
        }
    }

    pub fn spawn_particle_at(&mut self, position: Vec2) {
        if self.particles.len() < self.max_particles {
            self.add_particle(Particle::new(position));
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.spawn_timer += dt;
        
        if self.spawn_timer >= 1.0 / self.spawn_rate {
            if let Some((min_bounds, max_bounds)) = self.bounds {
                let spawn_pos = Vec2::new(
                    rand::random::<f32>() * (max_bounds.x - min_bounds.x) + min_bounds.x,
                    rand::random::<f32>() * (max_bounds.y - min_bounds.y) + min_bounds.y,
                );
                self.spawn_particle_at(spawn_pos);
                self.spawn_timer = 0.0;
            }
        }

        for particle in &mut self.particles {
            particle.velocity *= self.damping;
            particle.update(dt);
            
            if let Some((min_bounds, max_bounds)) = self.bounds {
                self.apply_boundary_conditions(particle, min_bounds, max_bounds);
            }
        }

        self.particles.retain(|p| p.is_alive());
    }

    fn apply_boundary_conditions(&self, particle: &mut Particle, min_bounds: Vec2, max_bounds: Vec2) {
        if self.wrap_boundaries {
            if particle.position.x < min_bounds.x {
                particle.position.x = max_bounds.x;
            } else if particle.position.x > max_bounds.x {
                particle.position.x = min_bounds.x;
            }
            
            if particle.position.y < min_bounds.y {
                particle.position.y = max_bounds.y;
            } else if particle.position.y > max_bounds.y {
                particle.position.y = min_bounds.y;
            }
        } else {
            if particle.position.x < min_bounds.x || particle.position.x > max_bounds.x {
                particle.velocity.x = -particle.velocity.x * 0.8;
                particle.position.x = particle.position.x.clamp(min_bounds.x, max_bounds.x);
            }
            
            if particle.position.y < min_bounds.y || particle.position.y > max_bounds.y {
                particle.velocity.y = -particle.velocity.y * 0.8;
                particle.position.y = particle.position.y.clamp(min_bounds.y, max_bounds.y);
            }
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn total_energy(&self) -> f32 {
        self.particles.iter().map(|p| p.kinetic_energy()).sum()
    }

    pub fn center_of_mass(&self) -> Vec2 {
        if self.particles.is_empty() {
            return Vec2::ZERO;
        }

        let total_mass: f32 = self.particles.iter().map(|p| p.mass).sum();
        if total_mass == 0.0 {
            return Vec2::ZERO;
        }

        let weighted_position: Vec2 = self.particles
            .iter()
            .map(|p| p.position * p.mass)
            .sum();
        
        weighted_position / total_mass
    }

    pub fn average_velocity(&self) -> Vec2 {
        if self.particles.is_empty() {
            return Vec2::ZERO;
        }

        let total_velocity: Vec2 = self.particles.iter().map(|p| p.velocity).sum();
        total_velocity / self.particles.len() as f32
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn set_bounds(&mut self, min: Vec2, max: Vec2) {
        self.bounds = Some((min, max));
    }

    pub fn remove_bounds(&mut self) {
        self.bounds = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let particle = Particle::new(Vec2::new(1.0, 2.0))
            .with_mass(2.0)
            .with_velocity(Vec2::new(3.0, 4.0));
        
        assert_eq!(particle.position, Vec2::new(1.0, 2.0));
        assert_eq!(particle.mass, 2.0);
        assert_eq!(particle.velocity, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn test_particle_update() {
        let mut particle = Particle::new(Vec2::ZERO)
            .with_velocity(Vec2::new(1.0, 0.0));
        
        particle.update(1.0);
        assert_eq!(particle.position, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn test_force_application() {
        let mut particle = Particle::new(Vec2::ZERO)
            .with_mass(2.0);
        
        particle.apply_force(Vec2::new(4.0, 0.0));
        assert_eq!(particle.acceleration, Vec2::new(2.0, 0.0));
    }

    #[test]
    fn test_particle_system() {
        let mut system = ParticleSystem::new(10);
        assert_eq!(system.particle_count(), 0);
        
        system.add_particle(Particle::new(Vec2::ZERO));
        assert_eq!(system.particle_count(), 1);
    }
}