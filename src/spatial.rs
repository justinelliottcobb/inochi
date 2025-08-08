use glam::Vec2;
use crate::particle::Particle;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SpatialGrid {
    cell_size: f32,
    bounds: (Vec2, Vec2),
    grid: HashMap<(i32, i32), Vec<usize>>,
    particle_positions: Vec<Vec2>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32, bounds: (Vec2, Vec2)) -> Self {
        Self {
            cell_size,
            bounds,
            grid: HashMap::new(),
            particle_positions: Vec::new(),
        }
    }

    pub fn update(&mut self, particles: &[Particle]) {
        self.grid.clear();
        self.particle_positions.clear();
        self.particle_positions.reserve(particles.len());

        for (index, particle) in particles.iter().enumerate() {
            let cell = self.position_to_cell(particle.position);
            self.grid.entry(cell).or_insert_with(Vec::new).push(index);
            self.particle_positions.push(particle.position);
        }
    }

    pub fn query_neighbors(&self, position: Vec2, radius: f32) -> Vec<usize> {
        let mut neighbors = Vec::new();
        let min_cell = self.position_to_cell(position - Vec2::splat(radius));
        let max_cell = self.position_to_cell(position + Vec2::splat(radius));

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(indices) = self.grid.get(&(x, y)) {
                    for &index in indices {
                        if let Some(particle_pos) = self.particle_positions.get(index) {
                            let distance = position.distance(*particle_pos);
                            if distance <= radius {
                                neighbors.push(index);
                            }
                        }
                    }
                }
            }
        }

        neighbors
    }

    pub fn query_neighbors_in_range(&self, particle_index: usize, radius: f32) -> Vec<usize> {
        if let Some(position) = self.particle_positions.get(particle_index) {
            let neighbors = self.query_neighbors(*position, radius);
            neighbors.into_iter().filter(|&i| i != particle_index).collect()
        } else {
            Vec::new()
        }
    }

    fn position_to_cell(&self, position: Vec2) -> (i32, i32) {
        let x = ((position.x - self.bounds.0.x) / self.cell_size).floor() as i32;
        let y = ((position.y - self.bounds.0.y) / self.cell_size).floor() as i32;
        (x, y)
    }

    pub fn get_cell_count(&self) -> usize {
        self.grid.len()
    }

    pub fn get_max_particles_per_cell(&self) -> usize {
        self.grid.values().map(|v| v.len()).max().unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct QuadTree {
    bounds: (Vec2, Vec2),
    particles: Vec<usize>,
    children: Option<Box<[QuadTree; 4]>>,
    max_particles: usize,
    max_depth: usize,
    depth: usize,
}

impl QuadTree {
    pub fn new(bounds: (Vec2, Vec2), max_particles: usize, max_depth: usize) -> Self {
        Self {
            bounds,
            particles: Vec::new(),
            children: None,
            max_particles,
            max_depth,
            depth: 0,
        }
    }

    fn new_with_depth(bounds: (Vec2, Vec2), max_particles: usize, max_depth: usize, depth: usize) -> Self {
        Self {
            bounds,
            particles: Vec::new(),
            children: None,
            max_particles,
            max_depth,
            depth,
        }
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.children = None;
    }

    pub fn insert(&mut self, particle_index: usize, position: Vec2) -> bool {
        if !self.contains_point(position) {
            return false;
        }

        if self.particles.len() < self.max_particles || self.depth >= self.max_depth {
            self.particles.push(particle_index);
            return true;
        }

        if self.children.is_none() {
            self.subdivide();
        }

        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                if child.insert(particle_index, position) {
                    return true;
                }
            }
        }

        // Fallback: add to current node if children can't accommodate
        self.particles.push(particle_index);
        true
    }

    pub fn query_range(&self, range: (Vec2, Vec2), results: &mut Vec<usize>, particle_positions: &[Vec2]) {
        if !self.intersects_range(range) {
            return;
        }

        // Check particles in this node
        for &index in &self.particles {
            if let Some(pos) = particle_positions.get(index) {
                if pos.x >= range.0.x && pos.x <= range.1.x &&
                   pos.y >= range.0.y && pos.y <= range.1.y {
                    results.push(index);
                }
            }
        }

        // Check children
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_range(range, results, particle_positions);
            }
        }
    }

    pub fn query_radius(&self, center: Vec2, radius: f32, results: &mut Vec<usize>, particle_positions: &[Vec2]) {
        let range = (
            center - Vec2::splat(radius),
            center + Vec2::splat(radius)
        );

        if !self.intersects_range(range) {
            return;
        }

        let radius_squared = radius * radius;

        // Check particles in this node
        for &index in &self.particles {
            if let Some(pos) = particle_positions.get(index) {
                if center.distance_squared(*pos) <= radius_squared {
                    results.push(index);
                }
            }
        }

        // Check children
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_radius(center, radius, results, particle_positions);
            }
        }
    }

    fn subdivide(&mut self) {
        let (min, max) = self.bounds;
        let center = (min + max) * 0.5;

        self.children = Some(Box::new([
            // Top-left
            QuadTree::new_with_depth(
                (min, center),
                self.max_particles,
                self.max_depth,
                self.depth + 1
            ),
            // Top-right
            QuadTree::new_with_depth(
                (Vec2::new(center.x, min.y), Vec2::new(max.x, center.y)),
                self.max_particles,
                self.max_depth,
                self.depth + 1
            ),
            // Bottom-left
            QuadTree::new_with_depth(
                (Vec2::new(min.x, center.y), Vec2::new(center.x, max.y)),
                self.max_particles,
                self.max_depth,
                self.depth + 1
            ),
            // Bottom-right
            QuadTree::new_with_depth(
                (center, max),
                self.max_particles,
                self.max_depth,
                self.depth + 1
            ),
        ]));

        // Redistribute particles to children
        let particles_to_redistribute = std::mem::take(&mut self.particles);
        // We need particle positions to redistribute, but we don't have them here
        // In practice, this would be called from update() which has access to positions
        self.particles = particles_to_redistribute; // Keep them for now
    }

    fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.bounds.0.x && point.x <= self.bounds.1.x &&
        point.y >= self.bounds.0.y && point.y <= self.bounds.1.y
    }

    fn intersects_range(&self, range: (Vec2, Vec2)) -> bool {
        !(self.bounds.1.x < range.0.x || self.bounds.0.x > range.1.x ||
          self.bounds.1.y < range.0.y || self.bounds.0.y > range.1.y)
    }

    pub fn get_node_count(&self) -> usize {
        let mut count = 1;
        if let Some(ref children) = self.children {
            for child in children.iter() {
                count += child.get_node_count();
            }
        }
        count
    }

    pub fn get_max_depth(&self) -> usize {
        let mut max_depth = self.depth;
        if let Some(ref children) = self.children {
            for child in children.iter() {
                max_depth = max_depth.max(child.get_max_depth());
            }
        }
        max_depth
    }
}

