use nannou::prelude::*;
use nannou::wgpu;
use crate::particle::{Particle, ParticleSystem};
use crate::config::{RenderConfig, ParticleRenderMode};
use std::collections::VecDeque;

pub struct ParticleRenderer {
    config: RenderConfig,
    trail_history: Vec<VecDeque<Vec2>>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    uniform_buffer: Option<wgpu::Buffer>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    pub camera: Camera,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ParticleVertex {
    position: [f32; 2],
    color: [f32; 4],
    size: f32,
    _padding: [f32; 3],
}

unsafe impl bytemuck::Pod for ParticleVertex {}
unsafe impl bytemuck::Zeroable for ParticleVertex {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniforms {
    view_matrix: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
    time: f32,
    resolution: [f32; 2],
    _padding: f32,
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub target: Option<Vec2>,
    pub smoothing: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            target: None,
            smoothing: 0.1,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, target: Vec2) {
        self.target = Some(target);
    }

    pub fn clear_target(&mut self) {
        self.target = None;
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(target) = self.target {
            let diff = target - self.position;
            self.position += diff * self.smoothing * dt * 60.0; // 60 FPS normalization
        }
    }

    pub fn world_to_screen(&self, world_pos: Vec2, screen_size: Vec2) -> Vec2 {
        let centered = world_pos - self.position;
        let zoomed = centered * self.zoom;
        let rotated = Vec2::new(
            zoomed.x * self.rotation.cos() - zoomed.y * self.rotation.sin(),
            zoomed.x * self.rotation.sin() + zoomed.y * self.rotation.cos(),
        );
        rotated + screen_size * 0.5
    }

    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2) -> Vec2 {
        let centered = screen_pos - screen_size * 0.5;
        let rotated = Vec2::new(
            centered.x * self.rotation.cos() + centered.y * self.rotation.sin(),
            -centered.x * self.rotation.sin() + centered.y * self.rotation.cos(),
        );
        let unzoomed = rotated / self.zoom;
        unzoomed + self.position
    }

    pub fn get_view_bounds(&self, screen_size: Vec2) -> (Vec2, Vec2) {
        let half_size = screen_size * 0.5 / self.zoom;
        (
            self.position - half_size,
            self.position + half_size,
        )
    }
}

impl ParticleRenderer {
    pub fn new(config: RenderConfig) -> Self {
        Self {
            config,
            trail_history: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
            uniform_buffer: None,
            render_pipeline: None,
            camera: Camera::new(),
        }
    }

    pub fn update_config(&mut self, config: RenderConfig) {
        self.config = config;
        // Update camera settings
        self.camera.zoom = self.config.camera_zoom;
        self.camera.position = self.config.camera_position;
    }

    pub fn update(&mut self, system: &ParticleSystem, dt: f32) {
        self.camera.update(dt);
        self.update_trails(system);
    }

    fn update_trails(&mut self, system: &ParticleSystem) {
        if !self.config.enable_trails {
            self.trail_history.clear();
            return;
        }

        // Ensure trail history matches particle count
        while self.trail_history.len() < system.particles.len() {
            self.trail_history.push(VecDeque::new());
        }
        
        while self.trail_history.len() > system.particles.len() {
            self.trail_history.pop();
        }

        // Update trail positions
        for (i, particle) in system.particles.iter().enumerate() {
            if let Some(trail) = self.trail_history.get_mut(i) {
                trail.push_front(particle.position);
                
                // Limit trail length
                while trail.len() > self.config.trail_length {
                    trail.pop_back();
                }
            }
        }
    }

    pub fn render(&self, draw: &Draw, system: &ParticleSystem) {
        // Clear background
        draw.background().color(rgba(
            self.config.background_color[0],
            self.config.background_color[1],
            self.config.background_color[2],
            self.config.background_color[3],
        ));

        // Draw grid if enabled
        if self.config.enable_grid {
            self.draw_grid(draw);
        }

        // Draw trails first (so particles appear on top)
        if self.config.enable_trails {
            self.draw_trails(draw, system);
        }

        // Draw particles
        self.draw_particles(draw, system);

        // Draw velocity vectors if enabled
        if self.config.show_velocity_vectors {
            self.draw_velocity_vectors(draw, system);
        }

        // Draw force vectors if enabled
        if self.config.show_force_vectors {
            self.draw_force_vectors(draw, system);
        }

        // Draw particle IDs if enabled
        if self.config.show_particle_ids {
            self.draw_particle_ids(draw, system);
        }
    }

