use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref ADB_COMMANDS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        // Device selection commands
        m.insert("ANY_DEVICE", "host:tport:any");
        m.insert("SELECT_DEVICE", "host:tport:serial:{}"); // Requires device ID formatting

        // Shell commands
        m.insert("SHELL", "shell:{}"); // Requires command formatting
        m.insert("SHELL_V2", "shell,v2,TERM=xterm-256color,raw:{}"); // Requires command formatting

        // Property commands
        m.insert("GETPROP", "shell:getprop");
        m.insert("GETPROP_SINGLE", "shell:getprop {}"); // Requires property name formatting

        // Sync commands
        m.insert("SYNC", "sync:");
        m.insert("PUSH", "sync:{}"); // Requires path formatting
        m.insert("PULL", "sync:{}"); // Requires path formatting

        // Server commands
        m.insert("VERSION", "host:version");
        m.insert("DEVICES", "host:devices");
        m.insert("KILL", "host:kill");
        m.insert("TRACK_DEVICES", "host:track-devices");

        // Transport commands
        m.insert("TRANSPORT", "host:transport:{}"); // Requires device ID formatting

        m
    };
}
pub fn format_command(cmd: &str, args: &[&str]) -> String {
    match ADB_COMMANDS.get(cmd.to_uppercase().as_str()) {
        Some(&template) => {
            if args.is_empty() {
                template.to_string()
            } else {
                template.replace("{}", &args.join(" "))
            }
        }
        None => panic!("Unknown ADB command: {}", cmd),
    }
}
