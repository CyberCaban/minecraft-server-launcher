pub const TEMPLATE_NAME: &str = "Forge 1.20.1";

pub fn render_template(project: &str, port: u16, memory_gb: u32, rcon_password: &str) -> String {
    format!(
        r#"services:
  minecraft:
    image: itzg/minecraft-server:latest
    container_name: "{project}"
    tty: true
    stdin_open: true
    ports:
      - "{port}:25565"
    environment:
      EULA: "true"
      ENABLE_RCON: "true"
      RCON_PASSWORD: "{rcon_password}"
      GUI: "false"
      MEMORY: "{memory_gb}G"
      VERSION: 1.20.1
      SKIP_GENERIC_PACK_UPDATE_CHECK: "true"
    volumes:
      - ./data:/data
      - ./backups:/backups
"#
    )
}

pub fn detect_rcon(content: &str) -> bool {
    let Ok(root) = serde_yaml::from_str::<serde_yaml::Value>(content) else {
        return false;
    };
    let Some(services) = root.get("services").and_then(|s| s.as_mapping()) else {
        return false;
    };
    for (_, service) in services {
        let Some(env) = service.get("environment") else {
            continue;
        };
        let raw = if let Some(map) = env.as_mapping() {
            map.get("ENABLE_RCON")
                .or_else(|| map.get("enable_rcon"))
                .and_then(scalar_to_string)
        } else if let Some(list) = env.as_sequence() {
            list.iter().find_map(|v| {
                let s = v.as_str()?;
                let mut parts = s.splitn(2, '=');
                let key = parts.next()?;
                let val = parts.next()?;
                if key.trim().eq_ignore_ascii_case("ENABLE_RCON") {
                    Some(val.trim().to_string())
                } else {
                    None
                }
            })
        } else {
            None
        };
        if let Some(raw) = raw {
            if is_truthy(&raw) {
                return true;
            }
        }
    }
    false
}

fn scalar_to_string(v: &serde_yaml::Value) -> Option<String> {
    if let Some(b) = v.as_bool() {
        return Some(b.to_string());
    }
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    if let Some(i) = v.as_i64() {
        return Some(i.to_string());
    }
    None
}

fn is_truthy(s: &str) -> bool {
    matches!(s.trim().to_lowercase().as_str(), "true" | "yes" | "1" | "on")
}
