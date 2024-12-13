use crate::library::adb;
pub fn run(host: &str, port: &str) {
    let model = adb::run_command(host, port, "getprop ro.product.product.model", None);
    println!("{}", model)
}
