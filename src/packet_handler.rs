use crate::types::MyInfo;

use super::types::{Mesh, NInfo, Payload, Pkt, Telem};
use log::info;
use meshtastic::protobufs::{
    from_radio, mesh_packet, telemetry, FromRadio, NeighborInfo, PortNum, Position, RouteDiscovery,
    Routing, User,
};
use meshtastic::Message;

// Shout-out to https://github.com/PeterGrace/meshtui for some of the code structure here
pub fn process_packet(packet: FromRadio) -> Option<Pkt> {
    if let Some(payload_v) = packet.clone().payload_variant {
        match payload_v {
            from_radio::PayloadVariant::Packet(pa) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.MeshPacket.html
                let mut pkt: Mesh = Mesh::from_remote(pa.clone());
                if let Some(payload) = pa.payload_variant {
                    match payload.clone() {
                        mesh_packet::PayloadVariant::Decoded(de) => {
                            match de.portnum() {
                                PortNum::PositionApp => {
                                    let data = Position::decode(de.payload.as_slice()).unwrap();
                                    pkt.payload_variant = None;
                                    pkt.payload = Some(Payload::PositionApp(data));
                                    return Some(Pkt::Mesh(pkt));
                                }
                                PortNum::TelemetryApp => {
                                    let data = meshtastic::protobufs::Telemetry::decode(
                                        de.payload.as_slice(),
                                    )
                                    .unwrap();
                                    pkt.payload_variant = None;
                                    if let Some(v) = data.variant {
                                        match v {
                                            telemetry::Variant::EnvironmentMetrics(env) => {
                                                pkt.payload = Some(Payload::TelemetryApp(
                                                    Telem::Environment(env),
                                                ));
                                                return Some(Pkt::Mesh(pkt));
                                            }
                                            telemetry::Variant::DeviceMetrics(dm) => {
                                                pkt.payload =
                                                    Some(Payload::TelemetryApp(Telem::Device(dm)));
                                                return Some(Pkt::Mesh(pkt));
                                            }
                                            telemetry::Variant::AirQualityMetrics(aqi) => {
                                                pkt.payload = Some(Payload::TelemetryApp(
                                                    Telem::AirQuality(aqi),
                                                ));
                                                return Some(Pkt::Mesh(pkt));
                                            }
                                            telemetry::Variant::PowerMetrics(pwm) => {
                                                pkt.payload =
                                                    Some(Payload::TelemetryApp(Telem::Power(pwm)));
                                                return Some(Pkt::Mesh(pkt));
                                            }
                                        }
                                    }
                                }
                                PortNum::NeighborinfoApp => {
                                    let data = NeighborInfo::decode(de.payload.as_slice()).unwrap();
                                    pkt.payload_variant = None;
                                    pkt.payload = Some(Payload::NeighborinfoApp(data));
                                    return Some(Pkt::Mesh(pkt));
                                }
                                PortNum::NodeinfoApp => {
                                    let data = User::decode(de.payload.as_slice()).unwrap();
                                    pkt.payload_variant = None;
                                    pkt.payload = Some(Payload::NodeinfoApp(data));
                                    return Some(Pkt::Mesh(pkt));
                                }
                                PortNum::RoutingApp => {
                                    let data = Routing::decode(de.payload.as_slice()).unwrap();
                                    pkt.payload_variant = None;
                                    pkt.payload = Some(Payload::RoutingApp(data));
                                    return Some(Pkt::Mesh(pkt));
                                }
                                PortNum::TracerouteApp => {
                                    let val_resp =
                                        RouteDiscovery::decode(de.payload.as_slice()).unwrap();
                                    pkt.payload_variant = None;
                                    pkt.payload = Some(Payload::TracerouteApp(val_resp));
                                    return Some(Pkt::Mesh(pkt));
                                }
                                PortNum::ReplyApp => {
                                    info!("We were just pinged.");
                                }
                                PortNum::TextMessageApp => {}
                                _ => {
                                    info!("{:#?}", de);
                                    return None;
                                } // PortNum::AdminApp => {}
                                  // PortNum::WaypointApp => {}

                                  // PortNum::PaxcounterApp => {}
                                  // PortNum::StoreForwardApp => {}
                                  // PortNum::RangeTestApp => {}
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
                let pkt = NInfo::from_remote(ni);
                return Some(Pkt::NInfo(pkt));
            }
            from_radio::PayloadVariant::Config(_c) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/config/enum.PayloadVariant.html
                return None;
            }
            from_radio::PayloadVariant::LogRecord(_lr) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.LogRecord.html
                return None;
            }
            from_radio::PayloadVariant::ConfigCompleteId(_ci) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/from_radio/enum.PayloadVariant.html#variant.ConfigCompleteId
                return None;
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
            from_radio::PayloadVariant::Channel(_c) => {
                // https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/struct.Channel.html
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
            from_radio::PayloadVariant::MqttClientProxyMessage(_mqtt) => {
                // We don't care and aren't using these
                return None;
            }
        }
    }
    None
}
