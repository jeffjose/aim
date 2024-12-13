use super::lib;

pub fn run(host: &str, port: &str) {
    let messages = vec!["host:version"];
    match lib::send_and_receive(&host, &port, messages) {
        Ok(responses) => {
            let formatted_output = format(&responses);
            println!("{}", formatted_output)
        }
        Err(e) => {
            eprintln!("Error: {}", e)
        }
    }
}

fn format(responses: &[String]) -> String {
    responses
        .iter()
        .map(|r| r.trim())
        .collect::<Vec<&str>>()
        .join("\n")
}