    fn draw_grid(&self, draw: &Draw) {
        let bounds = self.camera.get_view_bounds(Vec2::new(
            self.config.window_width as f32,
            self.config.window_height as f32,
        ));
        
        let grid_color = rgba(
            self.config.grid_color[0],
            self.config.grid_color[1],
            self.config.grid_color[2],
            self.config.grid_color[3],
        );

        // Vertical lines
        let start_x = (bounds.0.x / self.config.grid_spacing).floor() * self.config.grid_spacing;
        let mut x = start_x;
        while x <= bounds.1.x {
            let screen_x = self.camera.world_to_screen(
                Vec2::new(x, 0.0),
                Vec2::new(self.config.window_width as f32, self.config.window_height as f32)
            ).x;
            
            draw.line()
                .start(pt2(screen_x, 0.0))
                .end(pt2(screen_x, self.config.window_height as f32))
                .color(grid_color)
                .stroke_weight(1.0);
            
            x += self.config.grid_spacing;
        }

        // Horizontal lines
        let start_y = (bounds.0.y / self.config.grid_spacing).floor() * self.config.grid_spacing;
        let mut y = start_y;
        while y <= bounds.1.y {
            let screen_y = self.camera.world_to_screen(
                Vec2::new(0.0, y),
                Vec2::new(self.config.window_width as f32, self.config.window_height as f32)
            ).y;
            
            draw.line()
                .start(pt2(0.0, screen_y))
                .end(pt2(self.config.window_width as f32, screen_y))
                .color(grid_color)
                .stroke_weight(1.0);
            
            y += self.config.grid_spacing;
        }
    }

    fn draw_particles(&self, draw: &Draw, system: &ParticleSystem) {
        let screen_size = Vec2::new(
            self.config.window_width as f32,
            self.config.window_height as f32,
        );

        for particle in &system.particles {
            let screen_pos = self.camera.world_to_screen(particle.position, screen_size);
            
            // Skip particles outside screen bounds for performance
            if screen_pos.x < -50.0 || screen_pos.x > screen_size.x + 50.0 ||
               screen_pos.y < -50.0 || screen_pos.y > screen_size.y + 50.0 {
                continue;
            }

            let color = self.get_particle_color(particle, system);
            let size = particle.size * self.camera.zoom * self.config.point_size;

            match self.config.particle_render_mode {
                ParticleRenderMode::Points => {
                    draw.ellipse()
                        .x_y(screen_pos.x, screen_pos.y)
                        .radius(size * 0.5)
                        .color(color);
                },
                ParticleRenderMode::Circles => {
                    draw.ellipse()
                        .x_y(screen_pos.x, screen_pos.y)
                        .radius(size)
                        .color(color)
                        .stroke(rgba(color.red, color.green, color.blue, color.alpha * 0.5))
                        .stroke_weight(1.0);
                },
                ParticleRenderMode::Sprites => {
                    // For now, render as circles with glow effect
                    draw.ellipse()
                        .x_y(screen_pos.x, screen_pos.y)
                        .radius(size * 1.5)
                        .color(rgba(color.red, color.green, color.blue, color.alpha * 0.3));
                    
                    draw.ellipse()
                        .x_y(screen_pos.x, screen_pos.y)
                        .radius(size)
                        .color(color);
                },
                ParticleRenderMode::Metaballs => {
                    // Simplified metaball effect - larger, more transparent circles
                    draw.ellipse()
                        .x_y(screen_pos.x, screen_pos.y)
                        .radius(size * 2.0)
                        .color(rgba(color.red, color.green, color.blue, color.alpha * 0.2));
                },
                ParticleRenderMode::Lines => {
                    let vel_end = screen_pos + particle.velocity * 0.1 * self.camera.zoom;
                    draw.line()
                        .start(pt2(screen_pos.x, screen_pos.y))
                        .end(pt2(vel_end.x, vel_end.y))
                        .color(color)
                        .stroke_weight(self.config.line_width);
                },
                ParticleRenderMode::Trails => {
                    // This mode is handled by draw_trails
                    draw.ellipse()
                        .x_y(screen_pos.x, screen_pos.y)
                        .radius(size * 0.5)
                        .color(color);
                },
            }
        }
    }

