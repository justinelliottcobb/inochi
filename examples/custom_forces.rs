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
    current_example: usize,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1000, 700)
        .title("Custom Forces Examples")
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let mut model = Model {
        particle_system: ParticleSystem::new(300),
        force_calculator: ForceCalculator::new(),
        renderer: ParticleRenderer::new(SimulationConfig::default().rendering),
        time: 0.0,
        current_example: 0,
    };
    
    // Start with the first example
    setup_example(&mut model, 0);
    
    model
}

fn setup_example(model: &mut Model, example_id: usize) {
    model.particle_system.clear();
    model.force_calculator = ForceCalculator::new();
    model.current_example = example_id;
    
    match example_id {
        0 => setup_lennard_jones_example(model),
        1 => setup_electromagnetic_example(model),
        2 => setup_vortex_example(model),
        3 => setup_spring_network_example(model),
        4 => setup_flocking_example(model),
        _ => setup_lennard_jones_example(model),
    }
}

fn setup_lennard_jones_example(model: &mut Model) {
    // Lennard-Jones potential demonstration
    model.force_calculator.interaction_matrix.add_interaction(
        0, 0,
        ForceType::LennardJones { epsilon: 4.0, sigma: 20.0 }
    );
    
    model.force_calculator.add_global_force(ForceType::Damping { coefficient: 0.02 });
    
    // Create a grid of particles
    for x in -3..=3 {
        for y in -3..=3 {
            let position = vec2(x as f32 * 25.0, y as f32 * 25.0);
            let velocity = vec2(
                (random::<f32>() - 0.5) * 20.0,
                (random::<f32>() - 0.5) * 20.0,
            );
            
            let particle = Particle::new(position)
                .with_velocity(velocity)
                .with_species(0)
                .with_size(3.0)
                .with_color([0.8, 0.8, 1.0, 1.0]);
            
            model.particle_system.add_particle(particle);
        }
    }
}

fn setup_electromagnetic_example(model: &mut Model) {
    // Electromagnetic forces with positive and negative charges
    model.force_calculator.interaction_matrix.add_interaction(
        0, 0,
        ForceType::ElectroMagnetic { strength: 500.0, min_distance: 5.0 }
    );
    
    model.force_calculator.interaction_matrix.add_interaction(
        1, 1,
        ForceType::ElectroMagnetic { strength: 500.0, min_distance: 5.0 }
    );
    
    model.force_calculator.interaction_matrix.add_interaction(
        0, 1,
        ForceType::ElectroMagnetic { strength: -800.0, min_distance: 5.0 }
    );
    
    model.force_calculator.add_global_force(ForceType::Damping { coefficient: 0.01 });
    
    // Create charged particles
    for i in 0..40 {
        let angle = (i as f32) * 0.15;
        let radius = 80.0 + (i as f32) * 2.0;
        
        let position = vec2(angle.cos() * radius, angle.sin() * radius);
        let species = i % 2;
        let charge = if species == 0 { 1.0 } else { -1.0 };
        let color = if species == 0 { [1.0, 0.3, 0.3, 1.0] } else { [0.3, 0.3, 1.0, 1.0] };
        
        let particle = Particle::new(position)
            .with_velocity(vec2(0.0, 0.0))
            .with_charge(charge)
            .with_species(species)
            .with_size(4.0)
            .with_color(color);
        
        model.particle_system.add_particle(particle);
    }
}

fn setup_vortex_example(model: &mut Model) {
    // Vortex forces creating spiral patterns
    model.force_calculator.add_global_force(
        ForceType::Vortex {
            center: vec2(0.0, 0.0),
            strength: 30.0,
            max_distance: 200.0,
        }
    );
    
    model.force_calculator.add_global_force(
        ForceType::Vortex {
            center: vec2(100.0, 100.0),
            strength: -20.0,
            max_distance: 150.0,
        }
    );
    
    model.force_calculator.add_global_force(
        ForceType::Vortex {
            center: vec2(-100.0, -100.0),
            strength: -20.0,
            max_distance: 150.0,
        }
    );
    
    model.force_calculator.add_global_force(ForceType::Damping { coefficient: 0.005 });
    
    // Scatter particles randomly
    for _ in 0..60 {
        let position = vec2(
            random_range(-150.0, 150.0),
            random_range(-150.0, 150.0),
        );
        
        let particle = Particle::new(position)
            .with_velocity(vec2(0.0, 0.0))
            .with_species(0)
            .with_size(2.5)
            .with_color([0.3, 1.0, 0.8, 1.0]);
        
        model.particle_system.add_particle(particle);
    }
}