pub struct QuadTreeManager {
    quadtree: QuadTree,
    particle_positions: Vec<Vec2>,
}

impl QuadTreeManager {
    pub fn new(bounds: (Vec2, Vec2), max_particles_per_node: usize, max_depth: usize) -> Self {
        Self {
            quadtree: QuadTree::new(bounds, max_particles_per_node, max_depth),
            particle_positions: Vec::new(),
        }
    }

    pub fn update(&mut self, particles: &[Particle]) {
        self.quadtree.clear();
        self.particle_positions.clear();
        self.particle_positions.reserve(particles.len());

        for (index, particle) in particles.iter().enumerate() {
            self.particle_positions.push(particle.position);
            self.quadtree.insert(index, particle.position);
        }
    }

    pub fn query_neighbors(&self, position: Vec2, radius: f32) -> Vec<usize> {
        let mut results = Vec::new();
        self.quadtree.query_radius(position, radius, &mut results, &self.particle_positions);
        results
    }

    pub fn query_neighbors_for_particle(&self, particle_index: usize, radius: f32) -> Vec<usize> {
        if let Some(position) = self.particle_positions.get(particle_index) {
            let neighbors = self.query_neighbors(*position, radius);
            neighbors.into_iter().filter(|&i| i != particle_index).collect()
        } else {
            Vec::new()
        }
    }

    pub fn query_range(&self, range: (Vec2, Vec2)) -> Vec<usize> {
        let mut results = Vec::new();
        self.quadtree.query_range(range, &mut results, &self.particle_positions);
        results
    }

