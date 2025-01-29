use meshtastic::protobufs::{mesh_packet::PayloadVariant, *};
use serde::{Deserialize, Serialize};

// A lot of this file is derived off patterns established here: https://serde.rs/remote-derive.html
// But, the meshtastic crate has serde and serde_json as a feature flag
// So there has to be a better way to handle types. I will refactor heavily once I get MVP working

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
    Device(DeviceMetrics),
    Environment(EnvironmentMetrics),
    AirQuality(AirQualityMetrics),
    Power(PowerMetrics),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
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
    //pub delayed: i32, -- Deprecated field
    pub via_mqtt: bool,
    pub hop_start: u32,
    pub payload_variant: Option<PayloadVariant>,
    pub payload: Option<Payload>,
}

// Provide a conversion to construct the local type.
impl Mesh {
    pub fn from_remote(def: MeshPacket) -> Mesh {
        Mesh {
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
            //delayed: def.delayed, -- Deprecated field
            via_mqtt: def.via_mqtt,
            hop_start: def.hop_start,
            payload_variant: def.payload_variant,
            payload: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NInfo {
    pub num: u32,
    pub user: Option<User>,
    pub position: Option<Position>,
    pub snr: f32,
    pub last_heard: u32,
    pub device_metrics: Option<DeviceMetrics>,
    pub channel: u32,
    pub via_mqtt: bool,
    pub hops_away: u32,
}

// Provide a conversion to construct the local type
// there's gotta be a better way, need to refactor
impl NInfo {
    pub fn from_remote(def: NodeInfo) -> NInfo {
        NInfo {
            num: def.num,
            user: def.user,
            position: def.position,
            snr: def.snr,
            last_heard: def.last_heard,
            device_metrics: def.device_metrics,
            channel: def.channel,
            via_mqtt: def.via_mqtt,
            hops_away: def.hops_away,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyInfo {
    pub my_node_num: u32,
}

impl MyInfo {
    pub fn from_remote(def: MyNodeInfo) -> MyInfo {
        MyInfo {
            my_node_num: def.my_node_num,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Pkt {
    Mesh(Mesh),
    NInfo(NInfo),
    MyNodeInfo(MyInfo),
}
