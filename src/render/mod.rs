#[cfg(feature = "opengl")]
pub mod opengl;

#[cfg(feature = "wgpu")]
pub mod wgpu;

use std::collections::HashMap;

use glam::{vec2, Mat4, Vec2};

use crate::math::transform::TransformOffset;
use crate::mesh::Mesh;
use crate::nodes::node::InoxNodeUuid;
use crate::nodes::node_data::InoxData;
use crate::nodes::node_tree::InoxNodeTree;
use crate::puppet::Puppet;

#[derive(Debug)]
pub struct VertexBuffers {
    pub verts: Vec<Vec2>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub deforms: Vec<Vec2>,
}

impl Default for VertexBuffers {
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

impl VertexBuffers {
    /// adds the mesh's vertices and UVs to the buffers and returns its index offset.
    pub fn push(&mut self, mesh: &Mesh) -> (u16, u16) {
        let index_offset = self.indices.len() as u16;
        let vert_offset = self.verts.len() as u16;

        self.verts.extend_from_slice(&mesh.vertices);
        self.uvs.extend_from_slice(&mesh.uvs);
        self.indices
            .extend(mesh.indices.iter().map(|index| index + vert_offset));
        self.deforms
            .resize(self.deforms.len() + mesh.vertices.len(), Vec2::ZERO);

        (index_offset, vert_offset)
    }
}

#[derive(Debug, Clone)]
pub struct PartRenderCtx {
    pub index_offset: u16,
    pub vert_offset: u16,
    pub index_len: usize,
    pub vert_len: usize,
}

#[derive(Debug, Clone)]
pub enum RenderCtxKind {
    Node,
    Part(PartRenderCtx),
    Composite(Vec<InoxNodeUuid>),
}

#[derive(Debug)]
pub struct NodeRenderCtx {
    pub trans: Mat4,
    pub trans_offset: TransformOffset,
    pub kind: RenderCtxKind,
}

pub type NodeRenderCtxs = HashMap<InoxNodeUuid, NodeRenderCtx>;

#[derive(Debug)]
pub struct RenderCtx {
    pub vertex_buffers: VertexBuffers,
    pub nodes_zsorted: Vec<InoxNodeUuid>,
    pub node_render_ctxs: NodeRenderCtxs,
}

impl RenderCtx {
    fn add_part<T>(
        nodes: &InoxNodeTree<T>,
        uuid: InoxNodeUuid,
        vertex_buffers: &mut VertexBuffers,
        node_render_ctxs: &mut NodeRenderCtxs,
    ) {
        let node = nodes.get_node(uuid).unwrap();

        if let InoxData::Part(ref part) = node.data {
            let (index_offset, vert_offset) = vertex_buffers.push(&part.mesh);
            node_render_ctxs.insert(
                uuid,
                NodeRenderCtx {
                    trans: Mat4::default(),
                    trans_offset: node.trans_offset,
                    kind: RenderCtxKind::Part(PartRenderCtx {
                        index_offset,
                        vert_offset,
                        index_len: part.mesh.indices.len(),
                        vert_len: part.mesh.vertices.len(),
                    }),
                },
            );
        }
    }

    pub fn new<T>(nodes: &InoxNodeTree<T>) -> Self {
        let mut vertex_buffers = VertexBuffers::default();
        let nodes_zsorted = nodes.zsorted_root();
        let mut node_render_ctxs = HashMap::new();

        for &uuid in &nodes_zsorted {
            let node = nodes.get_node(uuid).unwrap();

            match node.data {
                InoxData::Part(_) => {
                    Self::add_part(nodes, uuid, &mut vertex_buffers, &mut node_render_ctxs);
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
                        Self::add_part(nodes, uuid, &mut vertex_buffers, &mut node_render_ctxs);
                    }

                    node_render_ctxs.insert(
                        uuid,
                        NodeRenderCtx {
                            trans: Mat4::default(),
                            trans_offset: node.trans_offset,
                            kind: RenderCtxKind::Composite(children),
                        },
                    );
                }
                _ => {
                    node_render_ctxs.insert(
                        uuid,
                        NodeRenderCtx {
                            trans: Mat4::default(),
                            trans_offset: node.trans_offset,
                            kind: RenderCtxKind::Node,
                        },
                    );
                }
            }
        }

        Self {
            vertex_buffers,
            nodes_zsorted,
            node_render_ctxs,
        }
    }
}

impl Puppet {
    /// Update the puppet's nodes' absolute transforms, by combining transforms
    /// from each node's ancestors in a pre-order traversal manner.
    pub fn update_trans(&mut self) {
        let root_node = self.nodes.arena[self.nodes.root].get();
        let node_rctxs = &mut self.render_ctx.node_render_ctxs;

        // The root's absolute transform is its relative transform.
        let root_trans = node_rctxs
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
                let node_render_ctx = node_rctxs.get_mut(&node.uuid).unwrap();
                node_render_ctx.trans = root_trans * node_render_ctx.trans_offset.to_matrix();
            } else {
                let parent = &self.nodes.arena[node_index.parent().unwrap()].get();
                let parent_trans = node_rctxs.get(&parent.uuid).unwrap().trans;

                let node_render_ctx = node_rctxs.get_mut(&node.uuid).unwrap();
                node_render_ctx.trans = parent_trans * node_render_ctx.trans_offset.to_matrix();
            }
        }
    }
}
