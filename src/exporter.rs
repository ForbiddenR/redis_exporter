use std::collections::HashMap;

use prometheus::{Gauge, core::Collector};

use crate::error::Result;

#[derive(Debug)]
pub struct Exporter {
    client: redis::Client,
    up: Gauge,
    gauges: HashMap<String, Gauge>,
}

macro_rules! to_gauge {
    ($key: literal, $name: literal, $help:literal) => {
        ($key.to_string(), Gauge::new($name, $help).unwrap())
    };
}

impl Exporter {
    pub fn new(client: redis::Client) -> Self {
        let up = Gauge::new("node_status", "The status of current node").unwrap();
        let gauges = HashMap::from([
            to_gauge!(
                "connected_clients",
                "connected_clients",
                "Total connections connect to redis"
            ),
            to_gauge!("maxclients", "max_clients", "Max allowed connection number"),
            to_gauge!("used_memory", "used_memory", "Used memory in bytes"),
            to_gauge!("used_cpu_sys", "used_cpu_sys", "Used cpu in system"),
            to_gauge!("used_cpu_user", "used_cpu_user", "Used cpu in user"),
            to_gauge!("role", "role_master", "Current node is master"),
            to_gauge!("dbsize", "dbsize", "Total key number of current node"),
            to_gauge!("ttl", "avg_ttl", "Total avg_ttl of all db in this node"),
        ]);

        Self { client, up, gauges }
    }

    pub fn get_client(&self) -> &redis::Client {
        &self.client
    }

    pub async fn collect(&mut self) -> Result<Vec<prometheus::proto::MetricFamily>> {
        let mut metrics = vec![];
        self.up.set(match self.get_info().await {
            Ok(f) if f == 0.0 => f,
            Ok(f) => {
                self.gauges
                    .iter()
                    .for_each(|(_, g)| metrics.extend(g.collect()));
                f
            }
            Err(_) => 0.0,
        });
        metrics.extend(self.up.collect());
        Ok(metrics)
    }

    fn set_keyspace_metric(&self, key: &str, value: &str) -> Result<(f64, f64)> {
        let mut db_size = 0.0;
        let mut avg_ttl = 0.0;
        if key.starts_with("db") {
            for (k, v) in &split(value) {
                match k.as_str() {
                    "keys" => db_size = v.parse()?,
                    "avg_ttl" => avg_ttl = v.parse()?,
                    _ => {}
                }
            }
        }
        Ok((db_size, avg_ttl))
    }

    async fn get_info(&self) -> Result<f64> {
        if let Ok(mut conn) = self.get_client().get_multiplexed_tokio_connection().await {
            let info_message = redis::cmd("info").query_async::<String>(&mut conn).await?;

            let lines = info_message
                .split("\r\n")
                .filter(|p| !p.is_empty())
                .collect::<Vec<&str>>();

            let mut field_class = "";
            let mut role = "";
            let mut total_keys = 0.0;
            let mut total_avg_ttl = 0.0;
            for result in lines {
                if result.len() > 0 && result.starts_with("# ") {
                    field_class = &result[2..];
                    continue;
                }

                let (key, value) = match result.split_once(":") {
                    Some(v) => v,
                    None => continue,
                };

                if key.is_role() {
                    role = value;
                    continue;
                }

                match field_class {
                    "Keyspace" => {
                        if role.is_master()
                            && let Ok((keys, ttl)) = self.set_keyspace_metric(key, value)
                        {
                            total_keys += keys;
                            total_avg_ttl += ttl;
                        }
                        continue;
                    }
                    _ => {}
                }

                self.gauges
                    .get(key)
                    .map(|g| parse_and_set(value, |f| g.set(f)));
            }

            self.gauges.get("dbsize").map(|g| g.set(total_keys));
            self.gauges.get("ttl").map(|g| g.set(total_avg_ttl));

            self.gauges
                .get("role")
                .map(|g| g.set(if role.is_master() { 1.0 } else { 0.0 }));

            Ok(1.0)
        } else {
            Ok(0.0)
        }
    }
}

fn parse_and_set<F>(v: &str, f: F)
where
    F: Fn(f64),
{
    match v {
        "ok" | "true" => f(1.0),
        "err" | "fail" | "false" => f(2.0),
        other => {
            if let Ok(d) = other.parse() {
                f(d)
            }
        }
    };
}

fn split(pairs: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in pairs.split(",").collect::<Vec<&str>>() {
        if let Some((k, v)) = pair.split_once("=") {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}

trait Is {
    fn is_master(&self) -> bool;
    fn is_role(&self) -> bool;
}

impl Is for &str {
    fn is_master(&self) -> bool {
        *self == "master"
    }

    fn is_role(&self) -> bool {
        *self == "role"
    }
}
