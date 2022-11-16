use std::collections::BTreeMap;

use glam::{vec2, vec3, Vec2, Vec4};

/// Mesh
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    /// Vertices in the mesh.
    pub vertices: Vec<Vec2>,
    /// Base UVs.
    pub uvs: Vec<Vec2>,
    /// Indices in the mesh.
    pub indices: Vec<u16>,
    /// Origin of the mesh.
    pub origin: Vec2,
}

impl Mesh {
    /// Add a new vertex.
    pub fn add(&mut self, vertex: Vec2, uv: Vec2) {
        self.vertices.push(vertex);
        self.uvs.push(uv);
    }

    /// Clear connections/indices.
    pub fn clear_connections(&mut self) {
        self.indices.clear();
    }

    /// Connect 2 vertices together.
    pub fn connect(&mut self, first: u16, second: u16) {
        self.indices.extend([first, second]);
    }

    /// Find the index of a vertex.
    pub fn find(&self, vertex: Vec2) -> Option<usize> {
        self.vertices.iter().position(|v| *v == vertex)
    }

    /// Whether the mesh data is ready to be used.
    pub fn is_ready(&self) -> bool {
        self.can_triangulate()
    }

    /// Whether the mesh data is ready to be triangulated.
    pub fn can_triangulate(&self) -> bool {
        !self.indices.is_empty() && self.indices.len() % 3 == 0
    }

    /// Fixes the winding order of a mesh.
    #[allow(clippy::identity_op)]
    pub fn fix_winding(&mut self) {
        if !self.is_ready() {
            return;
        }

        for i in 0..self.indices.len() / 3 {
            let i = i * 3;

            let vert_a: Vec2 = self.vertices[self.indices[i + 0] as usize];
            let vert_b: Vec2 = self.vertices[self.indices[i + 1] as usize];
            let vert_c: Vec2 = self.vertices[self.indices[i + 2] as usize];

            let vert_ba = vert_b - vert_a;
            let vert_ba = vec3(vert_ba.x, vert_ba.y, 0.);
            let vert_ca = vert_c - vert_a;
            let vert_ca = vec3(vert_ca.x, vert_ca.y, 0.);

            // Swap winding
            if vert_ba.cross(vert_ca).z < 0. {
                self.indices.swap(i + 1, i + 2);
            }
        }
    }

    pub fn connections_at_point(&self, point: Vec2) -> usize {
        self.find(point)
            .map(|idx| self.connections_at_index(idx as u16))
            .unwrap_or(0)
    }

    pub fn connections_at_index(&self, index: u16) -> usize {
        self.indices.iter().filter(|&idx| *idx == index).count()
    }

    /// Generates a quad-based mesh which is cut `cuts` amount of times.
    ///
    /// # Example
    ///
    /// ~~~no_run
    /// Mesh::quad()
    ///     // Size of texture
    ///     .size(texture.width, texture.height)
    ///     // Uses all of UV
    ///     .uv_bounds(vec4(0., 0., 1., 1.))
    ///     // width > height
    ///     .cuts(32, 16)
    /// ~~~
    pub fn quad() -> QuadBuilder {
        QuadBuilder::default()
    }

    pub fn dbg_lens(&self) {
        println!(
            "lengths: {} {} {}",
            self.vertices.len(),
            self.uvs.len(),
            self.indices.len()
        );
    }
}

#[derive(Clone, Debug)]
pub struct QuadBuilder {
    size: (i32, i32),
    uv_bounds: Vec4,
    cuts: (i32, i32),
    origin: (i32, i32),
}

impl Default for QuadBuilder {
    fn default() -> Self {
        Self {
            size: Default::default(),
            uv_bounds: Default::default(),
            cuts: (6, 6),
            origin: Default::default(),
        }
    }
}

impl QuadBuilder {
    /// Size of the mesh.
    pub fn size(mut self, x: i32, y: i32) -> Self {
        self.size = (x, y);
        self
    }

    /// x, y UV coordinates + width/height in UV coordinate space.
    pub fn uv_bounds(mut self, uv_bounds: Vec4) -> Self {
        self.uv_bounds = uv_bounds;
        self
    }

    /// Cuts are how many times to cut the mesh on the X and Y axis.
    ///
    /// Note: splits may not be below 2, so they are clamped automatically.
    pub fn cuts(mut self, x: i32, y: i32) -> Self {
        let x = if x < 2 { 2 } else { x };
        let y = if y < 2 { 2 } else { y };

        self.cuts = (x, y);
        self
    }

    pub fn origin(mut self, x: i32, y: i32) -> Self {
        self.origin = (x, y);
        self
    }

    pub fn build(self) -> Mesh {
        let sw = self.size.0 / self.cuts.0;
        let sh = self.size.1 / self.cuts.1;
        let uvx = self.uv_bounds.w / self.cuts.0 as f32;
        let uvy = self.uv_bounds.z / self.cuts.1 as f32;

        let mut vert_map = BTreeMap::new();
        let mut vertices = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        // Generate vertices and UVs
        for y in 0..=self.cuts.1 {
            for x in 0..=self.cuts.0 {
                vertices.push(vec2(
                    (x * sw - self.origin.0) as f32,
                    (y * sh - self.origin.1) as f32,
                ));
                uvs.push(vec2(
                    self.uv_bounds.x + x as f32 * uvx,
                    self.uv_bounds.y + y as f32 * uvy,
                ));
                vert_map.insert((x, y), (vertices.len() - 1) as u16);
            }
        }

        // Generate indices
        let (cx, cy) = (self.cuts.0 / 2, self.cuts.1 / 2);
        for y in 0..cy {
            for x in 0..cx {
                // Indices
                let idx0 = (x, y);
                let idx1 = (x, y + 1);
                let idx2 = (x + 1, y);
                let idx3 = (x + 1, y + 1);

                // We want the vertices to generate in an X pattern so that we won't have too many distortion problems
                if (x < cx && y < cy) || (x >= cx && y >= cy) {
                    indices.extend([
                        vert_map[&idx0],
                        vert_map[&idx2],
                        vert_map[&idx3],
                        vert_map[&idx0],
                        vert_map[&idx3],
                        vert_map[&idx1],
                    ]);
                } else {
                    indices.extend([
                        vert_map[&idx0],
                        vert_map[&idx1],
                        vert_map[&idx2],
                        vert_map[&idx1],
                        vert_map[&idx2],
                        vert_map[&idx3],
                    ]);
                }
            }
        }

        Mesh {
            vertices,
            uvs,
            indices,
            origin: Vec2::default(),
        }
    }
}
