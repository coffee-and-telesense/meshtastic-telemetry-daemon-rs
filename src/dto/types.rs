use crate::{
    dto::models::{
        Airqualitymetric, Devicemetric, Environmentmetric, Errormetric, Localstat, Neighborinfo,
        Nodeinfo,
    },
    util::config::DEPLOYMENT_LOCATION,
};
use chrono::{NaiveDateTime, Utc};
use meshtastic::protobufs::{
    AirQualityMetrics, DeviceMetrics, EnvironmentMetrics, ErrorMetrics, LocalStats, Neighbor,
    NeighborInfo, NodeInfo,
};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::types::Oid;

/// Create a timestamp from a given epoch `u32`
///
/// # Arguments
/// * `epoch` - u32 epoch value
///
/// # Returns
/// * `NaiveDateTime` - from a u32 epoch value or the `Utc::now()` value of the daemon if the epoch value is
///   zero
///
/// # Panics
/// * If the epoch is more than 250,000 year from the common era or if the nanosecs is > 2
#[inline]
pub(crate) fn timestamp(epoch: u32) -> NaiveDateTime {
    if epoch > 1_735_689_600 {
        // Not recording timestamps earlier than 01/01/2025 12:00:00
        #[allow(deprecated)]
        NaiveDateTime::from_timestamp(epoch.into(), 0)
    } else {
        Utc::now().naive_utc()
    }
}

pub(crate) trait ToRow<R> {
    /// Convert a given struct into a `R` row struct
    ///
    /// # Arguments
    ///
    /// # Returns
    /// * `R` - the `Row` struct
    fn to_row(self, msg_id: Oid, node_id: Oid, time: NaiveDateTime) -> R
    where
        Self: std::marker::Sized;
}

impl ToRow<Airqualitymetric> for AirQualityMetrics {
    fn to_row(self, msg_id: Oid, node_id: Oid, time: NaiveDateTime) -> Airqualitymetric
    where
        Self: std::marker::Sized,
    {
        Airqualitymetric::new(
            msg_id,
            node_id,
            time,
            self.pm10_standard.map(Oid),
            self.pm25_standard.map(Oid),
            self.pm100_standard.map(Oid),
            self.pm10_environmental.map(Oid),
            self.pm25_environmental.map(Oid),
            self.pm100_environmental.map(Oid),
            self.particles_03um.map(Oid),
            self.particles_05um.map(Oid),
            self.particles_10um.map(Oid),
            self.particles_25um.map(Oid),
            self.particles_50um.map(Oid),
            self.particles_100um.map(Oid),
            self.co2.map(Oid),
            self.sensor,
        )
    }
}

impl ToRow<Devicemetric> for DeviceMetrics {
    fn to_row(self, msg_id: Oid, node_id: Oid, time: NaiveDateTime) -> Devicemetric
    where
        Self: std::marker::Sized,
    {
        Devicemetric::new(
            msg_id,
            node_id,
            time,
            self.battery_level.map(Oid),
            self.voltage,
            self.channel_utilization,
            self.air_util_tx,
            None,
            None,
            None,
            None,
            None,
        )
    }
}

impl ToRow<Devicemetric> for NodeInfo {
    fn to_row(self, msg_id: Oid, node_id: Oid, time: NaiveDateTime) -> Devicemetric
    where
        Self: std::marker::Sized,
    {
        let user = self.user;
        let dm = self.device_metrics;
        let loc = self.position;
        Devicemetric::new(
            msg_id,
            node_id,
            time,
            dm.and_then(|d| d.battery_level.map(Oid)),
            dm.and_then(|d| d.voltage),
            dm.and_then(|d| d.channel_utilization),
            dm.and_then(|d| d.air_util_tx),
            loc.and_then(|l| l.latitude_i),
            loc.and_then(|l| l.longitude_i),
            user.as_ref().map(|u| u.long_name.clone()),
            user.as_ref().map(|u| u.short_name.clone()),
            user.as_ref().map(|u| u.hw_model),
        )
    }
}

