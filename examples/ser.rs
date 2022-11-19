use inox2d::node::NodeId;

fn main() {
    let node_id = unsafe { NodeId::new(727) };

    let serialized = serde_json::to_string(&node_id).unwrap();
    println!("serialized = {}", serialized);

    let deserialized: NodeId = serde_json::from_str(&serialized).unwrap();
    println!("deserialized = {:?}", deserialized);
}