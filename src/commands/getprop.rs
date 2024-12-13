use super::_common;

pub fn run(host: &str, port: &str, command: &str) {
    let result = _common::get_prop(host, port, command);
    println!("{}", result)
}
