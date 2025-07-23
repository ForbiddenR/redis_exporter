use anyhow::Result;
use prometheus::{Gauge, GaugeVec, Opts, core::Collector, proto::MetricFamily};

use crate::info::{Info, KeySpace, Mode};

const DEFAULT_LABEL: &[&str; 0] = &[];

macro_rules! new {
    (i, $name: literal, $help:literal, $fn:expr$(,)?) => {
        InfoMetric::new(to_gauge_vec!($name, $help), $fn)
    };
    (k, $name: literal, $help:literal, $fn:expr$(,)?) => {
        KeyspaceMetric::new(to_gauge_vec!($name, $help), $fn)
    };
}

macro_rules! to_gauge_vec {
    ($name: literal, $help:literal) => {
        GaugeVec::new(Opts::new($name, $help), DEFAULT_LABEL)
            .expect("failed to create gauge vector")
    };
}

macro_rules! initializing {
    ($name:ident, $t:ty) => {
        #[derive(Debug)]
        struct $name {
            gauge_vec: GaugeVec,
            value_fn: fn(&$t) -> f64,
        }

        impl $name {
            fn new(gauge_vec: GaugeVec, value_fn: fn(&$t) -> f64) -> Self {
                Self {
                    gauge_vec,
                    value_fn,
                }
            }
        }
    };
}

initializing!(InfoMetric, Info);
initializing!(KeyspaceMetric, KeySpace);

#[derive(Debug)]
pub struct Exporter {
    client: redis::Client,
    up: Gauge,
    info_metrics: Vec<InfoMetric>,
    keyspace_metrics: Vec<KeyspaceMetric>,
}

impl Exporter {
    pub fn new(client: redis::Client) -> Self {
        let up = Gauge::new("node_status", "The status of current node").unwrap();
        let info_metrics = vec![
            new!(
                i,
                "connected_clients",
                "Total connections connect to redis",
                |i| i.connected_clents as f64,
            ),
            new!(
                i,
                "max_clients",
                "Max allowed connection number",
                |i| i.maxclients as f64,
            ),
            new!(
                i,
                "role_master",
                "Current node is master",
                |i| (i.role == "master") as u8 as f64
            ),
        ];

        let keyspace_metrics = vec![
            new!(
                k,
                "dbsize",
                "Total key number of current node",
                |k| k.key_number as f64
            ),
            new!(
                k,
                "avg_ttl",
                "Total avg_ttl of all db in this node",
                |k| k.avg_ttl as f64
            ),
        ];

        Self {
            client,
            up,
            info_metrics,
            keyspace_metrics,
        }
    }

    // get a reference of the redis client
    fn get_client(&self) -> &redis::Client {
        &self.client
    }

    pub async fn collect(&self, mode: &Option<Mode>) -> Vec<MetricFamily> {
        match self.get_info(mode).await {
            Ok(m) => {
                self.up.set(1.0);
                m
            }
            Err(e) => {
                eprintln!("Failed to collect metrics: {e}");
                self.up.set(0.0);
                return self.up.collect();
            }
        }
        .into_iter()
        .filter(|f| !f.get_metric().is_empty())
        .chain(self.up.collect())
        .collect()
    }

    async fn get_info(&self, mode: &Option<Mode>) -> Result<Vec<MetricFamily>> {
        macro_rules! clear {
            ($($metric:ident)+) => {
                $(
                    self.$metric.iter().for_each(|f|f.gauge_vec.reset());
                )+
            };
        }
        clear!(info_metrics keyspace_metrics);

        let mut conn = self.get_client().get_multiplexed_tokio_connection().await?;
        // fetch redis info message
        let info_message = redis::cmd("info").query_async::<String>(&mut conn).await?;

        // split it by "\r\n" and collect them as a vec
        let lines = info_message
            .split("\r\n")
            .filter(|p| !p.is_empty())
            .collect::<Vec<&str>>();

        let info = Info::parse(&lines);

        self.info_metrics.iter().for_each(|f| {
            f.gauge_vec
                .with_label_values(DEFAULT_LABEL)
                .set((f.value_fn)(&info))
        });

        if mode.as_ref().unwrap_or(info.mode()).is_standard() {
            for keyspace in &info.keyspaces {
                self.keyspace_metrics.iter().for_each(|f| {
                    f.gauge_vec
                        .with_label_values(DEFAULT_LABEL)
                        .add((f.value_fn)(keyspace));
                });
            }
        }

        Ok(self
            .info_metrics
            .iter()
            .flat_map(|f| f.gauge_vec.collect())
            .chain(
                self.keyspace_metrics
                    .iter()
                    .flat_map(|f| f.gauge_vec.collect()),
            )
            .collect())
    }
}
