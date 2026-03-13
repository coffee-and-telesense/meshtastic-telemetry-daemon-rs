use crate::util::config::DEPLOYMENT_LOCATION;
use anyhow::{Error, Result};
use meshtastic::protobufs::User;
use sqlx::{Pool, Postgres};
use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    fmt::{self, Display, Formatter},
    sync::{
        PoisonError, RwLock,
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering::Relaxed},
    },
};

/// Local node type storing only the information we care about from `NodeInfo` table
#[derive(Debug)]
pub(crate) struct NodeMeta {
    /// Long name of the node
    long_name: String,
    /// Short name of the node
    short_name: String,
    /// Hardware Model enum
    hw_model: i32,
    /// Node id, the string hash `!dasf31`
    id: String,
    /// Number of received packets
    rx_count: AtomicUsize,
}

/// We need some state information for the serial vs mesh packet resolution of conflicts
/// It is a necessary evil unfortunately.
#[derive(Debug)]
pub(crate) struct GatewayState {
    /// Our hashmap of known nodes
    nodes: RwLock<HashMap<u32, NodeMeta>>,
    /// Connected node number
    serial_node: AtomicU32,
    /// Any packets received yet?
    any_recv: AtomicBool,
}

impl Default for GatewayState {
    /// Creates an empty state with no nodes and no serial connection.
    fn default() -> Self {
        GatewayState {
            nodes: RwLock::new(HashMap::new()),
            serial_node: AtomicU32::new(0),
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
            .peekable()
        {
            f.write_str("\n")?;

            if *id == self.serial_node.load(Relaxed) {
                f.write_str("*serial    ")?;
            } else {
                f.write_str("           ")?;
            }

            write!(
                f,
                "{:20} ({:9}) {:10} - {:12} packets received",
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
    /// Creates an empty gateway state with no known nodes.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Increment the `rx_count` of a given node
    pub(crate) fn increment_count(&self, node_id: u32) -> bool {
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
    pub(crate) fn set_serial_number(&self, num: u32) {
        self.serial_node.store(num, Relaxed);
    }

    /// Insert a new node into the state
    pub(crate) fn insert(&self, node_id: u32, user: &User) -> Result<()> {
        match self
            .nodes
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(node_id)
        {
            Vacant(e) => {
                e.insert(NodeMeta {
                    long_name: user.long_name.clone(),
                    short_name: user.short_name.clone(),
                    hw_model: user.hw_model,
                    id: user.id.clone(),
                    rx_count: AtomicUsize::new(0),
                });
                Ok(())
            }
            Occupied(mut e) => {
                let n = e.get_mut();
                if n.long_name == user.long_name
                    && n.short_name == user.short_name
                    && n.hw_model == user.hw_model
                {
                    return Err(Error::msg("Node already in state"));
                }
                n.long_name.clone_from(&user.long_name);
                n.short_name.clone_from(&user.short_name);
                n.hw_model = user.hw_model;
                Ok(())
            }
        }
    }

    /// Get nodes from preexisting `PostgreSQL` table
    pub(crate) async fn load_from_db(&self, db: &Pool<Postgres>) -> Result<()> {
        let rows = sqlx::query!(
            "
SELECT
    node_id,
    longname,
    shortname,
    hwmodel
FROM nodeinfo
WHERE
    deployment_location = $1
    AND longname IS NOT NULL
    AND shortname IS NOT NULL
    AND hwmodel IS NOT NULL
    ",
            DEPLOYMENT_LOCATION.get()
        )
        .fetch_all(db)
        .await?;

        for row in rows {
            // Reconstruct a minimal User and insert into GatewayState
            match self.insert(
                row.node_id.0,
                &User {
                    long_name: row.longname,
                    short_name: row.shortname,
                    hw_model: row.hwmodel,
                    id: format!("!{:08x}", row.node_id.0),
                    ..Default::default()
                },
            ) {
                Ok(()) => tracing::trace!("Added {} to GatewayState", row.node_id.0),
                Err(e) => tracing::warn!(%e),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;

    use super::*;

    fn test_user(long: &str, short: &str) -> User {
        User {
            id: String::from("!abc123"),
            long_name: String::from(long),
            short_name: String::from(short),
            hw_model: 1,
            ..Default::default()
        }
    }

    #[test]
    fn increment_unknown_node_returns_false() {
        let state = GatewayState::new();
        assert!(!state.increment_count(0xDEAD_BEEF));
    }

    #[test]
    fn increment_known_node_returns_true() -> Result<()> {
        let state = GatewayState::new();
        let user = test_user("TestNode", "TN");
        state.insert(1, &user)?;
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
        let user = test_user("TestNode", "TN");
        state.insert(1, &user)?;
        state.increment_count(1);
        assert!(state.any_recvd()); // first call: true
        assert!(!state.any_recvd()); // second call: reset to false
        Ok(())
    }

    #[test]
    fn insert_new_node_returns_true() -> Result<()> {
        let state = GatewayState::new();
        let user = test_user("NodeA", "NA");
        state.insert(1, &user)?;
        Ok(())
    }

    #[test]
    fn insert_same_data_returns_false() -> Result<()> {
        let state = GatewayState::new();
        let user = test_user("NodeA", "NA");
        state.insert(1, &user)?;
        assert!(state.insert(1, &user).is_err()); // no change
        Ok(())
    }

    #[test]
    fn insert_changed_data_returns_true() -> Result<()> {
        let state = GatewayState::new();
        state.insert(1, &test_user("NodeA", "NA"))?;
        state.insert(1, &test_user("NodeB", "NB"))?; // changed
        Ok(())
    }

    #[test]
    fn serial_number_roundtrip() -> Result<()> {
        let state = GatewayState::new();
        state.set_serial_number(42);
        // Verify via Display output containing "*serial"
        state.insert(42, &test_user("Serial", "SR"))?;
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
}
