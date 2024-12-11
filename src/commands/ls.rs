use super::_common;

pub fn run(host: &str, port: &str, long: bool) {
    let message = if long {
        "000ehost:devices-l"
    } else {
        "000chost:devices"
    };
    match _common::send_and_receive(&host, &port, message) {
        Ok(responses) => {
            let formatted_output = format(&responses);

            display(formatted_output)
        }
        Err(e) => {
            eprintln!("Error: {}", e)
        }
    }
}

fn format(responses: &[String]) -> Vec<&str> {
    responses.iter().map(|r| r.trim()).collect::<Vec<&str>>()
}

fn display(string: Vec<&str>) {
    println!("{}", string.join("\n"))
}
