use colored_json::ToColoredJson;
use serde::Serialize;
use std::error::Error;

pub fn print_colored_json<T: Serialize>(data: &T) -> Result<(), Box<dyn Error>> {
    let json_str = serde_json::to_string_pretty(data)?;
    println!("{}", json_str.to_colored_json_auto()?);
    Ok(())
}
