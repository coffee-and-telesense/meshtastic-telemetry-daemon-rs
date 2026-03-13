use crate::{
    dto::dbops::{
        airqualitymetrics, devicemetrics, environmentmetrics, errormetrics, localstats,
        neighborinfo, nodeinfo, powermetrics,
    },
    util::state::GatewayState,
};
#[cfg(feature = "trace")]
use meshtastic::protobufs::{
    AdminMessage, Compressed, Data, HardwareMessage, MapReport, Paxcount, PowerStressMessage,
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

/// Dispatches a `FromRadio` packet to the appropriate database insert or upsert.
pub(crate) async fn process_packet(
    pkt: &FromRadio,
    state: &Arc<GatewayState>,
    pool: &Pool<Postgres>,
) {
    if let Some(pv) = &pkt.payload_variant {
        match pv {
            from_radio::PayloadVariant::Packet(mesh_packet) => {
                decode_payload(mesh_packet, state, pool).await;
            }
            from_radio::PayloadVariant::NodeInfo(node_info) => {
                //TODO: collapse if statement, account for the position/devicemetrics as well
                // only insert if user is some
                if node_info.user.is_some() {
                    let (dm_result, ni_result) = tokio::join!(
                        devicemetrics::upsert_fr(pkt, node_info, pool),
                        nodeinfo::upsert(node_info, pool),
                    );
                    match dm_result {
                        Ok(_) => {
                            tracing::info!(table = "DeviceMetrics", "upserted 1 row");
                        }
                        Err(e) => tracing::error!(%e, table = "DeviceMetrics", "upsert failed"),
                    }

                    match ni_result {
                        Ok(_) => tracing::info!(table = "NodeInfo", "upserted 1 row"),
                        Err(e) => tracing::error!(%e, table = "NodeInfo", "upsert failed"),
                    }
                    // insert into GatewayState
                    #[cfg(feature = "debug")]
                    if let Some(user) = &node_info.user {
                        match state.insert(node_info.num, user) {
                            Ok(()) => tracing::trace!("Added {} to GatewayState", node_info.num),
                            Err(e) => tracing::warn!(%e),
                        }
                    }
                }
            }
            from_radio::PayloadVariant::MyInfo(my_node_info) => {
                #[cfg(feature = "trace")]
                tracing::info!("Received MyInfo packet: {my_node_info:?}");
                // Indicate the serial connection for the local state from this packet
                state.set_serial_number(my_node_info.my_node_num);
            }
            _other => {
                #[cfg(feature = "trace")]
                #[expect(
                    clippy::used_underscore_binding,
                    reason = "conditionally compiled variable"
                )]
                trace_fromradio(_other);
            }
        }
    }
}

#[cfg(feature = "trace")]
fn trace_fromradio(payload: &from_radio::PayloadVariant) {
    match payload {
        from_radio::PayloadVariant::Config(config) => {
            tracing::trace!("Received config packet: {config:?}");
        }
        from_radio::PayloadVariant::LogRecord(log_record) => {
            tracing::trace!("Received log_record packet: {log_record:?}");
        }
        from_radio::PayloadVariant::ConfigCompleteId(id) => {
            tracing::trace!("Received config {id} complete packet over serial");
        }
        from_radio::PayloadVariant::Rebooted(rbt) => {
            tracing::trace!("Received rebooted packet: {rbt}");
        }
        from_radio::PayloadVariant::ModuleConfig(module_config) => {
            tracing::trace!("Received module_config packet: {module_config:?}");
        }
        from_radio::PayloadVariant::Channel(channel) => {
            tracing::trace!("Received channel packet: {channel:?}");
        }
        from_radio::PayloadVariant::QueueStatus(queue_status) => {
            tracing::trace!("Received queue_status packet: {queue_status:?}");
        }
        from_radio::PayloadVariant::XmodemPacket(xmodem) => {
            tracing::trace!("Received xmodem packet: {xmodem:?}");
        }
        from_radio::PayloadVariant::Metadata(device_metadata) => {
            tracing::trace!("Received device_metadata packet: {device_metadata:?}");
        }
        from_radio::PayloadVariant::MqttClientProxyMessage(mqtt_client_proxy_message) => {
            tracing::trace!(
                "Received mqtt_client_proxy_message packet: {mqtt_client_proxy_message:?}"
            );
        }
        from_radio::PayloadVariant::FileInfo(file_info) => {
            tracing::trace!("Received file_info packet: {file_info:?}");
        }
        from_radio::PayloadVariant::ClientNotification(client_notification) => {
            tracing::trace!("Received client_notification packet: {client_notification:?}");
        }
        from_radio::PayloadVariant::DeviceuiConfig(device_ui_config) => {
            tracing::trace!("Received device_ui_config packet: {device_ui_config:?}");
        }
        _ => tracing::trace!("Received an unknown from_radio PayloadVariant"),
    }
}

