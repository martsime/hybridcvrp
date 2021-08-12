use std::fs::File;
use std::io::BufReader;

use serde_json::Value;

pub fn load_json_file(filepath: &str) -> Value {
    let file = File::open(filepath).expect(&format!("Cannot open file {}", filepath));
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect(&format!("Failed to read file {}", filepath))
}
