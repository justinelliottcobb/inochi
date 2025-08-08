# Inochi API Documentation

## Core Types and Structures

### Particle

The fundamental unit of the simulation.

```rust
pub struct Particle {
    pub position: Vec2,        // Current position
    pub velocity: Vec2,        // Current velocity
    pub acceleration: Vec2,    // Current acceleration
    pub mass: f32,            // Mass for force calculations
    pub charge: f32,          // Electric charge
    pub age: f32,             // Time since creation
    pub lifespan: f32,        // Total lifetime (f32::INFINITY for immortal)
    pub color: [f32; 4],      // RGBA color
    pub species_id: u32,      // Species identifier for interactions
    pub energy: f32,          // Kinetic energy (calculated)
    pub size: f32,            // Visual size
    pub temperature: f32,     // Temperature for thermal effects
}
```

#### Methods

- `new(position: Vec2) -> Particle` - Create a new particle at position
- `with_velocity(velocity: Vec2) -> Self` - Set initial velocity
- `with_mass(mass: f32) -> Self` - Set mass
- `with_charge(charge: f32) -> Self` - Set electric charge
- `with_species(species_id: u32) -> Self` - Set species ID
- `with_color(color: [f32; 4]) -> Self` - Set RGBA color
- `with_lifespan(lifespan: f32) -> Self` - Set lifetime
- `with_size(size: f32) -> Self` - Set visual size
- `is_alive() -> bool` - Check if particle is still alive
- `life_ratio() -> f32` - Get remaining life as ratio (0.0 to 1.0)
- `update(dt: f32)` - Update particle physics
- `apply_force(force: Vec2)` - Apply a force vector
- `apply_impulse(impulse: Vec2)` - Apply an impulse (instant velocity change)
- `distance_to(&other: Particle) -> f32` - Distance to another particle
- `direction_to(&other: Particle) -> Vec2` - Unit vector toward another particle
- `kinetic_energy() -> f32` - Calculate kinetic energy
- `momentum() -> Vec2` - Calculate momentum vector

### ParticleSystem

Container and manager for particles.

```rust
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub max_particles: usize,
    pub spawn_rate: f32,
    pub spawn_timer: f32,
    pub bounds: Option<(Vec2, Vec2)>,
    pub wrap_boundaries: bool,
    pub damping: f32,
}
```

#### Methods

- `new(max_particles: usize) -> Self` - Create new system
- `add_particle(particle: Particle)` - Add a particle to the system
- `spawn_particle_at(position: Vec2)` - Spawn a default particle at position
- `update(dt: f32)` - Update all particles and handle spawning
- `particle_count() -> usize` - Get current particle count
- `total_energy() -> f32` - Calculate total system energy
- `center_of_mass() -> Vec2` - Calculate center of mass
- `average_velocity() -> Vec2` - Calculate average velocity
- `clear()` - Remove all particles
- `set_bounds(min: Vec2, max: Vec2)` - Set simulation boundaries
- `remove_bounds()` - Remove boundary constraints

## Force System

### ForceType

Enumeration of available force types.

```rust
pub enum ForceType {
    Gravity { strength: f32, min_distance: f32 },
    ElectroMagnetic { strength: f32, min_distance: f32 },
    LennardJones { epsilon: f32, sigma: f32 },
    Damping { coefficient: f32 },
    Brownian { intensity: f32 },
    Attraction { strength: f32, max_distance: f32 },
    Repulsion { strength: f32, max_distance: f32 },
    Vortex { center: Vec2, strength: f32, max_distance: f32 },
    Spring { rest_length: f32, stiffness: f32, damping: f32 },
    Flocking {
        separation_radius: f32,
        alignment_radius: f32,
        cohesion_radius: f32,
        separation_strength: f32,
        alignment_strength: f32,
        cohesion_strength: f32,
    },
}
```

### InteractionMatrix

Manages force interactions between particle species.

```rust
pub struct InteractionMatrix {
    pub interactions: HashMap<(u32, u32), Vec<ForceType>>,
    pub default_forces: Vec<ForceType>,
}
```

#### Methods

- `new() -> Self` - Create new interaction matrix
- `add_interaction(species_a: u32, species_b: u32, force: ForceType)` - Add force between species
- `get_forces(species_a: u32, species_b: u32) -> &[ForceType]` - Get forces for species pair

### ForceCalculator

Applies forces to particles based on interaction rules.

```rust
pub struct ForceCalculator {
    pub interaction_matrix: InteractionMatrix,
    pub global_forces: Vec<ForceType>,
    pub dt: f32,
}
```

#### Methods

