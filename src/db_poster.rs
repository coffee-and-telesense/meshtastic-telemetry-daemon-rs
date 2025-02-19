use super::types::{Payload, Pkt};
use crate::entities::*;
use crate::types::Telem;
use airqualitymetrics::ActiveModel;
use anyhow::{Context, Result};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};

pub async fn update_metrics(
    db: &DatabaseConnection,
    packet: Pkt,
    fake_msg_id: Option<u32>,
    dep_loc: &String,
) -> Result<u32> {
    match packet {
        Pkt::Mesh(mp) => {
            if let Some(p) = mp.payload.clone() {
                match p {
                    Payload::TelemetryApp(t) => match t {
                        Telem::Environment(data) => {
                            match (environmentmetrics::ActiveModel {
                                msg_id: ActiveValue::Set(mp.id),
                                node_id: ActiveValue::Set(mp.from),
                                time: ActiveValue::Set(Utc::now().naive_utc()),
                                relative_humidity: ActiveValue::Set(data.relative_humidity),
                                tempurature: ActiveValue::Set(data.temperature),
                                barometric_pressure: ActiveValue::Set(data.barometric_pressure),
                                gas_resistance: ActiveValue::Set(data.gas_resistance),
                                iaq: ActiveValue::Set(data.iaq),
                                wind_direction: ActiveValue::Set(data.wind_direction),
                                wind_gust: ActiveValue::Set(data.wind_gust),
                                wind_speed: ActiveValue::Set(data.wind_speed),
                                wind_lull: ActiveValue::Set(data.wind_lull),
                                rainfall_1h: ActiveValue::Set(data.rainfall_1h),
                                rainfall_24h: ActiveValue::Set(data.rainfall_24h),
                            }
                            .insert(db)
                            .await
                            .with_context(|| "Failed to insert environment metrics row"))
                            {
                                Ok(_) => Ok(1),
                                Err(e) => {
                                    error!("{:#}", e);
                                    Ok(0)
                                }
                            }
                        }

                        Telem::AirQuality(data) => {
                            match (airqualitymetrics::ActiveModel {
                                msg_id: ActiveValue::Set(mp.id),
                                node_id: ActiveValue::Set(mp.from),
                                time: ActiveValue::Set(Utc::now().naive_utc()),
                                pm10standard: ActiveValue::Set(data.pm10_standard),
                                pm25standard: ActiveValue::Set(data.pm25_standard),
                                pm100standard: ActiveValue::Set(data.pm100_standard),
                                pm10environmental: ActiveValue::Set(data.pm10_environmental),
                                pm25environmental: ActiveValue::Set(data.pm25_environmental),
                                pm100environmental: ActiveValue::Set(data.pm100_environmental),
                                particles03um: ActiveValue::Set(data.particles_03um),
                                particles05um: ActiveValue::Set(data.particles_05um),
                                particles10um: ActiveValue::Set(data.particles_10um),
                                particles25um: ActiveValue::Set(data.particles_25um),
                                particles50um: ActiveValue::Set(data.particles_50um),
                                particles100um: ActiveValue::Set(data.particles_100um),
                                co2: ActiveValue::Set(data.co2),
                            }
                            .insert(db)
                            .await
                            .with_context(|| "Failed to insert air quality metrics row"))
                            {
                                Ok(_) => Ok(1),
                                Err(e) => {
                                    error!("{:#}", e);
                                    Ok(0)
                                }
                            }
                        }

                        Telem::Device(data) => {
                            match (devicemetrics::ActiveModel {
                                msg_id: ActiveValue::Set(mp.id),
                                node_id: ActiveValue::Set(mp.from),
                                time: ActiveValue::Set(Utc::now().naive_utc()),
                                battery_levels: ActiveValue::Set(data.battery_level),
                                voltage: ActiveValue::Set(data.voltage),
                                channelutil: ActiveValue::Set(data.channel_utilization),
                                airutil: ActiveValue::Set(data.air_util_tx),
                                latitude: ActiveValue::NotSet, //TODO: investigate default values
                                longitude: ActiveValue::NotSet,
                                longname: ActiveValue::NotSet,
                                shortname: ActiveValue::NotSet,
                                hwmodel: ActiveValue::NotSet,
                            }
                            .insert(db)
                            .await
                            .with_context(|| "Failed to insert device metrics row"))
                            {
                                Ok(_) => Ok(1),
                                Err(e) => {
                                    error!("{:#}", e);
                                    Ok(0)
                                }
                            }
                        }

                        Telem::Power(_data) => {
                            // Not sure what we want to do with these metrics
                            Ok(0)
                        }
                    },

                    Payload::NodeinfoApp(data) => {
                        // Only updates user information
                        match (nodeinfo::ActiveModel {
                            node_id: ActiveValue::Set(mp.from),
                            longname: ActiveValue::Set(data.long_name),
                            shortname: ActiveValue::Set(data.short_name),
                            hwmodel: ActiveValue::Set(data.hw_model),
                            deployment_location: ActiveValue::Set(dep_loc.to_string()),
                        }
                        .insert(db)
                        .await
                        .with_context(|| "Failed to insert node info row"))
                        {
                            Ok(_) => Ok(1),
                            Err(e) => {
                                error!("{:#}", e);
                                Ok(0)
                            }
                        }
                    }

                    Payload::PositionApp(data) => {
                        // Updates the position for a given node id that is included in the
                        // packet sent from the mesh
                        match (devicemetrics::ActiveModel {
                            msg_id: ActiveValue::Set(mp.id),
                            node_id: ActiveValue::Set(mp.from),
                            time: ActiveValue::Set(Utc::now().naive_utc()),
                            latitude: ActiveValue::Set(data.latitude_i),
                            longitude: ActiveValue::Set(data.longitude_i),
                            battery_levels: ActiveValue::NotSet, //TODO: investigate default values
                            voltage: ActiveValue::NotSet,
                            channelutil: ActiveValue::NotSet,
                            airutil: ActiveValue::NotSet,
                            longname: ActiveValue::NotSet,
                            shortname: ActiveValue::NotSet,
                            hwmodel: ActiveValue::NotSet,
                        }
                        .insert(db)
                        .await
                        .with_context(|| "Failed to insert device metrics row"))
                        {
                            Ok(_) => Ok(1),
                            Err(e) => {
                                error!("{:#}", e);
                                Ok(0)
                            }
                        }
                    }

                    _ => {
                        // Other payloads are unhandled, but there are some that may be of
                        // interest to us. Namely: TextMessageApp, RemoteHardwareApp (if we
                        // ever use the GPIO pins and module of the same name), RoutingApp
                        // (telemetry about routes discovered and failed routes),
                        // AdminMessageApp (this is usually config R/W), IPTunnelApp (this just
                        // routes IP packets through Meshtastic network, probably not
                        // interesting to us), PaxcounterApp (reports on BLE/WIFI devices
                        // seen), SerialApp (this is an interface for sending/receiving packets
                        // over a serial connection, probably useless to us), StoreForwardApp
                        // (this has some interesting data about history, stats, and heartbeats),
                        // RangeTestApp (probably not useful to us), TracerouteApp (seems to
                        // provide the same data as the RoutingApp but when users explicitly
                        // request traceroutes), NeighborinfoApp (adjacency matrix data and
                        // other stuff like last heard, might be good for us)
                        //
                        // Some of these may also be provided outside of Mesh packets, so we
                        // would need to handle them similar to NInfo below. Need to
                        // investigate this further, but I suspect it is nested in the
                        // ModuleConfig response over serial.
                        Ok(0)
                    }
                }
            } else {
                // No payload provided by the packet, just return 0 inserts to the db
                Ok(0)
            }
        }

        Pkt::NInfo(ni) => {
            let mut rv = 0;
            // This is the NodeInfo that is communicated directly over serial to our process
            // It does not have a MeshPkt (Meshtastic Packet from LoRa)
            let dm = devicemetrics::ActiveModel {
                msg_id: ActiveValue::Set(fake_msg_id.expect("Fake_msg_id not provided")),
                node_id: ActiveValue::Set(ni.num),
                time: ActiveValue::Set(Utc::now().naive_utc()),
                latitude: ActiveValue::Set(ni.position.as_ref().and_then(|pos| pos.latitude_i)),
                longitude: ActiveValue::Set(ni.position.as_ref().and_then(|pos| pos.longitude_i)),
                battery_levels: ActiveValue::Set(
                    ni.device_metrics.as_ref().and_then(|dm| dm.battery_level),
                ),
                voltage: ActiveValue::Set(ni.device_metrics.as_ref().and_then(|dm| dm.voltage)),
                channelutil: ActiveValue::Set(
                    ni.device_metrics
                        .as_ref()
                        .and_then(|dm| dm.channel_utilization),
                ),
                airutil: ActiveValue::Set(ni.device_metrics.as_ref().and_then(|dm| dm.air_util_tx)),
                longname: ActiveValue::Set(ni.user.as_ref().map(|u| u.long_name.clone())),
                shortname: ActiveValue::Set(ni.user.as_ref().map(|u| u.short_name.clone())),
                hwmodel: ActiveValue::Set(ni.user.as_ref().map(|u| u.hw_model)),
            }
            .insert(db)
            .await
            .with_context(|| "Failed to insert device metrics row");
            if dm.is_ok() {
                rv += 1;
            }
            // Now update the nodeinfo table if needed
            if let Some(user) = ni.user {
                let nm = nodeinfo::ActiveModel {
                    node_id: ActiveValue::Set(ni.num),
                    longname: ActiveValue::Set(user.long_name),
                    shortname: ActiveValue::Set(user.short_name),
                    hwmodel: ActiveValue::Set(user.hw_model),
                    deployment_location: ActiveValue::Set(dep_loc.to_string()),
                }
                .insert(db)
                .await
                .with_context(|| "Failed to insert device metrics row");
                if nm.is_ok() {
                    rv += 1;
                }
            }
            Ok(rv)
        }

        _ => {
            // Only other type implemented at this time is MyNodeInfo, which just provides our
            // node's ID number which could be used for the managing of local state if needed,
            // but we do not need it when making database updates for now
            Ok(0)
        }
    }
}
