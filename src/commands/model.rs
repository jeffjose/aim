use super::_common;

pub fn run(host: &str, port: &str) {
    let model = _common::run_command(host, port, "getprop ro.product.product.model");
    println!("{}", model)
}
