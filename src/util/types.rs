use meshtastic::protobufs::{
    AirQualityMetrics, DeviceMetrics, EnvironmentMetrics, ErrorMetrics, LocalStats, MeshPacket,
    MyNodeInfo, NeighborInfo, NodeInfo, Position, PowerMetrics, RouteDiscovery, Routing, User,
    mesh_packet::PayloadVariant,
};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection};
#[cfg(feature = "print-packets")]
use serde::Deserialize;
use serde::Serialize;

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

//TODO: A lot of this file is derived off patterns established here: https://serde.rs/remote-derive.html
// But, the meshtastic crate has serde and serde_json as a feature flag. So there has to be a better way
// to handle types. I will refactor heavily once I get MVP working

/// Payload enum with various possible payloads a packet could have
///
/// Incomplete see: <https://docs.rs/meshtastic/0.1.6/meshtastic/protobufs/index.html>
#[cfg_attr(feature = "print-packets", derive(Serialize, Deserialize))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
#[allow(unused)]
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
#[allow(unused)]
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
#[allow(unused)]
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
    pub last_rx_time: u32,
    pub node_broadcast_interval_secs: u32,
    pub num_packets_rx: u32,
    pub rssi: i32,
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
