use inox2d::puppet::Puppet;

const JSON_PAYLOAD: &str = include_str!("../target/payload.json");

fn main() {
    let puppet: Puppet = serde_json::from_str(JSON_PAYLOAD).unwrap();
    println!("deserialized = {:#?}", puppet);

    let serialized = serde_json::to_string(&puppet).unwrap();
    println!("{}", serialized);
}