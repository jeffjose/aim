use crate::library::adb;

pub fn run(host: &str, port: &str, command: &str) {
    let result = adb::run_command(host, port, command, None);
    println!("{}", result)
}