/// Trace logging decoded payloads
#[cfg(feature = "trace")]
#[inline]
fn decode_and_trace<P: Debug>(ptype: &str, payload: P) {
    tracing::trace!("Received {ptype} packet: {payload:?}");
}

/// Decodes a `MeshPacket` payload and inserts the result into the database.
async fn decode_payload(pkt: &MeshPacket, state: &Arc<GatewayState>, pool: &Pool<Postgres>) {
    // Count received packets in debug builds for period reporting in logs
    #[cfg(feature = "debug")]
    {
        if !state.increment_count(pkt.from) {
            tracing::debug!("rx count missed for unregistered node {:08x}", pkt.from);
        }
    }
    // Check if the packet is on the telemetry channel before decoding a payload
    if pkt.channel != 0 {
        return;
    }
    let Some(payload) = &pkt.payload_variant else {
        return;
    };
    let mesh_packet::PayloadVariant::Decoded(data) = payload else {
        #[cfg(feature = "trace")]
        trace_encrypted(payload);
        return;
    };

    match data.portnum() {
        // We care about these four payload types for sure!
        PortNum::PositionApp => match Position::decode(data.payload.as_ref()) {
            Ok(pos) => match devicemetrics::insert_pos(pkt, &pos, pool).await {
                Ok(_) => tracing::info!(table = "DeviceMetrics", "inserted 1 row of position data"),
                Err(e) => {
                    tracing::error!(%e, table = "DeviceMetrics", "inserting position data failed");
                }
            },
            Err(e) => {
                tracing::warn!(%e, node_id = pkt.from, portnum = ?PortNum::PositionApp, "decode failed");
            }
        },
        PortNum::NodeinfoApp => match NodeInfo::decode(data.payload.as_ref()) {
            Ok(ni) => {
                let (dm_result, ni_result) = tokio::join!(
                    devicemetrics::upsert_mp(pkt, &ni, pool),
                    nodeinfo::upsert(&ni, pool),
                );

                match dm_result {
                    Ok(_) => tracing::info!(table = "DeviceMetrics", "upserted 1 row"),
                    Err(e) => tracing::error!(%e, table = "DeviceMetrics", "upsert failed"),
                }

                match ni_result {
                    Ok(_) => tracing::info!(table = "NodeInfo", "upserted 1 row"),
                    Err(e) => tracing::error!(%e, table = "NodeInfo", "upsert failed"),
                }

                // insert into GatewayState
                #[cfg(feature = "debug")]
                if let Some(user) = &ni.user {
                    match state.insert(ni.num, user) {
                        Ok(()) => tracing::trace!("Added {} to GatewayState", ni.num),
                        Err(e) => tracing::warn!(%e),
                    }
                }
            }
            Err(e) => {
                tracing::error!(%e, node_id = pkt.from, portnum = ?PortNum::NodeinfoApp, "decode failed");
            }
        },
        PortNum::TelemetryApp => match Telemetry::decode(data.payload.as_ref()) {
            Ok(telemetry) => decode_telemetry(pkt, &telemetry, pool).await,
            Err(e) => {
                tracing::warn!(%e, node_id = pkt.from, portnum = ?PortNum::TelemetryApp, "decode failed");
            }
        },
        PortNum::NeighborinfoApp => match NeighborInfo::decode(data.payload.as_ref()) {
            Ok(ni) => match neighborinfo::insert(pkt, &ni, pool).await {
                Ok(_) => tracing::info!(table = "NeighborInfo", "inserted 1 row"),
                Err(e) => tracing::error!(%e, table = "NeighborInfo", "insert failed"),
            },
            Err(e) => {
                tracing::warn!(%e, node_id = pkt.from, portnum = ?PortNum::NeighborinfoApp, "decode failed");
            }
        },
        _other => {
            #[cfg(feature = "trace")]
            #[expect(
                clippy::used_underscore_binding,
                reason = "conditionally compiled variable"
            )]
            trace_portnum(_other, data);
        }
    }
}

#[cfg(feature = "trace")]
fn trace_encrypted(payload: &mesh_packet::PayloadVariant) {
    let mesh_packet::PayloadVariant::Encrypted(items) = payload else {
        return;
    };
    tracing::trace!("Received encrypted packet: {items:?}");
}