    pub fn get_statistics(&self) -> SpatialStatistics {
        SpatialStatistics {
            node_count: self.quadtree.get_node_count(),
            max_depth: self.quadtree.get_max_depth(),
            particle_count: self.particle_positions.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpatialStatistics {
    pub node_count: usize,
    pub max_depth: usize,
    pub particle_count: usize,
}

pub enum SpatialPartitioning {
    Grid(SpatialGrid),
    QuadTree(QuadTreeManager),
}

impl SpatialPartitioning {
    pub fn new_grid(cell_size: f32, bounds: (Vec2, Vec2)) -> Self {
        SpatialPartitioning::Grid(SpatialGrid::new(cell_size, bounds))
    }

    pub fn new_quadtree(bounds: (Vec2, Vec2), max_particles_per_node: usize, max_depth: usize) -> Self {
        SpatialPartitioning::QuadTree(QuadTreeManager::new(bounds, max_particles_per_node, max_depth))
    }

    pub fn update(&mut self, particles: &[Particle]) {
        match self {
            SpatialPartitioning::Grid(grid) => grid.update(particles),
            SpatialPartitioning::QuadTree(quadtree) => quadtree.update(particles),
        }
    }

    pub fn query_neighbors(&self, position: Vec2, radius: f32) -> Vec<usize> {
        match self {
            SpatialPartitioning::Grid(grid) => grid.query_neighbors(position, radius),
            SpatialPartitioning::QuadTree(quadtree) => quadtree.query_neighbors(position, radius),
        }
    }

    pub fn query_neighbors_for_particle(&self, particle_index: usize, radius: f32) -> Vec<usize> {
        match self {
            SpatialPartitioning::Grid(grid) => grid.query_neighbors_in_range(particle_index, radius),
            SpatialPartitioning::QuadTree(quadtree) => quadtree.query_neighbors_for_particle(particle_index, radius),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::particle::Particle;

    #[test]
    fn test_spatial_grid() {
        let mut grid = SpatialGrid::new(10.0, (Vec2::new(-50.0, -50.0), Vec2::new(50.0, 50.0)));
        
        let particles = vec![
            Particle::new(Vec2::new(5.0, 5.0)),
            Particle::new(Vec2::new(15.0, 15.0)),
            Particle::new(Vec2::new(-5.0, -5.0)),
        ];
        
        grid.update(&particles);
        
        let neighbors = grid.query_neighbors(Vec2::new(0.0, 0.0), 20.0);
        assert!(!neighbors.is_empty());
    }

    #[test]
    fn test_quadtree() {
        let mut manager = QuadTreeManager::new(
            (Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0)),
            4,
            8
        );
        
        let particles = vec![
            Particle::new(Vec2::new(10.0, 10.0)),
            Particle::new(Vec2::new(20.0, 20.0)),
            Particle::new(Vec2::new(-10.0, -10.0)),
            Particle::new(Vec2::new(50.0, 50.0)),
        ];
        
        manager.update(&particles);
        
        let neighbors = manager.query_neighbors(Vec2::new(15.0, 15.0), 25.0);
        assert!(neighbors.len() >= 2); // Should find at least the two nearby particles
    }

    #[test]
    fn test_quadtree_range_query() {
        let mut manager = QuadTreeManager::new(
            (Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0)),
            4,
            8
        );
        
        let particles = vec![
            Particle::new(Vec2::new(10.0, 10.0)),
            Particle::new(Vec2::new(20.0, 20.0)),
            Particle::new(Vec2::new(-50.0, -50.0)),
        ];
        
        manager.update(&particles);
        
        let range_results = manager.query_range((Vec2::new(0.0, 0.0), Vec2::new(30.0, 30.0)));
        assert_eq!(range_results.len(), 2); // Should find the two particles in range
    }

    #[test]
    fn test_spatial_partitioning_enum() {
        let mut spatial = SpatialPartitioning::new_grid(10.0, (Vec2::new(-50.0, -50.0), Vec2::new(50.0, 50.0)));
        
        let particles = vec![
            Particle::new(Vec2::new(5.0, 5.0)),
            Particle::new(Vec2::new(15.0, 15.0)),
        ];
        
        spatial.update(&particles);
        
        let neighbors = spatial.query_neighbors(Vec2::new(10.0, 10.0), 10.0);
        assert!(!neighbors.is_empty());
    }
}