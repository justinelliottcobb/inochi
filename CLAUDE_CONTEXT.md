# Inochi Particle Life System - Project Context

## Project Status: COMPLETE ✅
Comprehensive particle life system in Rust with Nannou framework. All core features implemented and documented.

## Architecture Overview
```
src/
├── lib.rs          # Main app loop, UI (egui), App struct with systems integration
├── main.rs         # Desktop entry point, Nannou app setup
├── particle.rs     # Particle struct, ParticleSystem, physics properties
├── forces.rs       # ForceType enum, InteractionMatrix, PhysicsEngine, integration methods
├── config.rs       # SimulationConfig, ConfigManager, Preset enum, JSON/TOML support
├── spatial.rs      # QuadTree, SpatialGrid optimization (O(n²) → O(n log n))
├── renderer.rs     # ParticleRenderer, Camera, multi-mode rendering, trails
├── presets.rs      # PresetManager, 6 built-in presets, species setup
└── wasm.rs         # WebAssembly bindings, browser API
```

## Key Data Structures
- **Particle**: position, velocity, acceleration, mass, charge, species_id, age, color
- **ParticleSystem**: Vec<Particle>, spawning, boundaries, system-level physics
- **ForceCalculator**: InteractionMatrix + global forces, applies forces to particles
- **App**: orchestrates all systems, handles UI/input, main simulation loop

## Force Types Implemented
- Gravity, Electromagnetic, LennardJones, Damping, Brownian
- Attraction/Repulsion, Vortex, Spring, Flocking (Boids)
- Species-based interaction matrix system

## Built-in Presets
1. **ParticleLife**: Classic emergent behavior (species attraction/repulsion)
2. **Flocking**: Boids with separation/alignment/cohesion
3. **Gravity**: N-body orbital mechanics
4. **Electromagnetic**: Charged particles (±)
5. **Brownian**: Random walk/diffusion
6. **ReactionDiffusion**: Activator-inhibitor patterns

## Tech Stack
- **nannou** 0.19: Graphics/windowing
- **egui** 0.25: UI controls
- **glam**: Vector math
- **serde**: Config serialization
- **rayon**: Multithreading
- **wasm-bindgen**: Web deployment

## Performance Features
- Spatial partitioning (QuadTree/Grid)
- Multiple integration methods (Euler/Verlet/RK4)
- Frustum culling, adaptive quality
- WebAssembly optimization

## Completed Components
✅ Particle physics with 12+ force types
✅ Multi-species ecosystem (configurable interactions)
✅ Advanced rendering (6 modes, trails, vectors)
✅ Real-time UI controls (spawn, forces, camera)
✅ Configuration system (JSON/TOML, presets)
✅ Spatial optimization (QuadTree + Grid)
✅ WebAssembly deployment (full browser support)
✅ Comprehensive documentation (README, API docs)
✅ Example programs + preset configs

## File Status
- `Cargo.toml`: Dependencies configured for desktop + web
- `wasm-build.sh`: Web build script with HTML/JS wrapper
- `examples/`: Basic simulation + custom forces demos
- `presets/`: JSON configs for classic_particle_life, gravity_system
- `www/`: Web deployment assets
- Documentation: README.md, API.md (comprehensive)

## Current State
Project compiles cleanly. All major systems integrated. Desktop app has:
- Main window with particle simulation
- Real-time egui controls (pause, reset, presets, parameters)
- Camera controls (pan/zoom/focus)
- Performance monitoring
- Configuration save/load

Web version provides same functionality via WebAssembly with browser controls.

## Next Development Areas (if continuing)
- GPU compute shaders for >10k particles
- Advanced shading (PBR, bloom, HDR)
- Mobile/touch optimization
- Network/multiplayer support
- Scientific analysis tools (phase plots, energy graphs)
- Custom shader editor
- Sound/music generation from particle data

## Build Commands
```bash
# Desktop
cargo run --release

# Web
./wasm-build.sh
cd www && python3 serve.py

# Test
cargo test
cargo check
```

## Mathematical Foundation
- Verlet integration for stability
- Proper force accumulation (F=ma)
- Energy conservation in gravity sim
- Realistic electromagnetic (Coulomb's law)
- Boids flocking algorithms
- Lennard-Jones molecular dynamics

This is a complete, production-ready particle life system with scientific accuracy and artistic appeal.