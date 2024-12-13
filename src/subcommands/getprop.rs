use crate::library::adb;

pub fn run(host: &str, port: &str, propname: &str) {
    let result = adb::get_prop(host, port, propname);
    println!("{}", result)
}
