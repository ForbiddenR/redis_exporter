#[derive(Debug, Default)]
pub struct Info {
    pub role: String,
    pub connected_clents: i64,
    pub maxclients: i64,
    pub used_memory: f64,
    pub used_cpu_sys: f64,
    pub used_cpu_user: f64,
    pub keyspaces: Vec<KeySpace>,
}

#[derive(Debug, Default)]
pub struct KeySpace {
    pub db: String,
    pub key_number: i64,
    pub avg_ttl: i64,
}

impl Info {
    pub fn parse(info_datas: &Vec<&str>) -> Self {
        let mut info = Self::default();
        let mut field_class = "";

        for info_data in info_datas {
            if info_data.starts_with("# ") && info_data.len() > 2 {
                field_class = &info_data[2..];
                continue;
            }
            if let Some((key, value)) = info_data.split_once(':') {
                match field_class {
                    "Keyspace" => {
                        if key.starts_with("db") {
                            macro_rules! parse_key_space {
                                ($v:ident, $key:literal) => {
                                    $v.iter()
                                        .find(|&&p| p.starts_with($key))
                                        .and_then(|p| p.strip_prefix($key))
                                        .and_then(|s| s.parse().ok())
                                        .unwrap_or(0)
                                };
                            }

                            let db = key.to_string();
                            let parts: Vec<&str> = value.split(',').collect();
                            let key_number = parse_key_space!(parts, "keys=");
                            let avg_ttl = parse_key_space!(parts, "avg_ttl=");
                            info.keyspaces.push(KeySpace {
                                db,
                                key_number,
                                avg_ttl,
                            });
                            continue;
                        }
                    }
                    _ => {}
                }
                match key {
                    "role" => info.role = value.to_string(),
                    "connected_clients" => info.connected_clents = value.parse().unwrap_or(0),
                    "maxclients" => info.maxclients = value.parse().unwrap_or(0),
                    "used_memory" => info.used_memory = value.parse().unwrap_or(0.0),
                    "used_cpu_sys" => info.used_cpu_sys = value.parse().unwrap_or(0.0),
                    "used_cpu_user" => info.used_cpu_user = value.parse().unwrap_or(0.0),
                    _ => {}
                }
            }
        }
        info
    }

    pub fn is_master(&self) -> bool {
        &self.role == "master"
    }
}
