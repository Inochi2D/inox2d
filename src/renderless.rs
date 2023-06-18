use std::collections::HashMap;

use glam::{vec2, Vec2};

use crate::math::transform::Transform;
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

#[derive(Debug, Clone)]
pub struct NodeRenderInfo {
    pub trans_offset: Transform,
    pub trans_abs: Transform,
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

type NodeRenderInfos = HashMap<InoxNodeUuid, NodeRenderInfo>;

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

        let mut trans = node.transform;
        trans.update();

        if let InoxData::Part(ref part) = node.data {
            let (index_offset, vert_offset) = vertex_info.push(&part.mesh);
            node_render_infos.insert(
                uuid,
                NodeRenderInfo {
                    trans_offset: trans,
                    trans_abs: Transform::default(),
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
                            trans_offset: node.transform,
                            trans_abs: Transform::default(),
                            data: NodeDataRenderInfo::Composite { children },
                        },
                    );
                }
                _ => {
                    node_render_infos.insert(
                        uuid,
                        NodeRenderInfo {
                            trans_offset: node.transform,
                            trans_abs: Transform::default(),
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
    /// Pre-order traversal, just the order to ensure that parents are accessed earlier than children
    /// Skip the root
    fn update_node(
        &mut self,
        current_uuid: InoxNodeUuid,
        parent_uuid: InoxNodeUuid,
        root_trans_abs: &Transform,
    ) {
        let node_rinfs = &mut self.render_info.node_render_infos;
        let parent_trans_abs = node_rinfs[&parent_uuid].trans_abs;

        let node = self.nodes.get_node(current_uuid).unwrap();
        let node_offset = node_rinfs.get_mut(&current_uuid).unwrap();

        node_offset.trans_offset.update();
        if node.lock_to_root {
            node_offset.trans_abs = *root_trans_abs * node_offset.trans_offset;
        } else {
            node_offset.trans_abs = parent_trans_abs * node_offset.trans_offset;
        }

        for child_uuid in self.nodes.children_uuids(current_uuid).unwrap() {
            self.update_node(child_uuid, current_uuid, root_trans_abs);
        }
    }

    pub fn update(&mut self) {
        let root_node = self.nodes.arena[self.nodes.root].get();

        // The root's absolute transform is its relative transform.
        let root_trans = root_node.transform;
        self.update_node(root_node.uuid, root_node.uuid, &root_trans);
    }
}
