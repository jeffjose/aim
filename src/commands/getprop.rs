use super::_common;

pub fn run(host: &str, port: &str, propname: &str) {
    let result = _common::get_prop(host, port, propname);
    println!("{}", result)
}
