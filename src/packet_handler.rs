use std::sync::{Arc, Mutex};

use crate::types::{GatewayState, MyInfo};

use super::types::{Mesh, NInfo, Payload, Pkt, Telem};
use anyhow::Context;
use meshtastic::protobufs::{
    from_radio, mesh_packet, telemetry, FromRadio, NeighborInfo, PortNum, Position, RouteDiscovery,
    Routing, User,
};
use meshtastic::Message;

// Shout-out to https://github.com/PeterGrace/meshtui for some of the code structure here
pub fn process_packet(packet: FromRadio, state: Arc<Mutex<GatewayState>>) -> Option<Pkt> {
    if let Some(payload_v) = packet.clone().payload_variant {
        match payload_v {
            from_radio::PayloadVariant::Packet(pa) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.MeshPacket.html
                let mut pkt: Mesh = Mesh::from_remote(pa.clone());
                // Check if the mesh packet is on the telemetry channel, if not ignore it
                if pkt.channel != 0 {
                    return None;
                }

                if let Some(payload) = pa.payload_variant {
                    match payload.clone() {
                        mesh_packet::PayloadVariant::Decoded(de) => {
                            match de.portnum() {
                                PortNum::PositionApp => {
                                    match Position::decode(de.payload.as_slice()).with_context(
                                        || "Failed to decode Position payload from mesh",
                                    ) {
                                        Ok(data) => {
                                            // Set the packet received time to position timestamp
                                            pkt.rx_time = data.timestamp;
                                            pkt.payload_variant = None;
                                            pkt.payload = Some(Payload::PositionApp(data));
                                            return Some(Pkt::Mesh(pkt));
                                        }
                                        Err(e) => {
                                            info!("{:#}", e);
                                            return None;
                                        }
                                    }
                                }

                                PortNum::TelemetryApp => {
                                    match meshtastic::protobufs::Telemetry::decode(
                                        de.payload.as_slice(),
                                    )
                                    .with_context(|| "Failed to decode Telemetry payload from mesh")
                                    {
                                        Ok(data) => {
                                            pkt.payload_variant = None;
                                            if let Some(v) = data.variant {
                                                //TODO: Set received time from packet time
                                                //currently broken, maybe the nodes need a time set
                                                //in order for it to work?
                                                //pkt.rx_time = data.time;
                                                match v {
                                                    telemetry::Variant::EnvironmentMetrics(env) => {
                                                        pkt.payload = Some(Payload::TelemetryApp(
                                                            Telem::Environment(env),
                                                        ));
                                                        return Some(Pkt::Mesh(pkt));
                                                    }
                                                    telemetry::Variant::DeviceMetrics(dm) => {
                                                        pkt.payload = Some(Payload::TelemetryApp(
                                                            Telem::Device(dm),
                                                        ));
                                                        return Some(Pkt::Mesh(pkt));
                                                    }
                                                    telemetry::Variant::AirQualityMetrics(aqi) => {
                                                        pkt.payload = Some(Payload::TelemetryApp(
                                                            Telem::AirQuality(aqi),
                                                        ));
                                                        return Some(Pkt::Mesh(pkt));
                                                    }
                                                    telemetry::Variant::PowerMetrics(pwm) => {
                                                        pkt.payload = Some(Payload::TelemetryApp(
                                                            Telem::Power(pwm),
                                                        ));
                                                        return Some(Pkt::Mesh(pkt));
                                                    }
                                                    telemetry::Variant::LocalStats(_stats) => {
                                                        //TODO this will be a possible better solution
                                                        return None;
                                                    }
                                                    _ => {
                                                        // Do not care about health metrics right now
                                                        return None;
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            info!("{:#}", e);
                                            return None;
                                        }
                                    }
                                }

                                PortNum::NeighborinfoApp => {
                                    match NeighborInfo::decode(de.payload.as_slice()).with_context(
                                        || "Failed to decode NeighborInfo payload from mesh",
                                    ) {
                                        Ok(data) => {
                                            pkt.payload_variant = None;
                                            pkt.payload = Some(Payload::NeighborinfoApp(data));
                                            return Some(Pkt::Mesh(pkt));
                                        }
                                        Err(e) => {
                                            info!("{:#}", e);
                                            return None;
                                        }
                                    }
                                }

                                PortNum::NodeinfoApp => {
                                    match User::decode(de.payload.as_slice()).with_context(|| {
                                        "Failed to decode NodeInfo payload from mesh"
                                    }) {
                                        Ok(data) => {
                                            // Insert into our node state, will check if it already exists
                                            // (if it does nothing happens, if it doesn't it inserts the
                                            // user)
                                            let rv = state
                                                .lock()
                                                .unwrap()
                                                .insert(pkt.from, data.clone());
                                            pkt.payload_variant = None;
                                            pkt.payload = Some(Payload::NodeinfoApp(data));
                                            if rv {
                                                return Some(Pkt::Mesh(pkt));
                                            } else {
                                                return None;
                                            }
                                        }
                                        Err(e) => {
                                            info!("{:#}", e);
                                            return None;
                                        }
                                    }
                                }

                                PortNum::RoutingApp => {
                                    match Routing::decode(de.payload.as_slice()).with_context(
                                        || "Failed to decode Routing payload from mesh",
                                    ) {
                                        Ok(data) => {
                                            pkt.payload_variant = None;
                                            pkt.payload = Some(Payload::RoutingApp(data));
                                            return Some(Pkt::Mesh(pkt));
                                        }
                                        Err(e) => {
                                            info!("{:#}", e);
                                            return None;
                                        }
                                    }
                                }

                                PortNum::TracerouteApp => {
                                    match RouteDiscovery::decode(de.payload.as_slice())
                                        .with_context(|| {
                                            "Failed to decode Traceroute payload from mesh"
                                        }) {
                                        Ok(data) => {
                                            pkt.payload_variant = None;
                                            pkt.payload = Some(Payload::TracerouteApp(data));
                                            return Some(Pkt::Mesh(pkt));
                                        }
                                        Err(e) => {
                                            info!("{:#}", e);
                                            return None;
                                        }
                                    }
                                }

                                _ => {
                                    return None;
                                }
                            }
                        }

                        mesh_packet::PayloadVariant::Encrypted(_) => {
                            info!("Received an encrypted packet.");
                            return None;
                        }
                    }
                }
            }

            from_radio::PayloadVariant::MyInfo(mi) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.MyNodeInfo.html
                let pkt = MyInfo::from_remote(mi);
                return Some(Pkt::MyNodeInfo(pkt));
            }

            from_radio::PayloadVariant::NodeInfo(ni) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.NodeInfo.html
                let pkt = NInfo::from_remote(ni.clone());
                // Check if the mesh packet is on the telemetry channel, if not ignore it
                if pkt.channel != 0 {
                    info!("nodedb info from outside our channel");
                    return None;
                }
                let mut rv = false;
                if let Some(user) = ni.user {
                    // Insert a new node into our local state
                    rv = state.lock().unwrap().insert(ni.num, user);
                }
                if rv {
                    return Some(Pkt::NInfo(pkt));
                } else {
                    return None;
                }
            }

            from_radio::PayloadVariant::Rebooted(reboot) => {
                if reboot {
                    info!("Device rebooted recently");
                } else {
                    info!("Not rebooted recently");
                }
                return None;
            }

            from_radio::PayloadVariant::ModuleConfig(_mc) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.ModuleConfig.html
                return None;
            }

            from_radio::PayloadVariant::QueueStatus(_qs) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.QueueStatus.html
                return None;
            }

            from_radio::PayloadVariant::XmodemPacket(_xmp) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.XModem.html
                return None;
            }

            from_radio::PayloadVariant::Metadata(_meta) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.DeviceMetadata.html
                return None;
            }

            _ => {
                return None;
            }
        }
    }

    None
}