#[cfg(feature = "trace")]
#[expect(
    clippy::too_many_lines,
    reason = "most of these lines are just logging calls for tracing"
)]
fn trace_portnum(port: PortNum, data: &Data) {
    match port {
        PortNum::UnknownApp => {
            decode_and_trace("UnknownApp", data.payload.as_ref());
        }
        PortNum::TextMessageApp => match String::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("TextMessageApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::RemoteHardwareApp => match HardwareMessage::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("RemoteHardwareApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::RoutingApp => match Routing::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("RoutingApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::AdminApp => match AdminMessage::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("AdminApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::TextMessageCompressedApp => match Compressed::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("TextMessageCompressedApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::WaypointApp => match Waypoint::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("WaypointApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::AudioApp => {
            decode_and_trace("AudioApp", data.payload.as_ref());
        }
        PortNum::DetectionSensorApp => match String::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("DetectionSensorApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::AlertApp => match String::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("AlertApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::ReplyApp => match String::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("ReplyApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::IpTunnelApp => {
            decode_and_trace("IpTunnelApp", data.payload.as_ref());
        }
        PortNum::PaxcounterApp => match Paxcount::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("PaxcounterApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::SerialApp => {
            decode_and_trace("SerialApp", data.payload.as_ref());
        }
        PortNum::StoreForwardApp => match StoreAndForward::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("StoreForwardApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::RangeTestApp => match String::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("RangeTestApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::ZpsApp => {
            decode_and_trace("ZpsApp", data.payload.as_ref());
        }
        PortNum::SimulatorApp => {
            decode_and_trace("SimulatorApp", data.payload.as_ref());
        }
        PortNum::TracerouteApp => match RouteDiscovery::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("TracerouteApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::AtakPlugin => match TakPacket::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("AtakPlugin", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::MapReportApp => match MapReport::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("MapReportApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::PowerstressApp => match PowerStressMessage::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("PowerstressApp", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::PrivateApp => {
            decode_and_trace("PrivateApp", data.payload.as_ref());
        }
        PortNum::AtakForwarder => match TakPacket::decode(data.payload.as_ref()) {
            Ok(payload) => decode_and_trace("AtakForwarder", payload),
            Err(e) => {
                tracing::warn!(%e, portnum = ?port, "decode failed");
            }
        },
        PortNum::Max => {
            decode_and_trace("Max", data.payload.as_ref());
        }
        _ => tracing::debug!("Received an unknown PortNum"),
    }
}

/// Dispatches a telemetry variant to the matching database insert.
async fn decode_telemetry(pkt: &MeshPacket, tm: &Telemetry, pool: &Pool<Postgres>) {
    if let Some(data) = tm.variant {
        match data {
            Variant::DeviceMetrics(device_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/DeviceMetrics", device_metrics);
                match devicemetrics::insert_dm(pkt, tm, &device_metrics, pool).await {
                    Ok(_) => {
                        tracing::info!(table = "DeviceMetrics", node_id = pkt.from, "insert 1 row");
                    }
                    Err(e) => {
                        tracing::error!(%e, table = "DeviceMetrics", node_id = pkt.from, "insert failed");
                    }
                }
            }
            Variant::EnvironmentMetrics(environment_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/EnvironmentMetrics", environment_metrics);
                match environmentmetrics::insert(pkt, tm, &environment_metrics, pool).await {
                    Ok(_) => tracing::info!(
                        table = "EnvironmentMetrics",
                        node_id = pkt.from,
                        "inserted 1 row"
                    ),
                    Err(e) => {
                        tracing::error!(%e, table = "EnvironmentMetrics", node_id = pkt.from, "insert failed");
                    }
                }
            }
            Variant::AirQualityMetrics(air_quality_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/AirQualityMetrics", air_quality_metrics);
                match airqualitymetrics::insert(pkt, tm, &air_quality_metrics, pool).await {
                    Ok(_) => tracing::info!(
                        table = "AirQualityMetrics",
                        node_id = pkt.from,
                        "inserted 1 row"
                    ),
                    Err(e) => {
                        tracing::error!(%e, table = "AirQualityMetrics", node_id = pkt.from, "insert failed");
                    }
                }
            }
            Variant::LocalStats(local_stats) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/LocalStats", local_stats);
                match localstats::insert(pkt, tm, &local_stats, pool).await {
                    Ok(_) => {
                        tracing::info!(table = "LocalStats", node_id = pkt.from, "inserted 1 row");
                    }
                    Err(e) => {
                        tracing::error!(%e, table = "LocalStats", node_id = pkt.from, "insert failed");
                    }
                }
            }
            Variant::ErrorMetrics(error_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/ErrorMetrics", error_metrics);
                match errormetrics::insert(pkt, tm, &error_metrics, pool).await {
                    Ok(_) => {
                        tracing::info!(
                            table = "ErrorMetrics",
                            node_id = pkt.from,
                            "inserted 1 row"
                        );
                    }
                    Err(e) => {
                        tracing::error!(%e, table = "ErrorMetrics", node_id = pkt.from, "insert failed");
                    }
                }
            }
            Variant::PowerMetrics(power_metrics) => {
                #[cfg(feature = "trace")]
                decode_and_trace("Telemetry/PowerMetrics", power_metrics);
                match powermetrics::insert(pkt, tm, &power_metrics, pool).await {
                    Ok(_) => {
                        tracing::info!(
                            table = "PowerMetrics",
                            node_id = pkt.from,
                            "inserted 1 row"
                        );
                    }
                    Err(e) => {
                        tracing::error!(%e, table = "PowerMetrics", node_id = pkt.from, "insert failed");
                    }
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