- `new() -> Self` - Create new force calculator
- `with_dt(dt: f32) -> Self` - Set time step
- `add_global_force(force: ForceType)` - Add force affecting all particles
- `apply_forces(system: &mut ParticleSystem)` - Apply all forces to system

### PhysicsEngine

High-level physics manager with different integration methods.

```rust
pub struct PhysicsEngine {
    pub config: PhysicsConfig,
    pub force_calculator: ForceCalculator,
}
```

#### Methods

- `new(config: PhysicsConfig) -> Self` - Create physics engine with configuration
- `update(system: &mut ParticleSystem)` - Update system physics

## Spatial Optimization

### SpatialPartitioning

Enum for different spatial partitioning strategies.

```rust
pub enum SpatialPartitioning {
    Grid(SpatialGrid),
    QuadTree(QuadTreeManager),
}
```

#### Methods

- `new_grid(cell_size: f32, bounds: (Vec2, Vec2)) -> Self` - Create grid partitioning
- `new_quadtree(bounds: (Vec2, Vec2), max_particles: usize, max_depth: usize) -> Self` - Create quadtree
- `update(particles: &[Particle])` - Update spatial structure
- `query_neighbors(position: Vec2, radius: f32) -> Vec<usize>` - Find nearby particles
- `query_neighbors_for_particle(particle_index: usize, radius: f32) -> Vec<usize>` - Find neighbors of specific particle

## Rendering System

### ParticleRenderer

Handles all particle visualization.

```rust
pub struct ParticleRenderer {
    config: RenderConfig,
    trail_history: Vec<VecDeque<Vec2>>,
    pub camera: Camera,
}
```

#### Methods

- `new(config: RenderConfig) -> Self` - Create new renderer
- `update_config(config: RenderConfig)` - Update rendering configuration
- `update(system: &ParticleSystem, dt: f32)` - Update renderer state
- `render(draw: &Draw, system: &ParticleSystem)` - Render particles
- `handle_mouse_input(mouse_pos: Vec2, screen_size: Vec2)` - Handle mouse interaction
- `handle_zoom(zoom_delta: f32)` - Handle zoom input
- `handle_pan(delta: Vec2)` - Handle camera panning
- `reset_camera()` - Reset camera to default position
- `focus_on_particles(system: &ParticleSystem)` - Focus camera on particle center of mass

### Camera

Camera control system for viewing the simulation.

```rust
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub target: Option<Vec2>,
    pub smoothing: f32,
}
```

#### Methods

- `new() -> Self` - Create new camera
- `set_target(target: Vec2)` - Set camera target for smooth following
- `clear_target()` - Remove camera target
- `update(dt: f32)` - Update camera position (smooth movement)
- `world_to_screen(world_pos: Vec2, screen_size: Vec2) -> Vec2` - Convert world to screen coordinates
- `screen_to_world(screen_pos: Vec2, screen_size: Vec2) -> Vec2` - Convert screen to world coordinates
- `get_view_bounds(screen_size: Vec2) -> (Vec2, Vec2)` - Get visible world bounds

## Configuration System

### SimulationConfig

Main configuration structure containing all simulation parameters.

```rust
pub struct SimulationConfig {
    pub physics: PhysicsConfig,
    pub rendering: RenderConfig,
    pub particles: ParticleConfig,
    pub forces: ForceConfig,
    pub ui: UiConfig,
    pub performance: PerformanceConfig,
}
```

### ConfigManager

Manages loading, saving, and applying configurations.

```rust
pub struct ConfigManager {
    config: SimulationConfig,
    config_path: Option<String>,
}
```

#### Methods

- `new() -> Self` - Create with default configuration
- `from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>>` - Load from file
- `save_to_file(path: &str) -> Result<(), Box<dyn std::error::Error>>` - Save to file
- `save() -> Result<(), Box<dyn std::error::Error>>` - Save to original path
- `config() -> &SimulationConfig` - Get configuration reference
- `config_mut() -> &mut SimulationConfig` - Get mutable configuration reference
- `reset_to_defaults()` - Reset to default values
- `apply_preset(preset: Preset)` - Apply a predefined preset

## Preset System

### Preset

Enumeration of built-in simulation presets.

```rust
pub enum Preset {
    ParticleLife,
    Flocking,
    Gravity,
    Electromagnetic,
    Brownian,
    ReactionDiffusion,
}
```

#### Methods

- `all() -> Vec<Preset>` - Get all available presets
- `name(&self) -> &str` - Get preset display name
- `description(&self) -> &str` - Get preset description

### PresetManager

Utility for creating and managing presets.

