use super::lib;

pub fn run(host: &str, port: &str, command: &str) {
    let result = lib::run_command(host, port, command);
    println!("{}", result)
}
