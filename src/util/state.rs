//! Local state of the daemon

use crate::util::config::{DEPLOYMENT_LOCATION, PgPool};
use anyhow::{Context as _, Error, Result};
use diesel::{
    QueryableByName, RunQueryDsl as _,
    sql_types::{SmallInt, Text},
};
use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    fmt::{self, Display, Formatter},
    sync::{
        PoisonError, RwLock,
        atomic::{AtomicBool, AtomicU16, AtomicUsize, Ordering::Relaxed},
    },
};

/// Local node type storing only the information we care about from `NodeInfo` table
#[derive(Debug)]
pub(crate) struct NodeMeta {
    /// Name of the node
    name: String,
    /// Number of received packets
    rx_count: AtomicUsize,
}

/// We need some state information for the serial vs mesh packet resolution of conflicts
/// It is a necessary evil unfortunately.
#[derive(Debug)]
pub(crate) struct GatewayState {
    /// Our hashmap of known nodes
    nodes: RwLock<HashMap<u16, NodeMeta>>,
    /// Connected node number
    serial_node: AtomicU16,
    /// Any packets received yet?
    any_recv: AtomicBool,
}

impl Default for GatewayState {
    /// Creates an empty state with no nodes and no serial connection.
    fn default() -> Self {
        GatewayState {
            nodes: RwLock::new(HashMap::new()),
            serial_node: AtomicU16::new(0),
            any_recv: AtomicBool::new(false),
        }
    }
}

impl Display for GatewayState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Counts:")?;
        for (id, node) in self
            .nodes
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
        {
            f.write_str("\n")?;

            if *id == self.serial_node.load(Relaxed) {
                f.write_str("*serial    ")?;
            } else {
                f.write_str("           ")?;
            }

            write!(
                f,
                "{:20} {:10} - {:12} packets received",
                node.name,
                id,
                node.rx_count.load(Relaxed),
            )?;
        }
        Ok(())
    }
}

impl GatewayState {
    /// Creates an empty gateway state with no known nodes.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Increment the `rx_count` of a given node
    pub(crate) fn increment_count(&self, node_id: u16) -> bool {
        // Lock is only held for an atomic instruction, so it is short
        if let Some(n) = self
            .nodes
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&node_id)
        {
            n.rx_count.fetch_add(1, Relaxed);
            self.any_recv.store(true, Relaxed);
            return true;
        }
        false
    }

    /// Returns whether any packets were received since the last call, then resets the flag.
    #[inline]
    pub(crate) fn any_recvd(&self) -> bool {
        self.any_recv.swap(false, Relaxed)
    }

    /// Sets the node number of the locally-connected serial device.
    #[inline]
    pub(crate) fn set_serial_number(&self, num: u16) {
        self.serial_node.store(num, Relaxed);
    }

    /// Insert a new node into the state
    pub(crate) fn insert(&self, node_id: u16, name: &str) -> Result<()> {
        match self
            .nodes
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(node_id)
        {
            Vacant(e) => {
                e.insert(NodeMeta {
                    name: name.to_owned(),
                    rx_count: AtomicUsize::new(0),
                });
                Ok(())
            }
            Occupied(mut e) => {
                let n = e.get_mut();
                if n.name == name {
                    return Err(Error::msg("Node already in state"));
                }
                name.clone_into(&mut n.name);
                Ok(())
            }
        }
    }

    /// Get nodes from preexisting `PostgreSQL` table
    pub(crate) fn load_from_db(&self, db: &PgPool) -> Result<()> {
        let loc = DEPLOYMENT_LOCATION
            .get()
            .ok_or_else(|| Error::msg("DEPLOYMENT_LOCATION not initialized"))?;

        let mut con = db.get().context("Unable to get a connection from pool")?;

        let rows = diesel::sql_query(
            "
SELECT
    node_id,
    name
FROM nano_mesh_nodes
WHERE
    deployment_location = $1
    AND name IS NOT NULL
            ",
        )
        .bind::<Text, _>(loc.as_str())
        .load::<NodeInfoRow>(&mut con)?;

        for row in rows {
            // Reconstruct a minimal User and insert into GatewayState
            match self.insert(row.node_id.cast_unsigned(), &row.name) {
                Ok(()) => tracing::trace!("Added {} to GatewayState", row.node_id),
                Err(e) => tracing::warn!(%e),
            }
        }
        Ok(())
    }
}