#### Static Methods

- `create_particle_system_from_preset(preset: &Preset, config: &SimulationConfig) -> ParticleSystem` - Create system from preset
- `create_interaction_matrix(preset: &Preset) -> InteractionMatrix` - Create interaction matrix for preset
- `get_species_color(species_id: u32) -> [f32; 4]` - Get default color for species
- `create_test_scenario(scenario_name: &str) -> (ParticleSystem, InteractionMatrix)` - Create test scenarios
- `save_custom_preset(name: &str, config: &SimulationConfig, system: &ParticleSystem) -> Result<(), Box<dyn std::error::Error>>` - Save custom preset
- `load_custom_preset(name: &str) -> Result<CustomPresetData, Box<dyn std::error::Error>>` - Load custom preset
- `list_custom_presets() -> Result<Vec<String>, Box<dyn std::error::Error>>` - List available custom presets

## WebAssembly API

When compiled to WebAssembly, additional functions are available:

### Simulation Control

- `start_simulation() -> Result<(), JsValue>` - Initialize and start the simulation
- `reset_simulation()` - Reset simulation to initial state
- `toggle_pause()` - Pause/unpause simulation
- `change_preset(preset_name: &str)` - Switch to different preset

### Data Access

- `get_particle_count() -> usize` - Get current particle count
- `get_fps() -> f32` - Get current FPS
- `get_particles() -> Vec<f32>` - Get particle data as flat array
- `get_performance_stats() -> Vec<f32>` - Get performance metrics

### Interaction

- `add_particle(x: f32, y: f32, species_id: u32)` - Add particle at position
- `set_camera_position(x: f32, y: f32)` - Set camera position
- `set_camera_zoom(zoom: f32)` - Set camera zoom level
- `handle_mouse_drag(dx: f32, dy: f32)` - Handle mouse dragging
- `handle_mouse_wheel(delta: f32)` - Handle mouse wheel input

### Configuration

- `set_force_strength(force_type: &str, strength: f32)` - Adjust force strength
- `set_spawn_rate(rate: f32)` - Set particle spawn rate
- `enable_trails(enable: bool)` - Enable/disable particle trails
- `set_background_color(r: f32, g: f32, b: f32, a: f32)` - Set background color
- `export_config() -> String` - Export configuration as JSON
- `import_config(config_json: &str) -> bool` - Import configuration from JSON

### Debugging

- `log_particle_info(index: usize)` - Log particle information to console
- `get_system_info() -> String` - Get system statistics as string

## Usage Examples

### Creating a Custom Simulation

```rust
use inochi::*;

// Create particle system
let mut system = ParticleSystem::new(100);

// Add particles
for i in 0..50 {
    let particle = Particle::new(Vec2::new(i as f32 * 10.0, 0.0))
        .with_velocity(Vec2::new(0.0, 10.0))
        .with_species(i % 3)
        .with_size(2.0 + i as f32 * 0.1);
    
    system.add_particle(particle);
}

// Set up forces
let mut force_calc = ForceCalculator::new();
force_calc.add_global_force(ForceType::Gravity {
    strength: 100.0,
    min_distance: 1.0,
});

// Add species interactions
force_calc.interaction_matrix.add_interaction(
    0, 1,
    ForceType::Attraction {
        strength: 50.0,
        max_distance: 100.0,
    }
);

// Create physics engine
let mut physics = PhysicsEngine::new(PhysicsConfig::default());
physics.force_calculator = force_calc;

// Simulation loop
loop {
    physics.update(&mut system);
    system.update(0.016); // 60 FPS
}
```

### Custom Force Implementation

```rust
// Create a custom vortex force
let vortex = ForceType::Vortex {
    center: Vec2::new(0.0, 0.0),
    strength: 50.0,
    max_distance: 200.0,
};

// Add to force calculator
force_calculator.add_global_force(vortex);

// Or create species-specific interaction
force_calculator.interaction_matrix.add_interaction(
    0, 2, // Species 0 to Species 2
    ForceType::LennardJones {
        epsilon: 4.0,
        sigma: 12.0,
    }
);
```

### Configuration Management

```rust
// Load configuration from file
let mut config_manager = ConfigManager::from_file("my_config.json")?;

// Modify parameters
config_manager.config_mut().particles.max_particles = 1000;
config_manager.config_mut().forces.gravity_strength = 200.0;

// Apply preset
config_manager.apply_preset(Preset::Flocking);

// Save modified configuration
config_manager.save_to_file("modified_config.json")?;
```

This API provides comprehensive control over all aspects of the particle simulation, from low-level particle manipulation to high-level preset management.