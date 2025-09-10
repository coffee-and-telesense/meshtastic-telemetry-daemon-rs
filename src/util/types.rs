#[cfg(feature = "debug")]
use log::{info, warn};
use meshtastic::protobufs::{
    AirQualityMetrics, DeviceMetrics, EnvironmentMetrics, ErrorMetrics, LocalStats, MeshPacket,
    MyNodeInfo, NeighborInfo, NodeInfo, Position, PowerMetrics, RouteDiscovery, Routing, User,
    mesh_packet::PayloadVariant,
};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection};
#[cfg(feature = "print-packets")]
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

/// Trait to proxy enum matching to database names
pub trait Names {
    fn get_db_name<'a>(self) -> &'a str;
}

impl Names for &DatabaseConnection {
    /// Get a database's name (postgres, sqlite, mysql) from the db connection
    ///
    /// # Arguments
    /// * `self` - A `&DatabaseConnection` type for a given db
    ///
    /// # Returns
    /// * `&str` - A constant borrowed string with a lifetime limited to the return of this
    ///   function
    fn get_db_name<'a>(self) -> &'a str {
        match self.get_database_backend() {
            DatabaseBackend::MySql => "mysql",
            DatabaseBackend::Postgres => "postgres",
            DatabaseBackend::Sqlite => "sqlite",
        }
    }
}

/// Local node type storing only the information we care about from nodeinfo table
pub struct Node {
    /// Long name of the node
    long_name: String,
    /// Short name of the node
    short_name: String,
    /// HW Model enum
    hw_model: i32,
    /// Node id, the string hash `!dasf31`
    id: String,
    /// Fake message id used in devicemetrics for this serial packet
    fake_msg_id: u8,
    /// Number of received packets
    rx_count: usize,
}

/// Declare a type alias for our hashmap of `node_ids` to numbers
pub type NodeFakePkts = HashMap<u32, Node>;

/// We need some state information for the serial vs mesh packet resolution of conflicts
/// It is a necessary evil unfortunately.
pub struct GatewayState {
    /// Our hashmap of known nodes
    nodes: NodeFakePkts,
    /// The biggest fake message id up to a `u8::MAX` of 255
    biggest_fake: u8,
    /// Connected node number
    serial_node: u32,
}

impl Default for GatewayState {
    /// Default constructor
    ///
    /// # Returns
    /// * An empty `GatewayState` struct
    fn default() -> Self {
        GatewayState {
            nodes: NodeFakePkts::new(),
            biggest_fake: 0,
            serial_node: 0, // Set to 0 by default on init
        }
    }
}

impl GatewayState {
    /// New `GatewayState` struct
    ///
    /// # Returns
    /// * An empty `GatewayState` struct
    #[must_use]
    pub fn new() -> GatewayState {
        // Stub this function for now, but in the future:
        // TODO - get the nodes and corresponding fake msg ids from local sqlite db
        GatewayState {
            nodes: NodeFakePkts::new(),
            biggest_fake: 0,
            serial_node: 0, // Set to 0 by default on new
        }
    }

    /// Lookup a node's fake message id
    ///
    /// # Arguments
    /// * `self` - Operates on the `GatewayState` struct
    /// * `node_id` - The `u32` id in the `from` field of packets
    ///
    /// # Returns
    /// * `Option<u8>` - The fake message id if it exists or None
    #[must_use]
    pub fn find_fake_id(&self, node_id: u32) -> Option<u8> {
        if let Some(f) = self.nodes.get(&node_id) {
            return Some(f.fake_msg_id);
        }
        None
    }

    /// Increment the `rx_count` for a local state seen node (debug builds only)
    ///
    /// # Arguments
    /// * `self` - Operates on the `GatewayState` struct
    /// * `node_id` - The `u32` id in the `from` field of packets
    ///
    /// # Side effects/state changes
    /// * Increments the `rx_count` entry of the corresponding `node_id`
    #[cfg(feature = "debug")]
    pub fn increment_rx_count(&mut self, node_id: u32) {
        if let Some(f) = self.nodes.get_mut(&node_id) {
            f.rx_count += 1;
        }
    }

    /// Format a message of the received packet counts
    ///
    /// # Arguments
    /// * `self` - Operates on the `GatewayState` struct
    ///
    /// # Returns
    /// * `String` - String of the node counts to print
    #[cfg(feature = "debug")]
    pub fn format_rx_counts(&self) -> String {
        use chrono::Local;

        let now = Local::now();
        let mut rv: String =
            format!("{} - Counts:\n", now.format("%Y-%m-%d %H:%M:%S - ")).to_owned();
        for (id, node) in &self.nodes {
            rv.push_str(
                format!(
                    "{}{} ({}) {} - {} packets received\n",
                    if *id == self.serial_node {
                        "*serial "
                    } else {
                        "\t"
                    },
                    node.long_name,
                    node.id,
                    id,
                    node.rx_count
                )
                .as_str(),
            );
        }
        rv
    }

