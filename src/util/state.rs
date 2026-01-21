use log::{info, warn};
use meshtastic::protobufs::User;
use std::{borrow::Cow, collections::HashMap, fmt::Display};

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
    /// Connected node number
    serial_node: u32,
}

impl Default for GatewayState<'_> {
    /// Default constructor
    ///
    /// # Returns
    /// * An empty `GatewayState` struct
    fn default() -> Self {
        GatewayState {
            nodes: NodeFakePkts::new(),
            serial_node: 0, // Set to 0 by default on init
        }
    }
}

impl Display for GatewayState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Counts:\n")?;
        for (id, node) in &self.nodes {
            if *id == self.serial_node {
                f.write_str("*serial\t")?;
            } else {
                f.write_str("\t\t")?;
            }
            writeln!(
                f,
                "{} ({}) {} - {} packets received",
                node.long_name, node.id, id, node.rx_count
            )?;
        }

        Ok(())
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
            serial_node: 0, // Set to 0 by default on new
        }
    }

    /// Return `true` if any node in the local state contains an `rx_count` > 0
    ///
    /// # Arguments
    /// * `&self` - The `GatewayState` reference
    ///
    /// # Returns
    /// * `bool` - `true` if any node has an `rx_count` > 0, otherwise false
    #[inline]
    pub fn any_recvd(&self) -> bool {
        for node in self.nodes.values() {
            if node.rx_count > 1 {
                return true;
            }
        }
        false
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
    #[inline]
    pub fn increment_rx_count(&mut self, node_id: u32) {
        if let Some(f) = self.nodes.get_mut(&node_id) {
            f.rx_count += 1;
        }
    }

    /// Modify the `serial_node` connection
    ///
    /// # Arguments
    /// * `self` - Mutable self reference
    /// * `num` - The number of the serial node
    #[inline]
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
                rx_count: 0, // Initialize to 0 to not count any from nodedb?
            };
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
            return true;
        }
        false
    }
}
