use super::lib;

pub fn run(host: &str, port: &str) {
    let model = lib::run_command(host, port, "getprop ro.product.product.model", None);
    println!("{}", model)
}