    /// Modify the `serial_node` connection
    ///
    /// # Arguments
    /// * `self` - Mutable self reference
    /// * `num` - The number of the serial node
    pub fn set_serial_number(&mut self, num: u32) {
        self.serial_node = num;
    }

    /// Insert a new node into the state
    ///
    /// Possibly updating our local state if any of the `Node` struct items have changed
    ///
    /// # Arguments
    /// * `self` - Operates on the `GatewayState` struct
    /// * `node_id` - The `u32` id in the `from` field of packets
    /// * `user` - The `User` type payload from packets
    ///
    /// # Returns
    /// * `bool` - True if inserted/updated, false if not
    pub fn insert(&mut self, node_id: u32, user: User) -> bool {
        // Insert a new node if it does not already exist in the state
        if let std::collections::hash_map::Entry::Vacant(e) = self.nodes.entry(node_id) {
            info!("Inserting new node to the local state");
            let v = Node {
                long_name: user.long_name,
                short_name: user.short_name,
                hw_model: user.hw_model,
                id: user.id,
                fake_msg_id: self.biggest_fake,
                rx_count: 0, // Initialize to 0 to not count any from nodedb?
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
                warn!("Local state conflicts with nodeinfo received");
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

//TODO: A lot of this file is derived off patterns established here: https://serde.rs/remote-derive.html
// But, the meshtastic crate has serde and serde_json as a feature flag. So there has to be a better way
// to handle types. I will refactor heavily once I get MVP working

/// Payload enum with various possible payloads a packet could have
///
/// Incomplete see: <https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/index.html>
#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum Payload {
    /// Text messages
    TextMessageApp(String),
    /// GPS position locations
    PositionApp(Position),
    /// Node info advertisement
    NodeinfoApp(User),
    /// Routing information
    RoutingApp(Routing),
    /// Telemetry for sensors
    TelemetryApp(Telem),
    /// Traceroute packets
    TracerouteApp(RouteDiscovery),
    /// Neighbor info packets
    NeighborinfoApp(NeighborInfo),
    /// Maximum payload value, does nothing right now
    Max,
}

/// Telemetry enum of the various telemetry types
#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum Telem {
    /// Device metrics like battery level
    Device(DeviceMetrics),
    /// Environment metrics like temperature
    Environment(EnvironmentMetrics),
    /// Air quality metrics like CO2
    AirQuality(AirQualityMetrics),
    /// Power metrics like voltage
    Power(PowerMetrics),
    /// Local stats
    Local(LocalStats),
    /// Error metrics
    Error(ErrorMetrics),
}

impl Telem {
    /// Match the telemetry type to a str of the tablename
    ///
    /// # Arguments
    /// * `self` - A `Telem` enum
    ///
    /// # Returns
    /// * `&str` - A string of the tablename
    pub fn telem_name(&self) -> &str {
        match self {
            Telem::Device(_device_metrics) => "DeviceMetrics",
            Telem::Environment(_environment_metrics) => "EnvironmentalMetrics",
            Telem::AirQuality(_air_quality_metrics) => "AirQualityMetrics",
            Telem::Power(_power_metrics) => "PowerMetrics",
            Telem::Local(_local_stats) => "LocalStats",
            Telem::Error(_error_metrics) => "ErrorMetrics",
        }
    }
}

/// Mesh packet structure that aliases the Meshtastic library's `MeshPacket`
///
/// I need to figure out how to unify my types
#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct Mesh {
    /// What node id `u32` is this from?
    pub from: u32,
    /// What node id is this for?
    pub to: u32,
    /// What channel (index) did we receive this on?
    pub channel: u32,
    /// What's the message id?
    pub id: u32,
    /// When did we receive it as an epoch timestamp?
    pub rx_time: u32,
    /// What was the signal to noise ratio when we received it?
    pub rx_snr: f32,
    /// What is the hop limit of the packet?
    pub hop_limit: u32,
    /// Did the packet want an ACK response?
    pub want_ack: bool,
    /// What was the priority of the packet on the mesh?
    pub priority: i32,
    /// What was the received signal strength indicator?
    pub rx_rssi: i32,
    /// Did this packet originate on MQTT?
    pub via_mqtt: bool,
    /// Where did this packet start from?
    pub hop_start: u32,
    /// What payload variant does the packet have?
    pub payload_variant: Option<PayloadVariant>,
    /// What is the decoded payload of the packet after `process_packet()`
    pub payload: Option<Payload>,
}

/// Provide a conversion to construct the local type from the remote type
impl Mesh {
    /// Convert the remote Meshtastic `MeshPacket` to our `Mesh`
    ///
    /// # Arguments
    /// * `def` - Meshtastic `MeshPacket` instance
    ///
    /// # Returns
    /// * `Mesh` - `Mesh` variant of `Pkt` enum
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
            via_mqtt: def.via_mqtt,
            hop_start: def.hop_start,
            payload_variant: def.payload_variant,
            payload: None,
        }
    }
    /// Get the tablename from the Telemetry variant
    ///
    /// # Arguments
    /// * `self` - A `Mesh` structure
    ///
    /// # Returns
    /// * `&str` - A tablename as a string
    pub fn match_tablename(&self) -> &str {
        if let Some(p) = &self.payload {
            match p {
                Payload::TextMessageApp(_message) => "Message",
                Payload::PositionApp(_position) => "Position",
                Payload::NodeinfoApp(_user) => "User/NodeInfo",
                Payload::RoutingApp(_routing) => "Routing",
                Payload::TelemetryApp(telem) => telem.telem_name(),
                Payload::TracerouteApp(_route_discovery) => "RouteDiscovery",
                Payload::NeighborinfoApp(_neighbor_info) => "NeighborInfo",
                Payload::Max => "Max",
            }
        } else {
            ""
        }
    }
}

