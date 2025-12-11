#[cfg(feature = "debug")]
use log::{info, warn};
use meshtastic::protobufs::User;
use std::{borrow::Cow, collections::HashMap};

/// Local node type storing only the information we care about from nodeinfo table
pub struct Node<'a> {
    /// Long name of the node
    long_name: Cow<'a, str>,
    /// Short name of the node
    short_name: Cow<'a, str>,
    /// HW Model enum
    hw_model: i32,
    /// Node id, the string hash `!dasf31`
    id: Cow<'a, str>,
    /// Fake message id used in devicemetrics for this serial packet
    fake_msg_id: u8,
    /// Number of received packets
    rx_count: usize,
}

/// Declare a type alias for our hashmap of `node_ids` to numbers
pub type NodeFakePkts<'a> = HashMap<u32, Node<'a>>;

/// We need some state information for the serial vs mesh packet resolution of conflicts
/// It is a necessary evil unfortunately.
pub struct GatewayState<'a> {
    /// Our hashmap of known nodes
    nodes: NodeFakePkts<'a>,
    /// The biggest fake message id up to a `u8::MAX` of 255
    biggest_fake: u8,
    /// Connected node number
    serial_node: u32,
}

impl<'a> Default for GatewayState<'a> {
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

impl<'a> GatewayState<'a> {
    /// New `GatewayState` struct
    ///
    /// # Returns
    /// * An empty `GatewayState` struct
    #[must_use]
    pub fn new() -> GatewayState<'a> {
        // Stub this function for now, but in the future:
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
    pub fn format_rx_counts(&self) -> Cow<'_, str> {
        use chrono::Local;

        let now = Local::now();
        let mut rv: String = format!("{} - Counts:\n", now.format("%Y-%m-%d %H:%M:%S")).to_owned();
        for (id, node) in &self.nodes {
            rv.push_str(
                format!(
                    "{}{} ({}) {} - {} packets received\n",
                    if *id == self.serial_node {
                        "*serial\t"
                    } else {
                        "\t\t"
                    },
                    node.long_name,
                    node.id,
                    id,
                    node.rx_count
                )
                .as_str(),
            );
        }
        Cow::Owned(rv)
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
    pub fn insert(&mut self, node_id: u32, user: &User) -> bool {
        // Insert a new node if it does not already exist in the state
        if let std::collections::hash_map::Entry::Vacant(e) = self.nodes.entry(node_id) {
            info!("Inserting new node to the local state");
            let v = Node {
                long_name: Cow::Owned(user.long_name.as_str().to_owned()),
                short_name: Cow::Owned(user.short_name.as_str().to_owned()),
                hw_model: user.hw_model,
                id: Cow::Owned(user.id.as_str().to_owned()),
                fake_msg_id: self.biggest_fake,
                rx_count: 0, // Initialize to 0 to not count any from nodedb?
            };
            self.biggest_fake += 1;
            e.insert(v);
            return true;
        } else if let Some(n) = self.nodes.get_mut(&node_id)
            && (n.long_name != user.long_name
                || n.short_name != user.short_name
                || n.hw_model != user.hw_model)
            && n.id == user.id
        {
            warn!("Local state conflicts with nodeinfo received");
            // Update our local db
            n.long_name = Cow::Owned(user.long_name.as_str().to_owned());
            n.short_name = Cow::Owned(user.short_name.as_str().to_owned());
            n.hw_model = user.hw_model;
            // Increase the biggest_fake to reflect eventual change in db
            self.biggest_fake += 1;
            // Set the updated entry to use this new biggest_fake
            n.fake_msg_id = self.biggest_fake;
            self.biggest_fake += 1;
            return true;
        }
        false
    }
}
