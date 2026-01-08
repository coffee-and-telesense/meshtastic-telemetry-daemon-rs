use crate::{
    dto::{
        models::{
            Airqualitymetric, Devicemetric, Environmentmetric, Errormetric, Localstat,
            Neighborinfo, Nodeinfo,
        },
        types::{DbOps, ToRow, timestamp},
    },
    util::{log::log_msg, state::GatewayState},
};
#[cfg(feature = "trace")]
use meshtastic::protobufs::{
    AdminMessage, Compressed, HardwareMessage, MapReport, Paxcount, PowerStressMessage,
    RouteDiscovery, Routing, StoreAndForward, TakPacket, Waypoint,
};
use meshtastic::{
    Message,
    protobufs::{
        FromRadio, MeshPacket, NeighborInfo, NodeInfo, PortNum, Position, Telemetry, from_radio,
        mesh_packet, telemetry::Variant,
    },
};
use sqlx::{Pool, Postgres, postgres::types::Oid};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

/// Process Packets
///
/// Match packet types based on payloads or origin on mesh or serial, then convert them to our
/// local types to pass along to the database handler. This could probably be simplified and I
/// should do that sometime. I should also make it much shorter because it is way too long
///
/// Shout-out to <https://github.com/PeterGrace/meshtui> for some of the code structure here
///
/// # Arguments
/// * `pkt` - A `FromRadio` reference that is read on the serial connection to a Meshtastic node
/// * `state` - The `GatewayState` with the various concurrency locks
///
/// # Returns
/// * An optional `Pkt`, our local types for packet handling
///
/// # Panics
/// This function will panic if it fails to acquire a lock on the `GatewayState`
#[allow(clippy::too_many_lines)] // most of these lines are just logging calls
pub async fn process_packet(
    pkt: &FromRadio,
    state: &Arc<Mutex<GatewayState<'_>>>,
    pool: &Pool<Postgres>,
) {
    if let Some(pv) = &pkt.payload_variant {
        match pv {
            from_radio::PayloadVariant::Packet(mesh_packet) => {
                decode_payload(mesh_packet, state, pool).await;
            }
            from_radio::PayloadVariant::NodeInfo(node_info) => {
                // none of the arguments are used, so do dummy args
                let row: Nodeinfo = node_info.to_row(Oid(0), Oid(node_info.num), timestamp(0));
                match row.insert(pool).await {
                    Ok(_) => log_msg("Inserted 1 row into NodeInfo table", log::Level::Info),
                    Err(_) => {
                        // Try updating the row
                        match row.update(pool).await {
                            Ok(_) => log_msg("Updated 1 row in NodeInfo table", log::Level::Info),
                            Err(e) => log_msg(format!("{e}").as_str(), log::Level::Error),
                        }
                    }
                }
            }
            #[cfg(not(feature = "trace"))]
            _ => (),
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::MyInfo(my_node_info) => {
                log_msg(
                    format!("Received MyInfo packet: {my_node_info:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::Config(config) => {
                log_msg(
                    format!("Received config packet: {config:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::LogRecord(log_record) => {
                log_msg(
                    format!("Received log_record packet: {log_record:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::ConfigCompleteId(id) => {
                log_msg(
                    format!("Received config {id} complete packet over serial").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::Rebooted(rbt) => {
                log_msg(
                    format!("Received rebooted packet: {rbt}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::ModuleConfig(module_config) => {
                log_msg(
                    format!("Received module_config packet: {module_config:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::Channel(channel) => {
                log_msg(
                    format!("Received channel packet: {channel:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::QueueStatus(queue_status) => {
                log_msg(
                    format!("Received queue_status packet: {queue_status:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::XmodemPacket(xmodem) => {
                log_msg(
                    format!("Received xmodem packet: {xmodem:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::Metadata(device_metadata) => {
                log_msg(
                    format!("Received device_metadata packet: {device_metadata:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::MqttClientProxyMessage(mqtt_client_proxy_message) => {
                log_msg(
                    format!(
                        "Received mqtt_client_proxy_message packet: {mqtt_client_proxy_message:?}"
                    )
                    .as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::FileInfo(file_info) => {
                log_msg(
                    format!("Received file_info packet: {file_info:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::ClientNotification(client_notification) => {
                log_msg(
                    format!("Received client_notification packet: {client_notification:?}")
                        .as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::DeviceuiConfig(device_ui_config) => {
                log_msg(
                    format!("Received device_ui_config packet: {device_ui_config:?}").as_str(),
                    log::Level::Info,
                );
            }
        }
    }
}

/// Trace logging decoded payloads
///
/// # Arguments
/// * `ptype` - `&str` of the payload type name
/// * `payload` - `P` generic that implements the `Debug` trait
///
/// # Returns
/// None
#[cfg(feature = "trace")]
#[inline]
fn decode_and_trace<P: Debug>(ptype: &str, payload: P) {
    log_msg(
        format!("Received {ptype} packet: {payload:?}").as_str(),
        log::Level::Info,
    );
}

/// Decode payloads
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference that is read on the serial connection to a Meshtastic node
/// * `state` - The `GatewayState` with the various concurrency locks
///
/// # Returns
/// * An optional `Pkt`, our local types for packet handling
///
/// # Panics
/// This function will panic if it fails to acquire a lock on the `GatewayState`
#[allow(clippy::too_many_lines)] // most of these lines are just logging calls
async fn decode_payload(
    pkt: &MeshPacket,
    state: &Arc<Mutex<GatewayState<'_>>>,
    pool: &Pool<Postgres>,
) {
    // Check if the packet is on the telemetry channel before decoding a payload
    if pkt.channel == 0
        && let Some(payload) = &pkt.payload_variant
    {
        match payload {
            mesh_packet::PayloadVariant::Decoded(data) => {
                match data.portnum() {
                    // We care about these four payload types for sure!
                    PortNum::PositionApp => match Position::decode(data.payload.as_slice()) {
                        Ok(p) => {}
                        Err(e) => log_msg(format!("{e}").as_str(), log::Level::Warn),
                    },
                    PortNum::NodeinfoApp => match NodeInfo::decode(data.payload.as_slice()) {
                        Ok(ni) => {
                            let row: Devicemetric =
                                ni.to_row(Oid(pkt.id), Oid(pkt.from), timestamp(pkt.rx_time));
                            match row.insert(pool).await {
                                Ok(_) => log_msg(
                                    "Inserted 1 row into DeviceMetrics table",
                                    log::Level::Info,
                                ),
                                Err(_) => {
                                    // Try updating the row
                                    match row.update(pool).await {
                                        Ok(_) => log_msg(
                                            "Updated 1 row in NodeInfo table",
                                            log::Level::Info,
                                        ),
                                        Err(e) => {
                                            log_msg(format!("{e}").as_str(), log::Level::Error);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => log_msg(format!("{e}").as_str(), log::Level::Warn),
                    },
                    PortNum::TelemetryApp => match Telemetry::decode(data.payload.as_slice()) {
                        Ok(telemetry) => decode_telemetry(pkt, telemetry, pool).await,
                        Err(e) => log_msg(format!("{e}").as_str(), log::Level::Warn),
                    },
                    PortNum::NeighborinfoApp => match NeighborInfo::decode(data.payload.as_slice())
                    {
                        Ok(ni) => {
                            let row: Neighborinfo =
                                ni.to_row(Oid(pkt.id), Oid(pkt.from), timestamp(pkt.rx_time));
                            match row.insert(pool).await {
                                Ok(_) => log_msg(
                                    "Inserted 1 row into NeighborInfo table",
                                    log::Level::Info,
                                ),
                                Err(e) => log_msg(format!("{e}").as_str(), log::Level::Error),
                            }
                        }
                        Err(e) => log_msg(format!("{e}").as_str(), log::Level::Warn),
                    },
                    #[cfg(not(feature = "trace"))]
                    _ => log_msg("Received untracked payload", log::Level::Info),
                    // The others are nice for tracing during development
                    #[cfg(feature = "trace")]
                    PortNum::UnknownApp => {
                        decode_and_trace("UnknownApp", data.payload.as_slice());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::TextMessageApp => match String::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("TextMessageApp", payload),
                        Err(e) => log_msg(
                            format!("Error decoding TextMessageApp: {e}").as_str(),
                            log::Level::Warn,
                        ),
                    },
                    #[cfg(feature = "trace")]
                    PortNum::RemoteHardwareApp => {
                        match HardwareMessage::decode(data.payload.as_slice()) {
                            Ok(payload) => decode_and_trace("RemoteHardwareApp", payload),
                            Err(e) => {
                                log_msg(
                                    format!("Error decoding RemoteHardwareApp: {e}").as_str(),
                                    log::Level::Warn,
                                );
                            }
                        }
                    }
                    #[cfg(feature = "trace")]
                    PortNum::RoutingApp => match Routing::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("RoutingApp", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding RoutingApp: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::AdminApp => match AdminMessage::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("AdminApp", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding AdminApp: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::TextMessageCompressedApp => {
                        match Compressed::decode(data.payload.as_slice()) {
                            Ok(payload) => decode_and_trace("TextMessageCompressedApp", payload),
                            Err(e) => {
                                log_msg(
                                    format!("Error decoding TextMessageCompressedApp: {e}")
                                        .as_str(),
                                    log::Level::Warn,
                                );
                            }
                        }
                    }
                    #[cfg(feature = "trace")]
                    PortNum::WaypointApp => match Waypoint::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("WaypointApp", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding WaypointApp: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::AudioApp => {
                        decode_and_trace("AudioApp", data.payload.as_slice());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::DetectionSensorApp => match String::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("DetectionSensorApp", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding DetectionSensorApp: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::AlertApp => match String::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("AlertApp", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding AlertApp: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::ReplyApp => match String::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("ReplyApp", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding ReplyApp: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::IpTunnelApp => {
                        decode_and_trace("IpTunnelApp", data.payload.as_slice());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::PaxcounterApp => match Paxcount::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("PaxcounterApp", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding PaxcounterApp: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::SerialApp => {
                        decode_and_trace("SerialApp", data.payload.as_slice());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::StoreForwardApp => {
                        match StoreAndForward::decode(data.payload.as_slice()) {
                            Ok(payload) => decode_and_trace("StoreForwardApp", payload),
                            Err(e) => {
                                log_msg(
                                    format!("Error decoding StoreForwardApp: {e}").as_str(),
                                    log::Level::Warn,
                                );
                            }
                        }
                    }
                    #[cfg(feature = "trace")]
                    PortNum::RangeTestApp => match String::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("RangeTestApp", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding RangeTestApp: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::ZpsApp => {
                        decode_and_trace("ZpsApp", data.payload.as_slice());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::SimulatorApp => {
                        decode_and_trace("SimulatorApp", data.payload.as_slice());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::TracerouteApp => {
                        match RouteDiscovery::decode(data.payload.as_slice()) {
                            Ok(payload) => decode_and_trace("TracerouteApp", payload),
                            Err(e) => {
                                log_msg(
                                    format!("Error decoding TracerouteApp: {e}").as_str(),
                                    log::Level::Warn,
                                );
                            }
                        }
                    }
                    #[cfg(feature = "trace")]
                    PortNum::AtakPlugin => match TakPacket::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("AtakPlugin", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding AtakPlugin: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::MapReportApp => match MapReport::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("MapReportApp", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding MapReportApp: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::PowerstressApp => {
                        match PowerStressMessage::decode(data.payload.as_slice()) {
                            Ok(payload) => decode_and_trace("PowerstressApp", payload),
                            Err(e) => {
                                log_msg(
                                    format!("Error decoding PowerstressApp: {e}").as_str(),
                                    log::Level::Warn,
                                );
                            }
                        }
                    }
                    #[cfg(feature = "trace")]
                    PortNum::PrivateApp => {
                        decode_and_trace("PrivateApp", data.payload.as_slice());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::AtakForwarder => match TakPacket::decode(data.payload.as_slice()) {
                        Ok(payload) => decode_and_trace("AtakForwarder", payload),
                        Err(e) => {
                            log_msg(
                                format!("Error decoding AtakForwarder: {e}").as_str(),
                                log::Level::Warn,
                            );
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::Max => {
                        decode_and_trace("Max", data.payload.as_slice());
                    }
                }
            }
            #[cfg(not(feature = "trace"))]
            _ => (),
            #[cfg(feature = "trace")]
            mesh_packet::PayloadVariant::Encrypted(items) => {
                log_msg(
                    format!("Received encrypted packet: {items:?}").as_str(),
                    log::Level::Info,
                );
            }
        }
    }
}

async fn decode_telemetry(pkt: &MeshPacket, tm: Telemetry, pool: &Pool<Postgres>) {
    if let Some(data) = tm.variant {
        match data {
            Variant::DeviceMetrics(device_metrics) => {
                let row: Devicemetric =
                    device_metrics.to_row(Oid(pkt.id), Oid(pkt.from), timestamp(tm.time));
                match row.insert(pool).await {
                    Ok(_) => log_msg("Inserted 1 row into DeviceMetrics table", log::Level::Info),
                    Err(e) => log_msg(format!("{e}").as_str(), log::Level::Error),
                }
            }
            Variant::EnvironmentMetrics(environment_metrics) => {
                let row: Environmentmetric =
                    environment_metrics.to_row(Oid(pkt.id), Oid(pkt.from), timestamp(tm.time));
                match row.insert(pool).await {
                    Ok(_) => log_msg(
                        "Inserted 1 row into EnvironmentMetrics table",
                        log::Level::Info,
                    ),
                    Err(e) => log_msg(format!("{e}").as_str(), log::Level::Error),
                }
            }
            Variant::AirQualityMetrics(air_quality_metrics) => {
                let row: Airqualitymetric =
                    air_quality_metrics.to_row(Oid(pkt.id), Oid(pkt.from), timestamp(tm.time));
                match row.insert(pool).await {
                    Ok(_) => log_msg(
                        "Inserted 1 row into AirQualityMetrics table",
                        log::Level::Info,
                    ),
                    Err(e) => log_msg(format!("{e}").as_str(), log::Level::Error),
                }
            }
            Variant::LocalStats(local_stats) => {
                let row: Localstat =
                    local_stats.to_row(Oid(pkt.id), Oid(pkt.from), timestamp(tm.time));
                match row.insert(pool).await {
                    Ok(_) => log_msg("Inserted 1 row into LocalStats table", log::Level::Info),
                    Err(e) => log_msg(format!("{e}").as_str(), log::Level::Error),
                }
            }
            Variant::ErrorMetrics(error_metrics) => {
                let row: Errormetric =
                    error_metrics.to_row(Oid(pkt.id), Oid(pkt.from), timestamp(tm.time));
                match row.insert(pool).await {
                    Ok(_) => log_msg("Inserted 1 row into ErrorMetrics table", log::Level::Info),
                    Err(e) => log_msg(format!("{e}").as_str(), log::Level::Error),
                }
            }
            #[cfg(not(feature = "trace"))]
            _ => {}
            #[cfg(feature = "trace")]
            Variant::PowerMetrics(power_metrics) => {
                log_msg(
                    format!("Received PowerMetrics packet: {power_metrics:?}").as_str(),
                    log::Level::Info,
                );
            }
            #[cfg(feature = "trace")]
            Variant::HealthMetrics(health_metrics) => {
                log_msg(
                    format!("Received HealthMetrics packet: {health_metrics:?}").as_str(),
                    log::Level::Info,
                );
            }
        }
    }
}
