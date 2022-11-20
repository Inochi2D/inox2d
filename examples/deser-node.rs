use inox2d::nodes::node_tree::NodeTree;

const JSON_PAYLOAD: &str = include_str!("../target/one-node.json");

fn main() {
    let node: NodeTree = serde_json::from_str(JSON_PAYLOAD).unwrap();
    println!("deserialized = {:#?}", node);

    let serialized = serde_json::to_string(&node).unwrap();
    println!("serialized = {}", serialized);
}