fn setup_spring_network_example(model: &mut Model) {
    // Note: This is a simplified example. A full spring network would require
    // tracking specific particle pairs and their rest lengths.
    
    model.force_calculator.interaction_matrix.add_interaction(
        0, 0,
        ForceType::Spring {
            rest_length: 30.0,
            stiffness: 0.5,
            damping: 0.1,
        }
    );
    
    // Create a loose grid that will form a spring network
    for x in -2..=2 {
        for y in -2..=2 {
            let position = vec2(x as f32 * 40.0, y as f32 * 40.0);
            let noise_x = (random::<f32>() - 0.5) * 20.0;
            let noise_y = (random::<f32>() - 0.5) * 20.0;
            
            let particle = Particle::new(position + vec2(noise_x, noise_y))
                .with_velocity(vec2(0.0, 0.0))
                .with_species(0)
                .with_size(4.0)
                .with_mass(1.0)
                .with_color([1.0, 0.8, 0.3, 1.0]);
            
            model.particle_system.add_particle(particle);
        }
    }
}

fn setup_flocking_example(model: &mut Model) {
    // Flocking behavior
    model.force_calculator.add_global_force(
        ForceType::Flocking {
            separation_radius: 25.0,
            alignment_radius: 50.0,
            cohesion_radius: 75.0,
            separation_strength: 40.0,
            alignment_strength: 15.0,
            cohesion_strength: 10.0,
        }
    );
    
    model.force_calculator.add_global_force(ForceType::Damping { coefficient: 0.02 });
    
    // Create several flocks
    for flock in 0..3 {
        let center_x = (flock as f32 - 1.0) * 100.0;
        
        for _ in 0..20 {
            let position = vec2(
                center_x + random_range(-30.0, 30.0),
                random_range(-50.0, 50.0),
            );
            
            let velocity = vec2(
                random_range(-15.0, 15.0),
                random_range(-15.0, 15.0),
            );
            
            let particle = Particle::new(position)
                .with_velocity(velocity)
                .with_species(0)
                .with_size(3.0)
                .with_color([0.8, 1.0, 0.8, 1.0]);
            
            model.particle_system.add_particle(particle);
        }
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let dt = update.since_last.as_secs_f32();
    model.time += dt;
    
    // Apply forces
    model.force_calculator.apply_forces(&mut model.particle_system);
    
    // Update particles
    model.particle_system.update(dt);
    
    // Update renderer
    model.renderer.update(&model.particle_system, dt);
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Key1 => setup_example(model, 0),
        Key::Key2 => setup_example(model, 1),
        Key::Key3 => setup_example(model, 2),
        Key::Key4 => setup_example(model, 3),
        Key::Key5 => setup_example(model, 4),
        Key::R => setup_example(model, model.current_example),
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    
    // Render particles
    model.renderer.render(&draw, &model.particle_system);
    
    // UI
    let example_names = [
        "1: Lennard-Jones",
        "2: Electromagnetic", 
        "3: Vortex Fields",
        "4: Spring Network",
        "5: Flocking",
    ];
    
    for (i, name) in example_names.iter().enumerate() {
        let color = if i == model.current_example { WHITE } else { GRAY };
        draw.text(name)
            .xy(vec2(-450.0, 300.0 - (i as f32 * 25.0)))
            .font_size(16)
            .color(color);
    }
    
    draw.text("Press 1-5 to switch examples, R to reset")
        .xy(vec2(-450.0, -250.0))
        .font_size(14)
        .color(LIGHTGRAY);
    
    draw.text(&format!("Particles: {}", model.particle_system.particle_count()))
        .xy(vec2(-450.0, -280.0))
        .font_size(14)
        .color(LIGHTGRAY);
    
    // Current example description
    let description = match model.current_example {
        0 => "Particles interact via Lennard-Jones potential\n(attractive at distance, repulsive up close)",
        1 => "Charged particles with electromagnetic forces\nRed=positive, Blue=negative charges",
        2 => "Multiple vortex fields create complex flows\nParticles spiral around force centers",
        3 => "Spring forces create flexible network\nParticles connected by virtual springs",
        4 => "Flocking behavior with separation,\nalignment, and cohesion forces",
        _ => "",
    };
    
    draw.text(description)
        .xy(vec2(-450.0, -320.0))
        .font_size(12)
        .color(DARKGRAY)
        .line_spacing(5.0);
    
    draw.to_frame(app, &frame).unwrap();
}