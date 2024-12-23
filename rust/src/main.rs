mod types;

fn main() {
    let json_value = types::Json::Object(std::collections::HashMap::new());
    println!("{:?}", json_value);
}