use std::collections::HashMap;

use glam::{vec2, Mat4, Vec2};

use crate::math::transform::TransformOffset;
use crate::mesh::Mesh;
use crate::nodes::node::InoxNodeUuid;
use crate::nodes::node_data::InoxData;
use crate::nodes::node_tree::InoxNodeTree;
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
    pub vert_offset: u16,
    pub vert_len: usize,
}

#[derive(Debug, Clone)]
pub enum NodeDataRenderInfo {
    Node,
    Part(PartRenderInfo),
    Composite { children: Vec<InoxNodeUuid> },
}

#[derive(Debug)]
pub struct NodeRenderInfo {
    pub trans: Mat4,
    pub trans_offset: TransformOffset,
    pub data: NodeDataRenderInfo,
}

// Implemented for parameter bindings to animate the puppet
// impl AsRef<PartOffsets> for PartRenderInfo {
//     fn as_ref(&self) -> &PartOffsets {
//         &self.part_offsets
//     }
// }

// impl AsMut<PartOffsets> for PartRenderInfo {
//     fn as_mut(&mut self) -> &mut PartOffsets {
//         &mut self.part_offsets
//     }
// }

#[derive(Debug, Clone)]
pub struct CompositeRenderInfo {
    pub children: Vec<InoxNodeUuid>,
}

pub type NodeRenderInfos = HashMap<InoxNodeUuid, NodeRenderInfo>;

#[derive(Debug)]
pub struct RenderInfo {
    pub vertex_info: VertexInfo,
    pub nodes_zsorted: Vec<InoxNodeUuid>,
    pub node_render_infos: NodeRenderInfos,
}

impl RenderInfo {
    fn add_part<T>(
        nodes: &InoxNodeTree<T>,
        uuid: InoxNodeUuid,
        vertex_info: &mut VertexInfo,
        node_render_infos: &mut NodeRenderInfos,
    ) {
        let node = nodes.get_node(uuid).unwrap();

        if let InoxData::Part(ref part) = node.data {
            let (index_offset, vert_offset) = vertex_info.push(&part.mesh);
            node_render_infos.insert(
                uuid,
                NodeRenderInfo {
                    trans: Mat4::default(),
                    trans_offset: node.trans_offset,
                    data: NodeDataRenderInfo::Part(PartRenderInfo {
                        index_offset,
                        vert_offset,
                        vert_len: part.mesh.vertices.len(),
                    }),
                },
            );
        }
    }

    pub fn new<T>(nodes: &InoxNodeTree<T>) -> Self {
        let mut vertex_info = VertexInfo::default();
        let nodes_zsorted = nodes.zsorted_root();
        let mut node_render_infos = HashMap::new();

        for &uuid in &nodes_zsorted {
            let node = nodes.get_node(uuid).unwrap();

            match node.data {
                InoxData::Part(_) => {
                    Self::add_part(nodes, uuid, &mut vertex_info, &mut node_render_infos);
                }
                InoxData::Composite(_) => {
                    // Children include the parent composite, so we have to filter it out.
                    // TODO: wait... does it make sense for it to do that?
                    let children = nodes
                        .zsorted_children(node.uuid)
                        .into_iter()
                        .filter(|uuid| *uuid != node.uuid)
                        .collect::<Vec<_>>();

                    // put composite children's meshes into composite bufs
                    for &uuid in &children {
                        Self::add_part(nodes, uuid, &mut vertex_info, &mut node_render_infos);
                    }

                    node_render_infos.insert(
                        uuid,
                        NodeRenderInfo {
                            trans: Mat4::default(),
                            trans_offset: node.trans_offset,
                            data: NodeDataRenderInfo::Composite { children },
                        },
                    );
                }
                _ => {
                    node_render_infos.insert(
                        uuid,
                        NodeRenderInfo {
                            trans: Mat4::default(),
                            trans_offset: node.trans_offset,
                            data: NodeDataRenderInfo::Node,
                        },
                    );
                }
            }
        }

        Self {
            vertex_info,
            nodes_zsorted,
            node_render_infos,
        }
    }
}

impl Puppet {
    pub fn update(&mut self) {
        let root_node = self.nodes.arena[self.nodes.root].get();
        let node_rinfs = &mut self.render_info.node_render_infos;

        // The root's absolute transform is its relative transform.
        let root_trans = node_rinfs
            .get(&root_node.uuid)
            .unwrap()
            .trans_offset
            .to_matrix();

        // Pre-order traversal, just the order to ensure that parents are accessed earlier than children
        // Skip the root
        for id in self.nodes.root.descendants(&self.nodes.arena).skip(1) {
            let node_index = &self.nodes.arena[id];
            let node = node_index.get();

            if node.lock_to_root {
                let node_render_info = node_rinfs.get_mut(&node.uuid).unwrap();
                node_render_info.trans = root_trans * node_render_info.trans_offset.to_matrix();
            } else {
                let parent = &self.nodes.arena[node_index.parent().unwrap()].get();
                let parent_trans = node_rinfs.get(&parent.uuid).unwrap().trans;

                let node_render_info = node_rinfs.get_mut(&node.uuid).unwrap();
                node_render_info.trans = parent_trans * node_render_info.trans_offset.to_matrix();
            }
        }
    }
}
