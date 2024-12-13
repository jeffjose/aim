use super::_common;

pub async fn run(host: &str, port: &str, propnames: &[String]) {
    let result = _common::get_props_parallel(host, port, &propnames).await;
    println!("{:?}", result)
}
