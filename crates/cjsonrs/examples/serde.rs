use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub name: String,
    pub age: u32,
}

fn main() {
    let sample = Sample {
        name: "John".to_string(),
        age: 30,
    };

    let cjson = cjsonrs::serde::to_cjson(&sample).unwrap();
    println!("CJson constructed from Rust struct! {}", cjson);

    let sample: Sample = cjsonrs::serde::from_cjson(&cjson).unwrap();
    println!("Rust struct constructed from CJson! {:?}", sample);
}