    fn draw_trails(&self, draw: &Draw, system: &ParticleSystem) {
        let screen_size = Vec2::new(
            self.config.window_width as f32,
            self.config.window_height as f32,
        );

        for (i, particle) in system.particles.iter().enumerate() {
            if let Some(trail) = self.trail_history.get(i) {
                if trail.len() < 2 {
                    continue;
                }

                let base_color = self.get_particle_color(particle, system);
                
                for (j, &pos) in trail.iter().enumerate() {
                    let screen_pos = self.camera.world_to_screen(pos, screen_size);
                    
                    // Skip if off-screen
                    if screen_pos.x < -50.0 || screen_pos.x > screen_size.x + 50.0 ||
                       screen_pos.y < -50.0 || screen_pos.y > screen_size.y + 50.0 {
                        continue;
                    }

                    let age_factor = j as f32 / trail.len() as f32;
                    let alpha = base_color.alpha * (1.0 - age_factor) * self.config.trail_fade;
                    let size = particle.size * self.camera.zoom * (1.0 - age_factor * 0.5);
                    
                    let trail_color = rgba(
                        base_color.red,
                        base_color.green,
                        base_color.blue,
                        alpha,
                    );

                    draw.ellipse()
                        .x_y(screen_pos.x, screen_pos.y)
                        .radius(size)
                        .color(trail_color);
                }
            }
        }
    }

    fn draw_velocity_vectors(&self, draw: &Draw, system: &ParticleSystem) {
        let screen_size = Vec2::new(
            self.config.window_width as f32,
            self.config.window_height as f32,
        );

        for particle in &system.particles {
            let screen_pos = self.camera.world_to_screen(particle.position, screen_size);
            let velocity_scaled = particle.velocity * 10.0 * self.camera.zoom;
            let end_pos = screen_pos + velocity_scaled;

            draw.line()
                .start(pt2(screen_pos.x, screen_pos.y))
                .end(pt2(end_pos.x, end_pos.y))
                .color(rgba(1.0, 1.0, 0.0, 0.7))
                .stroke_weight(1.0);

            // Draw arrow head
            let arrow_length = 5.0;
            let angle = velocity_scaled.y.atan2(velocity_scaled.x);
            let arrow1 = end_pos + Vec2::new(
                -arrow_length * (angle - 0.5).cos(),
                -arrow_length * (angle - 0.5).sin(),
            );
            let arrow2 = end_pos + Vec2::new(
                -arrow_length * (angle + 0.5).cos(),
                -arrow_length * (angle + 0.5).sin(),
            );

            draw.line()
                .start(pt2(end_pos.x, end_pos.y))
                .end(pt2(arrow1.x, arrow1.y))
                .color(rgba(1.0, 1.0, 0.0, 0.7))
                .stroke_weight(1.0);

            draw.line()
                .start(pt2(end_pos.x, end_pos.y))
                .end(pt2(arrow2.x, arrow2.y))
                .color(rgba(1.0, 1.0, 0.0, 0.7))
                .stroke_weight(1.0);
        }
    }

    fn draw_force_vectors(&self, draw: &Draw, system: &ParticleSystem) {
        let screen_size = Vec2::new(
            self.config.window_width as f32,
            self.config.window_height as f32,
        );

        for particle in &system.particles {
            let screen_pos = self.camera.world_to_screen(particle.position, screen_size);
            let force_scaled = particle.acceleration * particle.mass * 50.0 * self.camera.zoom;
            let end_pos = screen_pos + force_scaled;

            draw.line()
                .start(pt2(screen_pos.x, screen_pos.y))
                .end(pt2(end_pos.x, end_pos.y))
                .color(rgba(1.0, 0.0, 0.0, 0.7))
                .stroke_weight(2.0);
        }
    }

    fn draw_particle_ids(&self, _draw: &Draw, _system: &ParticleSystem) {
        // Text rendering would require additional setup with nannou_egui or similar
        // For now, this is a placeholder
    }

