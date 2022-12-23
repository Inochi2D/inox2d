use std::collections::BTreeMap;
use std::fmt::Display;

use indextree::{Arena, NodeId};

use super::node::{ExtInoxNode, InoxNodeUuid};

pub type InoxNodeTree = ExtInoxNodeTree<()>;

#[derive(Debug)]
pub struct ExtInoxNodeTree<T> {
    pub root: indextree::NodeId,
    pub arena: Arena<ExtInoxNode<T>>,
    pub uuids: BTreeMap<InoxNodeUuid, indextree::NodeId>,
}

impl<T> ExtInoxNodeTree<T> {
    fn get_internal_node(&self, uuid: InoxNodeUuid) -> Option<&indextree::Node<ExtInoxNode<T>>> {
        self.arena.get(*self.uuids.get(&uuid)?)
    }
    fn get_internal_node_mut(
        &mut self,
        uuid: InoxNodeUuid,
    ) -> Option<&mut indextree::Node<ExtInoxNode<T>>> {
        self.arena.get_mut(*self.uuids.get(&uuid)?)
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_node(&self, uuid: InoxNodeUuid) -> Option<&ExtInoxNode<T>> {
        Some(self.get_internal_node(uuid)?.get())
    }

    pub fn get_node_mut(&mut self, uuid: InoxNodeUuid) -> Option<&mut ExtInoxNode<T>> {
        Some(self.get_internal_node_mut(uuid)?.get_mut())
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_parent(&self, uuid: InoxNodeUuid) -> Option<&ExtInoxNode<T>> {
        let node = self.get_internal_node(uuid)?;
        Some(self.arena.get(node.parent()?)?.get())
    }

    pub fn get_children_uuids(&self, uuid: InoxNodeUuid) -> Option<Vec<InoxNodeUuid>> {
        let node = self.get_internal_node(uuid)?;
        let node_id = self.arena.get_node_id(node)?;
        Some(
            node_id
                .children(&self.arena)
                .filter_map(|nid| self.arena.get(nid))
                .map(|nod| nod.get().uuid)
                .collect::<Vec<_>>(),
        )
    }

    pub fn ancestors(&self, uuid: InoxNodeUuid) -> indextree::Ancestors<ExtInoxNode<T>> {
        self.uuids[&uuid].ancestors(&self.arena)
    }

    fn rec_zsorts_from_root(&self, node: &ExtInoxNode<T>, zsort: f32) -> Vec<(InoxNodeUuid, f32)> {
        let node_state = node;
        let zsort = zsort + node_state.zsort;
        let mut vec = vec![(node_state.uuid, zsort)];

        // Skip composite children because they're a special case
        if !node.data.is_composite() {
            for child_uuid in self.get_children_uuids(node.uuid).unwrap_or_default() {
                if let Some(child) = self.get_node(child_uuid) {
                    vec.extend(self.rec_zsorts_from_root(child, zsort));
                }
            }
        }

        vec
    }

    fn sort_uuids_by_zsort(&self, mut uuid_zsorts: Vec<(InoxNodeUuid, f32)>) -> Vec<InoxNodeUuid> {
        uuid_zsorts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());
        uuid_zsorts.into_iter().map(|(uuid, _zsort)| uuid).collect()
    }

    fn sort_by_zsort(&self, node: &ExtInoxNode<T>) -> Vec<InoxNodeUuid> {
        let uuid_zsorts = self.rec_zsorts_from_root(node, 0.);
        self.sort_uuids_by_zsort(uuid_zsorts)
    }

    pub fn zsorted(&self) -> Vec<InoxNodeUuid> {
        let root = self.arena.get(self.root).unwrap().get();
        self.sort_by_zsort(root)
    }
}

fn rec_fmt<T>(
    indent: usize,
    f: &mut std::fmt::Formatter<'_>,
    node_id: NodeId,
    arena: &Arena<ExtInoxNode<T>>,
) -> std::fmt::Result {
    let Some(node) = arena.get(node_id) else {
        return Ok(());
    };

    let node = node.get();

    let type_name = node.node_type_name();
    #[cfg(feature = "owo")]
    let type_name = {
        use owo_colors::OwoColorize;
        type_name.magenta()
    };

    writeln!(f, "{}- [{}] {}", "  ".repeat(indent), type_name, node.name)?;
    for child in node_id.children(arena) {
        rec_fmt(indent + 1, f, child, arena)?;
    }

    Ok(())
}

impl<T> Display for ExtInoxNodeTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(root_node) = self.arena.get(self.root) else {
            return write!(f, "(empty)");
        };

        let root_node = root_node.get();

        let type_name = root_node.node_type_name();
        #[cfg(feature = "owo")]
        let type_name = {
            use owo_colors::OwoColorize;
            type_name.magenta()
        };

        writeln!(f, "- [{}] {}", type_name, root_node.name)?;
        for child in self.root.children(&self.arena) {
            rec_fmt(1, f, child, &self.arena)?;
        }

        Ok(())
    }
}
