use super::lib;

pub fn run(host: &str, port: &str, propname: &str) {
    let result = lib::get_prop(host, port, propname);
    println!("{}", result)
}