    fn get_particle_color(&self, particle: &Particle, system: &ParticleSystem) -> Rgba {
        let mut color = rgba(
            particle.color[0],
            particle.color[1],
            particle.color[2],
            particle.color[3],
        );

        if self.config.color_by_velocity {
            let max_velocity = system.particles.iter()
                .map(|p| p.velocity.length())
                .fold(0.0, f32::max);
            
            if max_velocity > 0.0 {
                let velocity_ratio = particle.velocity.length() / max_velocity;
                color = rgba(velocity_ratio, 1.0 - velocity_ratio, 0.5, color.alpha);
            }
        }

        if self.config.color_by_energy {
            let max_energy = system.particles.iter()
                .map(|p| p.kinetic_energy())
                .fold(0.0, f32::max);
            
            if max_energy > 0.0 {
                let energy_ratio = particle.kinetic_energy() / max_energy;
                color = rgba(energy_ratio, 0.5, 1.0 - energy_ratio, color.alpha);
            }
        }

        // Apply species-specific coloring
        match particle.species_id {
            0 => rgba(1.0, 0.3, 0.3, color.alpha), // Red
            1 => rgba(0.3, 0.3, 1.0, color.alpha), // Blue
            2 => rgba(0.3, 1.0, 0.3, color.alpha), // Green
            3 => rgba(1.0, 1.0, 0.3, color.alpha), // Yellow
            4 => rgba(1.0, 0.3, 1.0, color.alpha), // Magenta
            5 => rgba(0.3, 1.0, 1.0, color.alpha), // Cyan
            _ => color, // Use original color
        }
    }

    pub fn handle_mouse_input(&mut self, mouse_pos: Vec2, screen_size: Vec2) {
        let world_pos = self.camera.screen_to_world(mouse_pos, screen_size);
        // Could be used for interaction, for now just update camera target
        // self.camera.set_target(world_pos);
    }

    pub fn handle_zoom(&mut self, zoom_delta: f32) {
        self.camera.zoom *= 1.0 + zoom_delta * 0.1;
        self.camera.zoom = self.camera.zoom.clamp(0.1, 10.0);
    }

    pub fn handle_pan(&mut self, delta: Vec2) {
        self.camera.position -= delta / self.camera.zoom;
    }

    pub fn reset_camera(&mut self) {
        self.camera.position = Vec2::ZERO;
        self.camera.zoom = 1.0;
        self.camera.rotation = 0.0;
        self.camera.clear_target();
    }

    pub fn focus_on_particles(&mut self, system: &ParticleSystem) {
        if !system.particles.is_empty() {
            let center = system.center_of_mass();
            self.camera.set_target(center);
        }
    }
}

// Helper struct for post-processing effects
pub struct PostProcessor {
    pub enable_bloom: bool,
    pub bloom_intensity: f32,
    pub hdr_exposure: f32,
}

impl PostProcessor {
    pub fn new() -> Self {
        Self {
            enable_bloom: false,
            bloom_intensity: 1.0,
            hdr_exposure: 1.0,
        }
    }

    pub fn process(&self, _frame: &wgpu::TextureView) {
        // Post-processing would be implemented here
        // This would require additional WGPU setup for framebuffers and shaders
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_world_to_screen() {
        let camera = Camera::new();
        let world_pos = Vec2::new(10.0, 20.0);
        let screen_size = Vec2::new(800.0, 600.0);
        
        let screen_pos = camera.world_to_screen(world_pos, screen_size);
        assert_eq!(screen_pos, Vec2::new(410.0, 320.0));
    }

    #[test]
    fn test_camera_screen_to_world() {
        let camera = Camera::new();
        let screen_pos = Vec2::new(410.0, 320.0);
        let screen_size = Vec2::new(800.0, 600.0);
        
        let world_pos = camera.screen_to_world(screen_pos, screen_size);
        assert_eq!(world_pos, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new();
        camera.zoom = 2.0;
        
        let world_pos = Vec2::new(10.0, 10.0);
        let screen_size = Vec2::new(800.0, 600.0);
        
        let screen_pos = camera.world_to_screen(world_pos, screen_size);
        assert_eq!(screen_pos, Vec2::new(420.0, 320.0));
    }
}