use std::collections::HashMap;

use meshtastic::protobufs::{
    mesh_packet::PayloadVariant, AirQualityMetrics, DeviceMetrics, EnvironmentMetrics, MeshPacket,
    MyNodeInfo, NeighborInfo, NodeInfo, Position, PowerMetrics, RouteDiscovery, Routing, User,
};
#[cfg(feature = "print-packets")]
use serde::{Deserialize, Serialize};

/// Local node type storing only the information we care about from nodeinfo table
pub struct Node {
    long_name: String,
    short_name: String,
    hw_model: i32,
    id: String,
    fake_msg_id: u8,
}

/// Declare a type alias for our hashmap of `node_ids` to numbers
pub type NodeFakePkts = HashMap<u32, Node>;

/// We need some state information for the serial vs mesh packet resolution of conflicts
/// It is a necessary evil unfortunately.
pub struct GatewayState {
    nodes: NodeFakePkts,
    biggest_fake: u8,
}

impl Default for GatewayState {
    ///
    ///
    /// # Arguments
    ///
    /// # Returns
    fn default() -> Self {
        GatewayState {
            nodes: NodeFakePkts::new(),
            biggest_fake: 0,
        }
    }
}

impl GatewayState {
    ///
    ///
    /// # Arguments
    /// *
    ///
    /// # Returns
    /// *
    #[must_use]
    pub fn new() -> GatewayState {
        // Stub this function for now, but in the future:
        // TODO - get the nodes and corresponding fake msg ids from local sqlite db
        GatewayState {
            nodes: HashMap::new(),
            biggest_fake: 0,
        }
    }
    // Lookup a Node's fake_msg_id

    ///
    ///
    /// # Arguments
    /// *
    ///
    /// # Returns
    /// *
    #[must_use]
    pub fn find_fake_id(&self, node_id: u32) -> Option<u8> {
        if let Some(f) = self.nodes.get(&node_id) {
            return Some(f.fake_msg_id);
        }
        None
    }
    // Insert a new node to the state

    ///
    ///
    /// # Arguments
    /// *
    ///
    /// # Returns
    /// *
    pub fn insert(&mut self, node_id: u32, user: User) -> bool {
        // Insert a new node if it does not already exist in the state
        if let std::collections::hash_map::Entry::Vacant(e) = self.nodes.entry(node_id) {
            let v = Node {
                long_name: user.long_name,
                short_name: user.short_name,
                hw_model: user.hw_model,
                id: user.id,
                fake_msg_id: self.biggest_fake,
            };
            self.biggest_fake += 1;
            e.insert(v);
            return true;
        } else if let Some(n) = self.nodes.get_mut(&node_id) {
            if (n.long_name != user.long_name
                || n.short_name != user.short_name
                || n.hw_model != user.hw_model)
                && n.id == user.id
            {
                // Update our local db
                n.long_name = user.long_name;
                n.short_name = user.short_name;
                n.hw_model = user.hw_model;
                // Increase the biggest_fake to reflect eventual change in db
                self.biggest_fake += 1;
                // Set the updated entry to use this new biggest_fake
                n.fake_msg_id = self.biggest_fake;
                self.biggest_fake += 1;
                return true;
            }
        }
        false
    }
}

// A lot of this file is derived off patterns established here: https://serde.rs/remote-derive.html
// But, the meshtastic crate has serde and serde_json as a feature flag
// So there has to be a better way to handle types. I will refactor heavily once I get MVP working

#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
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

#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum Telem {
    Device(DeviceMetrics),
    Environment(EnvironmentMetrics),
    AirQuality(AirQualityMetrics),
    Power(PowerMetrics),
}

#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
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
    ///
    ///
    /// # Arguments
    /// *
    ///
    /// # Returns
    /// *
    #[must_use]
    pub fn from_remote(def: MeshPacket) -> Mesh {
        Mesh {
            from: def.from,
            to: def.to,
            channel: def.channel,
            id: def.id,
            rx_time: def.rx_time,
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

#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
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
    ///
    ///
    /// # Arguments
    /// *
    ///
    /// # Returns
    /// *
    #[must_use]
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

#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct MyInfo {
    pub my_node_num: u32,
}

impl MyInfo {
    ///
    ///
    /// # Arguments
    /// *
    ///
    /// # Returns
    /// *
    #[must_use]
    pub fn from_remote(def: &MyNodeInfo) -> MyInfo {
        MyInfo {
            my_node_num: def.my_node_num,
        }
    }
}

#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum Pkt {
    Mesh(Mesh),
    NInfo(NInfo),
    MyNodeInfo(MyInfo),
}
