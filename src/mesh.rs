use std::collections::BTreeMap;

use glam::{vec2, vec3, IVec2, Vec2, Vec4};

#[derive(thiserror::Error, Debug)]
#[error("Could not convert Vec2s to Vec<Vec2> (the array did not have an even length)")]
pub struct Vec2sToVecVec2Error;

#[derive(Clone, Debug, Default)]
pub struct Vec2s(pub(crate) Vec<f32>);

impl TryFrom<Vec2s> for Vec<Vec2> {
    type Error = Vec2sToVecVec2Error;

    fn try_from(value: Vec2s) -> Result<Self, Self::Error> {
        let chunker = value.0.chunks_exact(2);
        if !chunker.remainder().is_empty() {
            return Err(Vec2sToVecVec2Error);
        }

        let v = chunker.map(|chunk| Vec2::new(chunk[0], chunk[1])).collect();
        Ok(v)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SMesh {
    /// Vertices in the mesh.
    pub vertices: Vec2s,
    /// Base UVs.
    pub uvs: Vec2s,
    /// Indices in the mesh.
    pub indices: Vec<u16>,
    /// Origin of the mesh.
    pub origin: Vec2,
}

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

impl From<&Mesh> for SMesh {
    fn from(mesh: &Mesh) -> Self {
        SMesh {
            vertices: Vec2s(mesh.vertices.iter().flat_map(Vec2::to_array).collect()),
            uvs: Vec2s(mesh.uvs.iter().flat_map(Vec2::to_array).collect()),
            indices: mesh.indices.clone(),
            origin: mesh.origin,
        }
    }
}

impl From<Mesh> for SMesh {
    fn from(mesh: Mesh) -> Self {
        SMesh {
            vertices: Vec2s(mesh.vertices.iter().flat_map(Vec2::to_array).collect()),
            uvs: Vec2s(mesh.uvs.iter().flat_map(Vec2::to_array).collect()),
            indices: mesh.indices,
            origin: mesh.origin,
        }
    }
}

impl TryFrom<SMesh> for Mesh {
    type Error = Vec2sToVecVec2Error;

    fn try_from(smesh: SMesh) -> Result<Self, Self::Error> {
        Ok(Mesh {
            vertices: smesh.vertices.try_into()?,
            uvs: smesh.uvs.try_into()?,
            indices: smesh.indices,
            origin: smesh.origin,
        })
    }
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
    size: IVec2,
    uv_bounds: Vec4,
    cuts: IVec2,
    origin: IVec2,
}

impl Default for QuadBuilder {
    fn default() -> Self {
        Self {
            size: Default::default(),
            uv_bounds: Default::default(),
            cuts: IVec2::new(6, 6),
            origin: Default::default(),
        }
    }
}

impl QuadBuilder {
    /// Size of the mesh.
    pub fn size(mut self, x: i32, y: i32) -> Self {
        self.size = IVec2::new(x, y);
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
        let x = x.max(2);
        let y = y.max(2);

        self.cuts = IVec2::new(x, y);
        self
    }

    pub fn origin(mut self, x: i32, y: i32) -> Self {
        self.origin = IVec2::new(x, y);
        self
    }

    pub fn build(self) -> Mesh {
        let IVec2 { x: sw, y: sh } = self.size / self.cuts;
        let uvx = self.uv_bounds.w / self.cuts.x as f32;
        let uvy = self.uv_bounds.z / self.cuts.y as f32;

        let mut vert_map = BTreeMap::new();
        let mut vertices = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        // Generate vertices and UVs
        for y in 0..=self.cuts.y {
            for x in 0..=self.cuts.x {
                vertices.push(vec2(
                    (x * sw - self.origin.x) as f32,
                    (y * sh - self.origin.y) as f32,
                ));
                uvs.push(vec2(
                    self.uv_bounds.x + x as f32 * uvx,
                    self.uv_bounds.y + y as f32 * uvy,
                ));
                vert_map.insert((x, y), (vertices.len() - 1) as u16);
            }
        }

        // Generate indices
        let center = self.cuts / 2;
        for y in 0..center.y {
            for x in 0..center.x {
                // Indices
                let idx0 = (x, y);
                let idx1 = (x, y + 1);
                let idx2 = (x + 1, y);
                let idx3 = (x + 1, y + 1);

                // We want the vertices to generate in an X pattern so that we won't have too many distortion problems
                if (x < center.x && y < center.y) || (x >= center.x && y >= center.y) {
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