impl ToRow<Environmentmetric> for EnvironmentMetrics {
    fn to_row(self, msg_id: Oid, node_id: Oid, time: NaiveDateTime) -> Environmentmetric
    where
        Self: std::marker::Sized,
    {
        Environmentmetric::new(
            msg_id,
            node_id,
            time,
            self.temperature,
            self.relative_humidity,
            self.barometric_pressure,
            self.gas_resistance,
            self.iaq.map(Oid),
            self.wind_direction.map(Oid),
            self.wind_speed,
            self.wind_gust,
            self.wind_lull,
            self.rainfall_1h,
            self.rainfall_24h,
            self.sensor,
        )
    }
}

#[derive(Serialize)]
struct Errors {
    no_routes: Option<Oid>,
    naks: Option<Oid>,
    timeouts: Option<Oid>,
    max_retransmits: Option<Oid>,
    no_channels: Option<Oid>,
    too_large: Option<Oid>,
}

impl ToRow<Errormetric> for ErrorMetrics {
    fn to_row(self, msg_id: Oid, node_id: Oid, time: NaiveDateTime) -> Errormetric
    where
        Self: std::marker::Sized,
    {
        Errormetric::new(
            msg_id,
            node_id,
            time,
            self.collision_rate,
            self.node_reach,
            self.num_nodes.map(Oid),
            self.usefulness,
            self.avg_delay.map(Oid),
            self.period.map(Oid),
            Some(json!(&Errors {
                no_routes: self.noroute.map(Oid),
                naks: self.naks.map(Oid),
                timeouts: self.timeouts.map(Oid),
                max_retransmits: self.max_retransmit.map(Oid),
                no_channels: self.no_channel.map(Oid),
                too_large: self.too_large.map(Oid)
            })),
        )
    }
}

impl ToRow<Localstat> for LocalStats {
    fn to_row(self, msg_id: Oid, node_id: Oid, time: NaiveDateTime) -> Localstat
    where
        Self: std::marker::Sized,
    {
        Localstat::new(
            msg_id,
            node_id,
            time,
            Some(Oid(self.uptime_seconds)),
            Some(self.channel_utilization),
            Some(self.air_util_tx),
            Some(Oid(self.num_packets_tx)),
            Some(Oid(self.num_packets_rx)),
            Some(Oid(self.num_packets_rx_bad)),
            Some(Oid(self.num_online_nodes)),
            Some(Oid(self.num_total_nodes)),
            Some(Oid(self.num_rx_dupe)),
            Some(Oid(self.num_tx_relay)),
            Some(Oid(self.num_tx_relay_canceled)),
        )
    }
}

impl ToRow<Neighborinfo> for NeighborInfo {
    fn to_row(self, msg_id: Oid, node_id: Oid, time: NaiveDateTime) -> Neighborinfo
    where
        Self: std::marker::Sized,
    {
        Neighborinfo::new(
            msg_id,
            node_id,
            time,
            Some(Oid(self.last_sent_by_id)),
            Some(Oid(self.node_broadcast_interval_secs)),
            Some(
                self.neighbors
                    .into_iter()
                    .map(|n| {
                        json!(&Neighbor {
                            node_id: n.node_id,
                            snr: n.snr,
                            last_rx_time: n.last_rx_time,
                            node_broadcast_interval_secs: n.node_broadcast_interval_secs,
                            num_packets_rx: n.num_packets_rx,
                            rssi: n.rssi,
                        })
                    })
                    .collect::<Vec<Value>>(),
            ),
        )
    }
}

impl ToRow<Nodeinfo> for &NodeInfo {
    fn to_row(self, _msg_id: Oid, node_id: Oid, _time: NaiveDateTime) -> Nodeinfo
    where
        Self: std::marker::Sized,
    {
        let user = self.user.clone().expect("Unable to get User from NodeInfo");
        let deployment_location = DEPLOYMENT_LOCATION
            .clone()
            .into_inner()
            .expect("Unable to get DEPLOYMENT_LOCATION constant for Nodeinfo row model creation");
        Nodeinfo::new(
            node_id,
            user.long_name,
            user.short_name,
            user.hw_model,
            deployment_location,
        )
    }
}
