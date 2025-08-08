use nannou::prelude::*;
use inochi::{
    particle::{Particle, ParticleSystem},
    forces::{ForceCalculator, ForceType},
    config::SimulationConfig,
    renderer::ParticleRenderer,
};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    particle_system: ParticleSystem,
    force_calculator: ForceCalculator,
    renderer: ParticleRenderer,
    time: f32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 600)
        .title("Basic Particle Simulation Example")
        .view(view)
        .build()
        .unwrap();

    // Create a simple particle system
    let mut particle_system = ParticleSystem::new(200);
    
    // Add some initial particles
    for i in 0..50 {
        let angle = (i as f32) * 0.1;
        let radius = 100.0;
        
        let position = vec2(
            angle.cos() * radius,
            angle.sin() * radius,
        );
        
        let velocity = vec2(
            -angle.sin() * 20.0,
            angle.cos() * 20.0,
        );
        
        let particle = Particle::new(position)
            .with_velocity(velocity)
            .with_mass(1.0 + (i as f32 * 0.1))
            .with_size(2.0 + (i as f32 * 0.05))
            .with_species(i % 3)
            .with_color(match i % 3 {
                0 => [1.0, 0.3, 0.3, 1.0], // Red
                1 => [0.3, 0.3, 1.0, 1.0], // Blue
                _ => [0.3, 1.0, 0.3, 1.0], // Green
            });
        
        particle_system.add_particle(particle);
    }
    
    // Set up forces
    let mut force_calculator = ForceCalculator::new();
    
    // Add some interesting forces
    force_calculator.add_global_force(ForceType::Damping { coefficient: 0.01 });
    force_calculator.add_global_force(ForceType::Brownian { intensity: 5.0 });
    
    // Add species interactions
    force_calculator.interaction_matrix.add_interaction(
        0, 1, 
        ForceType::Attraction { strength: 30.0, max_distance: 80.0 }
    );
    
    force_calculator.interaction_matrix.add_interaction(
        0, 0, 
        ForceType::Repulsion { strength: 20.0, max_distance: 50.0 }
    );
    
    force_calculator.interaction_matrix.add_interaction(
        1, 2, 
        ForceType::Attraction { strength: 15.0, max_distance: 60.0 }
    );

    // Create renderer
    let config = SimulationConfig::default();
    let renderer = ParticleRenderer::new(config.rendering);

    Model {
        particle_system,
        force_calculator,
        renderer,
        time: 0.0,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let dt = update.since_last.as_secs_f32();
    model.time += dt;
    
    // Apply forces to particles
    model.force_calculator.apply_forces(&mut model.particle_system);
    
    // Update particle system
    model.particle_system.update(dt);
    
    // Update renderer
    model.renderer.update(&model.particle_system, dt);
    
    // Handle input
    for key in app.keys.down.iter() {
        match key {
            Key::Space => {
                // Add a new particle at mouse position
                let mouse_pos = vec2(app.mouse.x, app.mouse.y);
                let velocity = vec2(
                    (random::<f32>() - 0.5) * 40.0,
                    (random::<f32>() - 0.5) * 40.0,
                );
                
                let particle = Particle::new(mouse_pos)
                    .with_velocity(velocity)
                    .with_species((model.time as u32) % 3)
                    .with_size(random_range(1.0, 4.0))
                    .with_color([random(), random(), random(), 1.0]);
                
                model.particle_system.add_particle(particle);
            },
            Key::R => {
                // Reset simulation
                model.particle_system.clear();
            },
            _ => {}
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    
    // Render the particle system
    model.renderer.render(&draw, &model.particle_system);
    
    // Add some UI text
    draw.text(&format!("Particles: {}", model.particle_system.particle_count()))
        .xy(vec2(-350.0, 250.0))
        .font_size(18)
        .color(WHITE);
    
    draw.text("Press SPACE to add particles, R to reset")
        .xy(vec2(-350.0, 220.0))
        .font_size(14)
        .color(GRAY);
    
    draw.text(&format!("Time: {:.1}s", model.time))
        .xy(vec2(-350.0, 190.0))
        .font_size(14)
        .color(GRAY);
    
    // Show energy and center of mass
    let total_energy = model.particle_system.total_energy();
    let center_of_mass = model.particle_system.center_of_mass();
    
    draw.text(&format!("Total Energy: {:.2}", total_energy))
        .xy(vec2(-350.0, 160.0))
        .font_size(12)
        .color(YELLOW);
    
    draw.text(&format!("Center of Mass: ({:.1}, {:.1})", center_of_mass.x, center_of_mass.y))
        .xy(vec2(-350.0, 140.0))
        .font_size(12)
        .color(YELLOW);
    
    // Draw center of mass indicator
    draw.ellipse()
        .xy(center_of_mass)
        .radius(5.0)
        .color(rgba(1.0, 1.0, 0.0, 0.5))
        .stroke(YELLOW)
        .stroke_weight(2.0);
    
    draw.to_frame(app, &frame).unwrap();
}