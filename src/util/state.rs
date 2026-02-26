use meshtastic::protobufs::User;
use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    fmt::Display,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU32, AtomicUsize, Ordering::Relaxed},
    },
};

/// Local node type storing only the information we care about from `NodeInfo` table
pub struct NodeMeta {
    /// Long name of the node
    long_name: String,
    /// Short name of the node
    short_name: String,
    /// HW Model enum
    hw_model: i32,
    /// Node id, the string hash `!dasf31`
    id: String,
    /// Number of received packets
    rx_count: Arc<AtomicUsize>,
}

/// Lightweight handle to a node's counter to eliminate lock holding in parts of the code
#[derive(Clone)]
pub struct RxCounter(Arc<AtomicUsize>);

impl RxCounter {
    #[inline]
    pub fn increment(&self) {
        self.0.fetch_add(1, Relaxed);
    }

    #[inline]
    pub fn load(&self) -> usize {
        self.0.load(Relaxed)
    }
}

/// We need some state information for the serial vs mesh packet resolution of conflicts
/// It is a necessary evil unfortunately.
pub struct GatewayState {
    /// Our hashmap of known nodes
    nodes: RwLock<HashMap<u32, NodeMeta>>,
    /// Connected node number
    serial_node: AtomicU32,
}

impl Default for GatewayState {
    /// Default constructor
    ///
    /// # Returns
    /// * An empty `GatewayState` struct
    fn default() -> Self {
        GatewayState {
            nodes: RwLock::new(HashMap::new()),
            serial_node: AtomicU32::new(0),
        }
    }
}

impl Display for GatewayState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Counts:\n")?;
        for (id, node) in self
            .nodes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
        {
            if *id == self.serial_node.load(Relaxed) {
                f.write_str("*serial\t")?;
            } else {
                f.write_str("\t\t")?;
            }
            writeln!(
                f,
                "{} ({}) {} - {} packets received",
                node.long_name,
                node.id,
                id,
                node.rx_count.load(Relaxed),
            )?;
        }
        Ok(())
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
        GatewayState {
            nodes: RwLock::new(HashMap::new()),
            serial_node: AtomicU32::new(0),
        }
    }

    /// Return a cloned `RxCounter` handle if the node is known.
    /// Allowing the caller to call `increment()` without holding locks
    pub fn get_counter(&self, node_id: u32) -> Option<RxCounter> {
        self.nodes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&node_id)
            .map(|n| RxCounter(Arc::clone(&n.rx_count)))
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
        self.nodes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .any(|n| n.rx_count.load(Relaxed) > 1)
    }

    /// Modify the `serial_node` connection
    ///
    /// # Arguments
    /// * `self` - Mutable self reference
    /// * `num` - The number of the serial node
    #[inline]
    pub fn set_serial_number(&self, num: u32) {
        self.serial_node.store(num, Relaxed);
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
    pub fn insert(&self, node_id: u32, user: &User) -> bool {
        match self
            .nodes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(node_id)
        {
            Vacant(e) => {
                e.insert(NodeMeta {
                    long_name: user.long_name.clone(),
                    short_name: user.short_name.clone(),
                    hw_model: user.hw_model,
                    id: user.id.clone(),
                    rx_count: Arc::new(AtomicUsize::new(0)),
                });
                true
            }
            Occupied(mut e) => {
                let n = e.get_mut();
                n.long_name.clone_from(&user.long_name);
                n.short_name.clone_from(&user.short_name);
                n.hw_model = user.hw_model;
                true
            }
        }
    }
}
