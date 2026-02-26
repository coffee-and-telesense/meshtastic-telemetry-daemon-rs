use crate::{
    dto::dbops::{
        airqualitymetrics, devicemetrics, environmentmetrics, errormetrics, localstats,
        neighborinfo, nodeinfo, powermetrics,
    },
    log_msg,
    util::state::GatewayState,
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
use sqlx::{Pool, Postgres};
#[cfg(feature = "trace")]
use std::fmt::Debug;
use std::sync::Arc;

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
pub async fn process_packet(pkt: &FromRadio, state: &Arc<GatewayState>, pool: &Pool<Postgres>) {
    if let Some(pv) = &pkt.payload_variant {
        match pv {
            from_radio::PayloadVariant::Packet(mesh_packet) => {
                decode_payload(mesh_packet, state, pool).await;
            }
            from_radio::PayloadVariant::NodeInfo(node_info) => {
                // only insert if user is some
                if node_info.user.is_some() {
                    let (dm_result, ni_result) = tokio::join!(
                        devicemetrics::upsert_fr(pkt, node_info, pool),
                        nodeinfo::upsert(node_info, pool),
                    );
                    match dm_result {
                        Ok(_) => {
                            log_msg!(log::Level::Info, "Upserted 1 row into DeviceMetrics table");
                        }
                        #[cfg(feature = "trace")]
                        Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                        #[cfg(not(feature = "trace"))]
                        Err(e) => log_msg!(log::Level::Error, "{e}"),
                    }

                    match ni_result {
                        Ok(_) => log_msg!(log::Level::Info, "Upserted 1 row into NodeInfo table"),
                        #[cfg(feature = "trace")]
                        Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                        #[cfg(not(feature = "trace"))]
                        Err(e) => log_msg!(log::Level::Error, "{e}"),
                    }
                    // insert into GatewayState
                    #[cfg(feature = "debug")]
                    if let Some(user) = &node_info.user {
                        state.insert(node_info.num, user);
                    }
                }
            }
            from_radio::PayloadVariant::MyInfo(my_node_info) => {
                #[cfg(feature = "trace")]
                log_msg!(log::Level::Info, "Received MyInfo packet: {my_node_info:?}");
                // Indicate the serial connection for the local state from this packet
                state.set_serial_number(my_node_info.my_node_num);
            }
            #[cfg(not(feature = "trace"))]
            _ => (),
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::Config(config) => {
                log_msg!(log::Level::Info, "Received config packet: {config:?}");
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::LogRecord(log_record) => {
                log_msg!(
                    log::Level::Info,
                    "Received log_record packet: {log_record:?}"
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::ConfigCompleteId(id) => {
                log_msg!(
                    log::Level::Info,
                    "Received config {id} complete packet over serial"
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::Rebooted(rbt) => {
                log_msg!(log::Level::Info, "Received rebooted packet: {rbt}");
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::ModuleConfig(module_config) => {
                log_msg!(
                    log::Level::Info,
                    "Received module_config packet: {module_config:?}"
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::Channel(channel) => {
                log_msg!(log::Level::Info, "Received channel packet: {channel:?}");
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::QueueStatus(queue_status) => {
                log_msg!(
                    log::Level::Info,
                    "Received queue_status packet: {queue_status:?}"
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::XmodemPacket(xmodem) => {
                log_msg!(log::Level::Info, "Received xmodem packet: {xmodem:?}");
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::Metadata(device_metadata) => {
                log_msg!(
                    log::Level::Info,
                    "Received device_metadata packet: {device_metadata:?}"
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::MqttClientProxyMessage(mqtt_client_proxy_message) => {
                log_msg!(
                    log::Level::Info,
                    "Received mqtt_client_proxy_message packet: {mqtt_client_proxy_message:?}"
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::FileInfo(file_info) => {
                log_msg!(log::Level::Info, "Received file_info packet: {file_info:?}");
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::ClientNotification(client_notification) => {
                log_msg!(
                    log::Level::Info,
                    "Received client_notification packet: {client_notification:?}"
                );
            }
            #[cfg(feature = "trace")]
            from_radio::PayloadVariant::DeviceuiConfig(device_ui_config) => {
                log_msg!(
                    log::Level::Info,
                    "Received device_ui_config packet: {device_ui_config:?}"
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
    log_msg!(log::Level::Info, "Received {ptype} packet: {payload:?}");
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
async fn decode_payload(pkt: &MeshPacket, state: &Arc<GatewayState>, pool: &Pool<Postgres>) {
    // Count received packets in debug builds for period reporting in logs
    #[cfg(feature = "debug")]
    {
        if let Some(counter) = state.get_counter(pkt.from) {
            counter.increment();
        } else {
            log_msg!(
                log::Level::Debug,
                "rx count missed for unregistered node {:08x}",
                pkt.from
            );
        }
    }
    // Check if the packet is on the telemetry channel before decoding a payload
    if pkt.channel == 0
        && let Some(payload) = &pkt.payload_variant
    {
        match payload {
            mesh_packet::PayloadVariant::Decoded(data) => {
                match data.portnum() {
                    // We care about these four payload types for sure!
                    PortNum::PositionApp => match Position::decode(data.payload.clone()) {
                        Ok(pos) => match devicemetrics::insert_pos(pkt, &pos, pool).await {
                            Ok(_) => log_msg!(
                                log::Level::Info,
                                "Inserted 1 row into DeviceMetrics table"
                            ),
                            #[cfg(feature = "trace")]
                            Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                            #[cfg(not(feature = "trace"))]
                            Err(e) => log_msg!(log::Level::Error, "{e}"),
                        },
                        #[cfg(feature = "trace")]
                        Err(e) => log_msg!(log::Level::Warn, "{e:?}"),
                        #[cfg(not(feature = "trace"))]
                        Err(e) => log_msg!(log::Level::Warn, "{e}"),
                    },
                    PortNum::NodeinfoApp => match NodeInfo::decode(data.payload.clone()) {
                        Ok(ni) => {
                            let (dm_result, ni_result) = tokio::join!(
                                devicemetrics::upsert_mp(pkt, &ni, pool),
                                nodeinfo::upsert(&ni, pool),
                            );

                            match dm_result {
                                Ok(_) => log_msg!(
                                    log::Level::Info,
                                    "Upserted 1 row into DeviceMetrics table"
                                ),
                                #[cfg(feature = "trace")]
                                Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                                #[cfg(not(feature = "trace"))]
                                Err(e) => log_msg!(log::Level::Error, "{e}"),
                            }

                            match ni_result {
                                Ok(_) => {
                                    log_msg!(
                                        log::Level::Info,
                                        "Upserted 1 row into NodeInfo table"
                                    );
                                }
                                #[cfg(feature = "trace")]
                                Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                                #[cfg(not(feature = "trace"))]
                                Err(e) => log_msg!(log::Level::Error, "{e}"),
                            }

                            // insert into GatewayState
                            #[cfg(feature = "debug")]
                            if let Some(user) = &ni.user {
                                state.insert(ni.num, user);
                            }
                        }
                        #[cfg(feature = "trace")]
                        Err(e) => log_msg!(log::Level::Warn, "{e:?}"),
                        #[cfg(not(feature = "trace"))]
                        Err(e) => log_msg!(log::Level::Warn, "{e}"),
                    },
                    PortNum::TelemetryApp => match Telemetry::decode(data.payload.clone()) {
                        Ok(telemetry) => decode_telemetry(pkt, telemetry, pool).await,
                        #[cfg(feature = "trace")]
                        Err(e) => log_msg!(log::Level::Warn, "{e:?}"),
                        #[cfg(not(feature = "trace"))]
                        Err(e) => log_msg!(log::Level::Warn, "{e}"),
                    },
                    PortNum::NeighborinfoApp => match NeighborInfo::decode(data.payload.clone()) {
                        Ok(ni) => match neighborinfo::insert(pkt, &ni, pool).await {
                            Ok(_) => {
                                log_msg!(
                                    log::Level::Info,
                                    "Inserted 1 row into NeighborInfo table"
                                );
                            }
                            #[cfg(feature = "trace")]
                            Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                            #[cfg(not(feature = "trace"))]
                            Err(e) => log_msg!(log::Level::Error, "{e}"),
                        },
                        #[cfg(feature = "trace")]
                        Err(e) => log_msg!(log::Level::Warn, "{e:?}"),
                        #[cfg(not(feature = "trace"))]
                        Err(e) => log_msg!(log::Level::Warn, "{e}"),
                    },
                    #[cfg(not(feature = "trace"))]
                    _ => log_msg!(log::Level::Info, "Received untracked payload"),
                    // The others are nice for tracing during development
                    #[cfg(feature = "trace")]
                    PortNum::UnknownApp => {
                        decode_and_trace("UnknownApp", data.payload.clone());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::TextMessageApp => match String::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("TextMessageApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding TextMessageApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::RemoteHardwareApp => {
                        match HardwareMessage::decode(data.payload.clone()) {
                            Ok(payload) => decode_and_trace("RemoteHardwareApp", payload),
                            Err(e) => {
                                log_msg!(
                                    log::Level::Warn,
                                    "Error decoding RemoteHardwareApp: {e:?}"
                                );
                            }
                        }
                    }
                    #[cfg(feature = "trace")]
                    PortNum::RoutingApp => match Routing::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("RoutingApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding RoutingApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::AdminApp => match AdminMessage::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("AdminApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding AdminApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::TextMessageCompressedApp => {
                        match Compressed::decode(data.payload.clone()) {
                            Ok(payload) => decode_and_trace("TextMessageCompressedApp", payload),
                            Err(e) => {
                                log_msg!(
                                    log::Level::Warn,
                                    "Error decoding TextMessageCompressedApp: {e:?}"
                                );
                            }
                        }
                    }
                    #[cfg(feature = "trace")]
                    PortNum::WaypointApp => match Waypoint::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("WaypointApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding WaypointApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::AudioApp => {
                        decode_and_trace("AudioApp", data.payload.clone());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::DetectionSensorApp => match String::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("DetectionSensorApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding DetectionSensorApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::AlertApp => match String::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("AlertApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding AlertApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::ReplyApp => match String::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("ReplyApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding ReplyApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::IpTunnelApp => {
                        decode_and_trace("IpTunnelApp", data.payload.clone());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::PaxcounterApp => match Paxcount::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("PaxcounterApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding PaxcounterApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::SerialApp => {
                        decode_and_trace("SerialApp", data.payload.clone());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::StoreForwardApp => {
                        match StoreAndForward::decode(data.payload.clone()) {
                            Ok(payload) => decode_and_trace("StoreForwardApp", payload),
                            Err(e) => {
                                log_msg!(log::Level::Warn, "Error decoding StoreForwardApp: {e:?}");
                            }
                        }
                    }
                    #[cfg(feature = "trace")]
                    PortNum::RangeTestApp => match String::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("RangeTestApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding RangeTestApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::ZpsApp => {
                        decode_and_trace("ZpsApp", data.payload.clone());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::SimulatorApp => {
                        decode_and_trace("SimulatorApp", data.payload.clone());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::TracerouteApp => match RouteDiscovery::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("TracerouteApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding TracerouteApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::AtakPlugin => match TakPacket::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("AtakPlugin", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding AtakPlugin: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::MapReportApp => match MapReport::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("MapReportApp", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding MapReportApp: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::PowerstressApp => {
                        match PowerStressMessage::decode(data.payload.clone()) {
                            Ok(payload) => decode_and_trace("PowerstressApp", payload),
                            Err(e) => {
                                log_msg!(log::Level::Warn, "Error decoding PowerstressApp: {e:?}");
                            }
                        }
                    }
                    #[cfg(feature = "trace")]
                    PortNum::PrivateApp => {
                        decode_and_trace("PrivateApp", data.payload.clone());
                    }
                    #[cfg(feature = "trace")]
                    PortNum::AtakForwarder => match TakPacket::decode(data.payload.clone()) {
                        Ok(payload) => decode_and_trace("AtakForwarder", payload),
                        Err(e) => {
                            log_msg!(log::Level::Warn, "Error decoding AtakForwarder: {e:?}");
                        }
                    },
                    #[cfg(feature = "trace")]
                    PortNum::Max => {
                        decode_and_trace("Max", data.payload.clone());
                    }
                }
            }
            #[cfg(not(feature = "trace"))]
            _ => (),
            #[cfg(feature = "trace")]
            mesh_packet::PayloadVariant::Encrypted(items) => {
                log_msg!(log::Level::Info, "Received encrypted packet: {items:?}");
            }
        }
    }
}

async fn decode_telemetry(pkt: &MeshPacket, tm: Telemetry, pool: &Pool<Postgres>) {
    if let Some(data) = tm.variant {
        match data {
            Variant::DeviceMetrics(device_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/DeviceMetrics", device_metrics);
                match devicemetrics::insert_dm(pkt, &tm, &device_metrics, pool).await {
                    Ok(_) => log_msg!(log::Level::Info, "Inserted 1 row into DeviceMetrics table"),
                    #[cfg(feature = "trace")]
                    Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                    #[cfg(not(feature = "trace"))]
                    Err(e) => log_msg!(log::Level::Error, "{e}"),
                }
            }
            Variant::EnvironmentMetrics(environment_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/EnvironmentMetrics", environment_metrics);
                match environmentmetrics::insert(pkt, &tm, &environment_metrics, pool).await {
                    Ok(_) => log_msg!(
                        log::Level::Info,
                        "Inserted 1 row into EnvironmentMetrics table"
                    ),
                    #[cfg(feature = "trace")]
                    Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                    #[cfg(not(feature = "trace"))]
                    Err(e) => log_msg!(log::Level::Error, "{e}"),
                }
            }
            Variant::AirQualityMetrics(air_quality_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/AirQualityMetrics", air_quality_metrics);
                match airqualitymetrics::insert(pkt, &tm, &air_quality_metrics, pool).await {
                    Ok(_) => log_msg!(
                        log::Level::Info,
                        "Inserted 1 row into AirQualityMetrics table"
                    ),
                    #[cfg(feature = "trace")]
                    Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                    #[cfg(not(feature = "trace"))]
                    Err(e) => log_msg!(log::Level::Error, "{e}"),
                }
            }
            Variant::LocalStats(local_stats) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/LocalStats", local_stats);
                match localstats::insert(pkt, &tm, &local_stats, pool).await {
                    Ok(_) => log_msg!(log::Level::Info, "Inserted 1 row into LocalStats table"),
                    #[cfg(feature = "trace")]
                    Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                    #[cfg(not(feature = "trace"))]
                    Err(e) => log_msg!(log::Level::Error, "{e}"),
                }
            }
            Variant::ErrorMetrics(error_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/ErrorMetrics", error_metrics);
                match errormetrics::insert(pkt, &tm, &error_metrics, pool).await {
                    Ok(_) => log_msg!(log::Level::Info, "Inserted 1 row into ErrorMetrics table"),
                    #[cfg(feature = "trace")]
                    Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                    #[cfg(not(feature = "trace"))]
                    Err(e) => log_msg!(log::Level::Error, "{e}"),
                }
            }
            Variant::PowerMetrics(power_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/PowerMetrics", power_metrics);
                match powermetrics::insert(pkt, &tm, &power_metrics, pool).await {
                    Ok(_) => log_msg!(log::Level::Info, "Inserted 1 row into PowerMetrics table"),
                    #[cfg(feature = "trace")]
                    Err(e) => log_msg!(log::Level::Error, "{e:?}"),
                    #[cfg(not(feature = "trace"))]
                    Err(e) => log_msg!(log::Level::Error, "{e}"),
                }
            }
            #[cfg(not(feature = "trace"))]
            _ => {}
            #[cfg(feature = "trace")]
            Variant::HealthMetrics(health_metrics) => {
                decode_and_trace("Telemetry/HealthMetrics", health_metrics);
            }
        }
    }
}
