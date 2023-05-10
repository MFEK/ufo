use serde_json::Value;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub name: String,
    pub ascender: i32,
    pub descender: i32,
}

pub fn parse_metadata(output: &str) -> Result<Metadata, Box<dyn Error>> {
    let mut lines = output.lines();

    let name = lines.next().ok_or("Missing name")?;
    let name: Value = serde_json::from_str(name)?;
    let name = name.as_str().unwrap_or("").to_string();

    let ascender = lines.next().ok_or("Missing ascender")?.parse::<i32>()?;
    let descender = lines.next().ok_or("Missing descender")?.parse::<i32>()?;

    Ok(Metadata {
        name,
        ascender,
        descender,
    })
}
