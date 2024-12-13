use super::lib;

pub async fn run(host: &str, port: &str, propnames: &[String]) {
    let result = lib::get_props_parallel(host, port, &propnames).await;
    println!("{:?}", result)
}
