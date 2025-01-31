use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use meshtastic::protobufs::{mesh_packet::PayloadVariant, *};
use serde::{Deserialize, Serialize};

pub struct Node {
    name: String,
    node_id: u32,
    fake_msg_id: u8,
}

// Declare a type alias for our hashmap of node_ids to numbers
pub type NodeFakePkts = HashMap<u32, Node>;

// We need some state information for the serial vs mesh packet resolution of conflicts
// It is a necessary evil unfortunately.
pub struct GatewayState {
    nodes: NodeFakePkts,
    biggest_fake: u8,
}

impl GatewayState {
    pub fn new() -> GatewayState {
        // Stub this function for now, but in the future:
        // TODO - get the nodes and corresponding fake msg ids from local sqlite db
        GatewayState {
            nodes: HashMap::new(),
            biggest_fake: 0,
        }
    }
    // Lookup a Node's fake_msg_id
    pub fn find_fake_id(&self, node_id: u32) -> Option<u8> {
        if let Some(f) = self.nodes.get(&node_id) {
            return Some(f.fake_msg_id);
        }
        None
    }
    // Insert a new node to the state
    pub fn insert(&mut self, node_id: u32, data: User) -> bool {
        // Insert a new node if it does not already exist in the state
        if !(self.nodes.contains_key(&node_id)) {
            let v = Node {
                name: data.id,
                node_id,
                fake_msg_id: self.biggest_fake,
            };
            self.biggest_fake += 1;
            self.nodes.insert(node_id, v);
            return true;
        }
        return false;
        // Otherwise:
    }
}

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
    pub hops_away: Option<u32>,
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
