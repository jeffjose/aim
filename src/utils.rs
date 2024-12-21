use colored::*;
use serde::Serialize;
use std::error::Error;

pub fn print_colored_json<T: Serialize>(data: &T) -> Result<(), Box<dyn Error>> {
    let json_str = serde_json::to_string_pretty(data)?;

    // Split the JSON string into lines and color each line appropriately
    let colored_lines: Vec<String> = json_str
        .lines()
        .map(|line| {
            if line.contains(":") {
                // This is a key-value pair
                let parts: Vec<&str> = line.splitn(2, ":").collect();
                let key = parts[0].trim_matches(|c| c == '"' || c == ' ' || c == '{' || c == '}');
                let value = parts.get(1).map_or("", |v| *v);

                format!(
                    "{}{}: {}",
                    " ".repeat(line.find(|c| c != ' ').unwrap_or(0)),
                    key.cyan(),
                    value
                )
            } else {
                // This is a structural line (brackets, commas, etc)
                line.to_string()
            }
        })
        .collect();

    println!("{}", colored_lines.join("\n"));
    Ok(())
}
