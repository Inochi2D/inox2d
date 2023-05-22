use std::collections::HashMap;

use glam::{vec2, Vec2};

use crate::math::transform::Transform;
use crate::mesh::Mesh;
use crate::nodes::node::InoxNodeUuid;
use crate::nodes::node_data::InoxData;
use crate::nodes::node_tree::InoxNodeTree;
use crate::params::PartOffsets;
use crate::puppet::Puppet;

#[derive(Debug)]
pub struct VertexInfo {
    pub verts: Vec<Vec2>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub deforms: Vec<Vec2>,
}

impl Default for VertexInfo {
    fn default() -> Self {
        // init with a quad covering the whole viewport

        #[rustfmt::skip]
        let verts = vec![
            vec2(-1.0, -1.0),
            vec2(-1.0,  1.0),
            vec2( 1.0, -1.0),
            vec2( 1.0,  1.0),
        ];

        #[rustfmt::skip]
        let uvs = vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 0.0),
            vec2(1.0, 1.0),
        ];

        #[rustfmt::skip]
        let indices = vec![
            0, 1, 2,
            2, 1, 3,
        ];

        let deforms = vec![Vec2::ZERO; 4];

        Self {
            verts,
            uvs,
            indices,
            deforms,
        }
    }
}

impl VertexInfo {
    /// adds the mesh's vertices and UVs to the buffers and returns its index offset.
    pub fn push(&mut self, mesh: &Mesh) -> (u16, u16) {
        let offset_vert = self.verts.len() as u16;
        let index_offset = self.indices.len() as u16;
        let vert_offset = self.verts.len() as u16;

        self.verts.extend_from_slice(&mesh.vertices);
        self.uvs.extend_from_slice(&mesh.uvs);
        self.indices
            .extend(mesh.indices.iter().map(|index| index + offset_vert));
        let new_deforms = vec![Vec2::ZERO; mesh.vertices.len()];
        self.deforms.extend_from_slice(&new_deforms);

        (index_offset, vert_offset)
    }
}

#[derive(Debug, Clone)]
pub struct PartRenderInfo {
    pub index_offset: u16,
    pub part_offsets: PartOffsets,
}

// Implemented for parameter bindings to animate the puppet
impl AsRef<PartOffsets> for PartRenderInfo {
    fn as_ref(&self) -> &PartOffsets {
        &self.part_offsets
    }
}

impl AsMut<PartOffsets> for PartRenderInfo {
    fn as_mut(&mut self) -> &mut PartOffsets {
        &mut self.part_offsets
    }
}

#[derive(Debug, Clone)]
pub struct CompositeRenderInfo {
    pub children: Vec<InoxNodeUuid>,
}

type PartRenderInfos = HashMap<InoxNodeUuid, PartRenderInfo>;
type CompositeRenderInfos = HashMap<InoxNodeUuid, CompositeRenderInfo>;

#[derive(Debug)]
pub struct RenderInfo {
    pub vertex_info: VertexInfo,

    pub nodes_zsorted: Vec<InoxNodeUuid>,
    pub part_render_infos: PartRenderInfos,
    pub composite_render_infos: CompositeRenderInfos,
}

impl RenderInfo {
    fn add_part<T>(
        nodes: &InoxNodeTree<T>,
        uuid: InoxNodeUuid,
        vertex_info: &mut VertexInfo,
        part_render_infos: &mut PartRenderInfos,
    ) {
        let node = nodes.get_node(uuid).unwrap();

        if let InoxData::Part(ref part) = node.data {
            let (index_offset, vert_offset) = vertex_info.push(&part.mesh);
            part_render_infos.insert(
                uuid,
                PartRenderInfo {
                    index_offset,
                    part_offsets: PartOffsets {
                        vert_offset,
                        vert_len: part.mesh.vertices.len(),
                        trans_offset: Transform::default(),
                        trans_abs: Transform::default(),
                    },
                },
            );
        }
    }

    pub fn new<T>(nodes: &InoxNodeTree<T>) -> Self {
        let mut vertex_info = VertexInfo::default();
        let nodes_zsorted = nodes.zsorted_root();
        let mut part_render_infos = HashMap::new();
        let mut composite_render_infos = HashMap::new();

        for &uuid in &nodes_zsorted {
            let node = nodes.get_node(uuid).unwrap();

            match node.data {
                InoxData::Part(_) => {
                    Self::add_part(nodes, uuid, &mut vertex_info, &mut part_render_infos);
                }
                InoxData::Composite(_) => {
                    // Children include the parent composite, so we have to filter it out.
                    // TODO: wait... does it make sense for it to do that?
                    let children = nodes
                        .zsorted_child(node.uuid)
                        .into_iter()
                        .filter(|uuid| *uuid != node.uuid)
                        .collect::<Vec<_>>();

                    // put composite children's meshes into composite bufs
                    for &uuid in &children {
                        Self::add_part(nodes, uuid, &mut vertex_info, &mut part_render_infos);
                    }

                    composite_render_infos.insert(uuid, CompositeRenderInfo { children });
                }
                _ => (),
            }
        }

        Self {
            vertex_info,
            nodes_zsorted,
            part_render_infos,
            composite_render_infos,
        }
    }
}

impl Puppet {
    pub fn update(&mut self) {
        let root_node = self.nodes.arena[self.nodes.root].get();
        let root_trans = self.render_info.part_render_infos[&root_node.uuid]
            .part_offsets
            .trans_offset;
        // The root's absolute transform is its relative transform.
        self.render_info
            .part_render_infos
            .get_mut(&root_node.uuid)
            .unwrap()
            .part_offsets
            .trans_abs = root_trans;

        // Pre-order traversal, just the order to ensure that parents are accessed earlier than children
        // Skip the root
        let traversal = self.nodes.root.descendants(&self.nodes.arena).skip(1);
        for id in traversal {
            let self_tree_node = &self.nodes.arena[id];
            let self_uuid = self_tree_node.get().uuid;
            let parent_uuid = self.nodes.arena[self_tree_node.parent().unwrap()]
                .get()
                .uuid;

            let parent_trans_abs = self.render_info.part_render_infos[&parent_uuid]
                .part_offsets
                .trans_abs;
            let mut self_part_offset = &mut self
                .render_info
                .part_render_infos
                .get_mut(&self_uuid)
                .unwrap()
                .part_offsets;
            if self_tree_node.get().lock_to_root {
                self_part_offset.trans_abs = root_trans * self_part_offset.trans_offset;
            } else {
                self_part_offset.trans_abs = parent_trans_abs * self_part_offset.trans_offset;
            }
        }
    }
}
