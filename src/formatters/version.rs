pub fn format(responses: &[String]) -> String {
    if let Some(first_response) = responses.first() {
        format!("Server Version: {}", first_response.trim())
    } else {
        "No version information received.".to_string()
    }
}
