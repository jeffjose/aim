use super::_common;

pub fn run(host: &str, port: &str, command: &str) {
    let result = _common::run_command(host, port, command);
    println!("{}", result)
}