/// `NInfo` packet structure that aliases the Meshtastic library's `NodeInfo`
#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct NInfo {
    /// The nodedb entry's node id `u32`
    pub num: u32,
    /// The user info like longname associated with this entry
    pub user: Option<User>,
    /// The position payload associated with this entry
    pub position: Option<Position>,
    /// The signal to noise ratio of this entry
    pub snr: f32,
    /// The timestamp for when we last heard this node
    pub last_heard: u32,
    /// The device metrics payload of this entry
    pub device_metrics: Option<DeviceMetrics>,
    /// What channel we heard this node on
    pub channel: u32,
    /// Whether this node came to our attention via MQTT or not
    pub via_mqtt: bool,
    /// How many hops away is this node?
    pub hops_away: Option<u32>,
}

/// Provide a conversion to construct the local type from the remote type
impl NInfo {
    /// Convert the remote Meshtastic `NodeInfo` to our `NInfo`
    ///
    /// # Arguments
    /// * `def` - Meshtastic `NodeInfo` instance
    ///
    /// # Returns
    /// * `NInfo` - `NInfo` variant of `Pkt` enum
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

/// `MyInfo` packet structure that aliases the Meshtastic library's `MyNodeInfo`
#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct MyInfo {
    /// My node `u32` id
    pub my_node_num: u32,
}

/// Provide a conversion to construct the local type from the remote type
impl MyInfo {
    /// Convert the remote Meshtastic `MyNodeInfo` to our `MyInfo`
    ///
    /// # Arguments
    /// * `def` - Meshtastic `MyNodeInfo` instance
    ///
    /// # Returns
    /// * `MyInfo` - `MyInfo` variant of `Pkt` enum
    #[must_use]
    pub fn from_remote(def: &MyNodeInfo) -> MyInfo {
        MyInfo {
            my_node_num: def.my_node_num,
        }
    }
}

/// `Pkt` enum of the various packet types
#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum Pkt {
    /// `Mesh` packets, or those arriving on `LoRa`
    Mesh(Mesh),
    /// `NInfo` packets, or the serial packets of nodes in the nodedb
    NInfo(NInfo),
    /// `MyNodeInfo` packets arriving on serial
    MyNodeInfo(MyInfo),
}

/// Neighbor struct to make JSON for `NeighborInfo` table
#[cfg_attr(feature = "print-packets", derive(Deserialize))]
#[derive(Serialize)]
pub struct Neighbor {
    pub node_id: u32,
    pub snr: f32,
}

/// Error reason counts
#[cfg_attr(feature = "print-packets", derive(Deserialize))]
#[derive(Serialize)]
pub struct ErrorCounts {
    pub no_routes: Option<u32>,
    pub naks: Option<u32>,
    pub timeouts: Option<u32>,
    pub max_retransmits: Option<u32>,
    pub no_channels: Option<u32>,
    pub too_large: Option<u32>,
}