/// Minimal projection of `nano_mesh_nodes` for state bootstrap
#[derive(QueryableByName, Debug)]
struct NodeInfoRow {
    #[diesel(sql_type = SmallInt)]
    node_id: i16,
    #[diesel(sql_type = Text)]
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;

    #[test]
    fn increment_unknown_node_returns_false() {
        let state = GatewayState::new();
        assert!(!state.increment_count(0xDEAD));
    }

    #[test]
    fn increment_known_node_returns_true() -> Result<()> {
        let state = GatewayState::new();
        state.insert(1, "TestNode")?;
        assert!(state.increment_count(1));
        Ok(())
    }

    #[test]
    fn any_recvd_false_when_no_packets() {
        let state = GatewayState::new();
        assert!(!state.any_recvd());
    }

    #[test]
    fn any_recvd_true_after_increment_then_resets() -> Result<()> {
        let state = GatewayState::new();
        state.insert(1, "TestNode")?;
        state.increment_count(1);
        assert!(state.any_recvd()); // first call: true
        assert!(!state.any_recvd()); // second call: reset to false
        Ok(())
    }

    #[test]
    fn insert_new_node_returns_true() -> Result<()> {
        let state = GatewayState::new();
        state.insert(1, "NodeA")?;
        Ok(())
    }

    #[test]
    fn insert_same_data_returns_false() -> Result<()> {
        let state = GatewayState::new();
        state.insert(1, "NodeA")?;
        assert!(state.insert(1, "NodeA").is_err()); // no change
        Ok(())
    }

    #[test]
    fn insert_changed_data_returns_true() -> Result<()> {
        let state = GatewayState::new();
        state.insert(1, "NodeA")?;
        state.insert(1, "NodeB")?;
        Ok(())
    }

    #[test]
    fn serial_number_roundtrip() -> Result<()> {
        let state = GatewayState::new();
        state.set_serial_number(42);
        // Verify via Display output containing "*serial"
        state.insert(42, "Serial")?;
        let display = format!("{state}");
        assert!(display.contains("*serial"));
        Ok(())
    }

    #[test]
    fn increment_does_not_set_flag_for_unknown_node() {
        let state = GatewayState::new();
        state.increment_count(999); // unknown
        assert!(!state.any_recvd()); // should still be false
    }

    #[test]
    fn display_formats_multiple_nodes_correctly() -> Result<()> {
        let state = GatewayState::new();
        state.insert(1, "Node1")?;
        state.insert(2, "Node2")?;

        // Set Node 1 as the serial node, and simulate Node 2 receiving 5 packets
        state.set_serial_number(1);
        for _ in 0..5 {
            state.increment_count(2);
        }

        let output = format!("{state}");

        // Assert the header and specific text formatting
        assert!(output.contains("Counts:"));
        assert!(output.contains("*serial"));
        assert!(output.contains("Node1"));
        assert!(output.contains("Node2"));
        assert!(output.contains("5 packets received")); // Node 2
        assert!(output.contains("0 packets received")); // Node 1
        Ok(())
    }

    #[test]
    fn concurrent_increments_are_thread_safe() -> Result<()> {
        use std::sync::Arc;
        use std::thread;

        let state = Arc::new(GatewayState::new());
        state.insert(100, "Concurrent")?;

        // Spawn 10 threads, each increments the counter 100 times
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let s = Arc::clone(&state);
                thread::spawn(move || {
                    for _ in 0..100 {
                        s.increment_count(100);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join()
                .map_err(|e| Error::msg(format!("thread panicked {e:?}")))?;
        }

        // We should have exactly 1000 packets counted without race conditions
        let output = format!("{state}");
        assert!(output.contains("1000 packets received"));
        assert!(state.any_recvd());
        Ok(())
    }
}
