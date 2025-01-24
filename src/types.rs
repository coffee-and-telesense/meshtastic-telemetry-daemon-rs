use meshtastic::protobufs::{mesh_packet::PayloadVariant, *};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Payload {
    // incomplete see: https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/index.html
    TextMessageApp(String),
    PositionApp(Position),
    NodeinfoApp(User),
    RoutingApp(Routing),
    TelemetryApp(Telem),
    TracerouteApp(RouteDiscovery),
    NeighborinfoApp(NeighborInfo),
    Max,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Telem {
    DeviceMetrics(DeviceMetrics),
    EnvironmentMetrics(EnvironmentMetrics),
    AirQualityMetrics(AirQualityMetrics),
    PowerMetrics(PowerMetrics),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshPkt {
    pub from: u32,
    pub to: u32,
    pub channel: u32,
    pub id: u32,
    pub rx_time: u32,
    pub rx_snr: f32,
    pub hop_limit: u32,
    pub want_ack: bool,
    pub priority: i32,
    pub rx_rssi: i32,
    pub delayed: i32,
    pub via_mqtt: bool,
    pub hop_start: u32,
    pub payload_variant: Option<PayloadVariant>,
    pub payload: Option<Payload>,
}

// Provide a conversion to construct the local type.
impl MeshPkt {
    pub fn from_remote(def: MeshPacket) -> MeshPkt {
        MeshPkt {
            from: def.from,
            to: def.to,
            channel: def.channel,
            id: def.id,
            rx_time: def.id,
            rx_snr: def.rx_snr,
            hop_limit: def.hop_limit,
            want_ack: def.want_ack,
            priority: def.priority,
            rx_rssi: def.rx_rssi,
            delayed: def.delayed,
            via_mqtt: def.via_mqtt,
            hop_start: def.hop_start,
            payload_variant: def.payload_variant,
            payload: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Pkt {
    MeshPkt(MeshPkt),
}
