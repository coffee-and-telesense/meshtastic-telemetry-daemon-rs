use crate::{
    dto::entities::{
        airqualitymetrics, devicemetrics, environmentmetrics, errormetrics, localstats,
        neighborinfo, nodeinfo,
    },
    util::state::GatewayState,
    util::types::{Mesh, NInfo, Names, Payload, Pkt, Telem},
};
use anyhow::{Context, Result};
use chrono::Utc;
#[cfg(feature = "debug")]
use log::{error, info};
use meshtastic::protobufs::User;
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, sea_query::OnConflict,
};
use std::sync::{Arc, Mutex};

/// Update metrics in database
///
/// # Arguments
/// * `db` - The database connection pool from sea-orm
/// * `packet` - A Pkt that has been processed by `packet_handler()`
/// * `fake_msg_id` - A fake message id number for serial packets
/// * `dep_loc` - The deployment location string read from the config file
///
/// # Returns
/// * Result type with number of rows inserted or none
///
/// # Errors
///
pub(crate) async fn update_metrics(
    db: &DatabaseConnection,
    packet: &Pkt,
    fake_msg_id: Option<u32>,
    dep_loc: &str,
) -> Result<u32> {
    match packet {
        Pkt::Mesh(mp) => {
            if let Some(p) = mp.payload.clone() {
                match p {
                    Payload::TelemetryApp(t) => match t {
                        Telem::Environment(data) => {
                            environmentmetrics::Model::create_model(mp, data)
                                .insert_row(db)
                                .await
                        }

                        Telem::AirQuality(data) => {
                            airqualitymetrics::Model::create_model(mp, data)
                                .insert_row(db)
                                .await
                        }

                        Telem::Device(data) => {
                            devicemetrics::Model::create_dm_model(mp, data)
                                .insert_row(db)
                                .await
                        }

                        Telem::Local(data) => {
                            localstats::Model::create_model(mp, data)
                                .insert_row(db)
                                .await
                        }

                        Telem::Error(data) => {
                            errormetrics::Model::create_model(mp, data)
                                .insert_row(db)
                                .await
                        }

                        Telem::Power(_data) => {
                            // Not sure what we want to do with these metrics
                            Ok(0)
                        }
                    },

                    Payload::NodeinfoApp(data) => {
                        // NodeinfoApp payloads over the mesh indicate an advertisement for a node
                        // that has joined the channel, since packet_handler() guarantees to only
                        // pass this function Pkts from nodes on our configured channel. We cannot
                        // tell if this is a new node, an update to a node we know, or a routine
                        // advertisement from an already known node. In response, we pass the
                        // relevant variables to another function to determine which case this is
                        // and return how many rows were inserted here to return back to main.
                        // First we need to create a NInfo packet type from the user data payload
                        // in order to pass it along.
                        info!("Node Info from Mesh");
                        let ni = NInfo {
                            num: mp.from,
                            user: Some(data),
                            position: None,
                            snr: mp.rx_snr,
                            last_heard: mp.rx_time, // Dummy value for now
                            device_metrics: None,
                            channel: mp.channel,
                            via_mqtt: mp.via_mqtt,
                            hops_away: None,
                        };
                        return node_info_conflict(ni, Some(mp.clone()), db, Some(mp.id), dep_loc)
                            .await;
                    }

                    Payload::PositionApp(data) => {
                        // Updates the position for a given node id that is included in the
                        // packet sent from the mesh
                        devicemetrics::Model {
                            msg_id: mp.id,
                            node_id: mp.from,
                            time: Utc::now().naive_utc(),
                            latitude: data.latitude_i,
                            longitude: data.longitude_i,
                            battery_levels: None, //TODO: investigate default values
                            voltage: None,
                            channelutil: None,
                            airutil: None,
                            longname: None,
                            shortname: None,
                            hwmodel: None,
                        }
                        .insert_row(db)
                        .await
                    }

                    Payload::NeighborinfoApp(data) => {
                        neighborinfo::Model::create_model(mp, data)
                            .insert_row(db)
                            .await
                    }

                    _ => {
                        // Other payloads are unhandled, but there are some that may be of
                        // interest to us. Namely: TextMessageApp, RemoteHardwareApp (if we
                        // ever use the GPIO pins and module of the same name), RoutingApp
                        // (telemetry about routes discovered and failed routes),
                        // AdminMessageApp (this is usually config R/W), IPTunnelApp (this just
                        // routes IP packets through Meshtastic network, probably not
                        // interesting to us), PaxcounterApp (reports on BLE/WIFI devices
                        // seen), SerialApp (this is an interface for sending/receiving packets
                        // over a serial connection, probably useless to us), StoreForwardApp
                        // (this has some interesting data about history, stats, and heartbeats),
                        // RangeTestApp (probably not useful to us), TracerouteApp (seems to
                        // provide the same data as the RoutingApp but when users explicitly
                        // request Traceroutes), NeighborinfoApp (adjacency matrix data and
                        // other stuff like last heard, might be good for us)
                        //
                        // Some of these may also be provided outside of Mesh packets, so we
                        // would need to handle them similar to NInfo below. Need to
                        // investigate this further, but I suspect it is nested in the
                        // ModuleConfig response over serial.
                        Ok(0)
                    }
                }
            } else {
                // No payload provided by the packet, just return 0 inserts to the db
                Ok(0)
            }
        }

        Pkt::NInfo(ni) => {
            info!("Node info from serial");
            // This is a NodeInfo payload from serial but not received over the mesh, meaning it is
            // the output from our initial serial connection when we receive a dump of all the
            // nodes in the nodedb of the connected Meshtastic node that is our network bridge.
            // These packets possibly have user info, in which case we treat it the same as those
            // from the mesh and pass it to the conflict resoltuion function.
            return node_info_conflict(ni.clone(), None, db, fake_msg_id, dep_loc).await;
        }

        Pkt::MyNodeInfo(_) => {
            // Only other type implemented at this time is MyNodeInfo, which just provides our
            // node's ID number which could be used for the managing of local state if needed,
            // but we do not need it when making database updates for now
            Ok(0)
        }
    }
}

/// Proactively insert nodeinfo rows for telemetry data received
///
/// # Arguments
/// * `pkt` - A `Mesh` packet reference
/// * `db` - A database connection
/// * `dep_loc` - The deployment location string from the config
/// * `state` - A `GatewayState` ARC clone
///
/// # Returns
/// * Result with the number of rows inserted/updated
///
/// # Panics
/// * Will panic if it fails to acquire a lock on the `GatewayState`
/// * Will panic if a `fake_node_mg_id` cannot be found in the `GatewayState` before inserts
pub(crate) async fn proactive_ninfo_insert(
    pkt: &Mesh,
    db: &DatabaseConnection,
    dep_loc: &str,
    state: Arc<Mutex<GatewayState<'_>>>,
) -> Result<u32> {
    // Check if we already know this node
    if state
        .lock()
        .expect("Failed to acquire lock for GatewayState in packet_handler()")
        .find_fake_id(pkt.from)
        .is_none()
    {
        // We do not know this node, so it likely is not in postgres.
        // First add it to our Gateway state
        let fake_user = User {
            id: String::new(),
            long_name: String::new(),
            short_name: String::new(),
            #[allow(deprecated)]
            macaddr: vec![],
            hw_model: -1,
            is_licensed: false,
            role: -1,
            public_key: vec![],
        };
        state
            .lock()
            .expect("Failed to acquire lock for GatewayState in proactive_node_insert()")
            .insert(pkt.from, &fake_user);
        // Now we insert into postgres
        let fake_msg_id = state
            .lock()
            .expect("Failed to acquire lock for GatewayState in proactive_node_insert()")
            .find_fake_id(pkt.from)
            .expect("No fake_msg_id provided in proactive_node_insert()");
        let fake_ni = NInfo {
            num: pkt.from,
            user: Some(fake_user),
            position: None,
            snr: -1.0,
            last_heard: 0,
            device_metrics: None,
            channel: 0,
            via_mqtt: false,
            hops_away: None,
        };
        // Insert it as if it came over serial
        info!("Proactively inserting a new node info and device metrics row");
        node_info_conflict(fake_ni, None, db, Some(u32::from(fake_msg_id)), dep_loc).await
    } else {
        // We already know about it so return 0 row changes
        info!("Node already known, skipping proactive inserts");
        Ok(0)
    }
}

/// Node info conflict resolver
///
/// This function resolves possible conflicts between `NodeInfo` received over Mesh or over serial
/// connection from the nodedb of the connected device.
///
/// # Arguments
/// * `ni` - A `NInfo` packet
/// * `pkt` - A possible `Mesh` packet
/// * `db` - The `DatabaseConnection` ref
/// * `fake_msg_id` - A possible fake message id for a devicemetric row insert
/// * `dep_loc` - The deployment location from the config file
///
/// # Returns
/// * Result with the number of rows inserted/updated
async fn node_info_conflict(
    ni: NInfo,
    pkt: Option<Mesh>,
    db: &DatabaseConnection,
    fake_msg_id: Option<u32>,
    dep_loc: &str,
) -> Result<u32> {
    let mut row_insert_count = 0;

    if let Some(mp) = pkt {
        // We have a mesh payload, so we need to determine if there are conflicts in the user data
        // to determine if we:
        // 1. Insert: a new fake devicemetrics to indicate nodeinfo column change and update the
        //    nodeinfo columns with new values
        // 2. Only update devicemetrics with values from the packet for the conflict free
        //    information like snr and other values

        if let Some(user) = ni.user.as_ref() {
            // Case 1
            // Our local state already has the updated node entry from the bridge (either serial or
            // mesh), so we just need to determine if we should insert or update an entry in the
            // database.
            let found = nodeinfo::Entity::find_by_id(ni.num)
                .one(db)
                .await
                .with_context(|| {
                    format!(
                        "Could not get entry in {} db, connection error?",
                        db.get_db_name()
                    )
                })?;

            match found {
                Some(u) => {
                    // Found an entry in the db, check if any nodeinfo columns need to be
                    // updated, and if so update them.
                    info!("Found a possible conflicting nodeinfo entry, possibly updating");

                    if u.shortname != user.short_name
                        || u.longname != user.long_name
                        || u.hwmodel != user.hw_model
                        || u.deployment_location != dep_loc
                    {
                        // Update the nodeinfo row values, node_id remains the same
                        let mut upd_ni: nodeinfo::ActiveModel = u.into();
                        upd_ni.longname = ActiveValue::Set(user.long_name.clone());
                        upd_ni.shortname = ActiveValue::Set(user.short_name.clone());
                        upd_ni.hwmodel = ActiveValue::Set(user.hw_model);
                        upd_ni.deployment_location = ActiveValue::Set(dep_loc.to_string());

                        // Create updated devicemetrics row
                        let upd_dm = devicemetrics::ActiveModel {
                            msg_id: ActiveValue::Set(fake_msg_id.expect("No fake_msg_id provided")),
                            node_id: ActiveValue::Set(mp.from),
                            time: ActiveValue::Set(Utc::now().naive_utc()),
                            longname: ActiveValue::Set(Some(user.long_name.clone())),
                            shortname: ActiveValue::Set(Some(user.short_name.clone())),
                            hwmodel: ActiveValue::Set(Some(user.hw_model)),
                            battery_levels: ActiveValue::NotSet,
                            voltage: ActiveValue::NotSet,
                            channelutil: ActiveValue::NotSet,
                            airutil: ActiveValue::NotSet,
                            latitude: ActiveValue::NotSet,
                            longitude: ActiveValue::NotSet,
                        };

                        // Try updating the nodeinfo row
                        match upd_ni.update(db).await.with_context(|| {
                            format!(
                                "Failed to update nodeinfo row entry for {} into {} db",
                                mp.from,
                                db.get_db_name()
                            )
                        }) {
                            Ok(_) => {
                                row_insert_count += 1;
                                info!("Updated the nodeinfo row");
                            }
                            Err(e) => {
                                error!("{e:#}");
                            }
                        }

                        // Try inserting the new devicemetrics row
                        match devicemetrics::Entity::insert(upd_dm)
                                .on_conflict(OnConflict::column(devicemetrics::Column::MsgId))
                                .do_nothing()
                                .exec(db)
                                .await
                                .with_context(|| {
                                    format!(
                                        "Failed to insert devicemetrics row for updated nodeinfo from mesh payload into {} db",
                                        db.get_db_name()
                                    )
                                }) {
                                Ok(_) => {
                                    row_insert_count += 1;
                                }
                                Err(e) => {
                                    error!("{e:#}");
                                }
                            }
                    }
                }
                None => {
                    // No entry in db, so we insert a new unheard node into both devicemetrics
                    // and the nodeinfo table using the mesh packet id as our fake_msg_id
                    return new_node(ni, db, Some(mp.id), dep_loc).await;
                }
            }
        } else {
            // Case 2
            // Here we only update rows with relevant updated data
            // Since the NInfo passed from the Mesh has no position or devicemetrics and we do not
            // track some of the other values in the Pkt::Mesh type, we do nothing here
            info!("No db transaction on payload without relevant data");
        }
    } else {
        // We have a serial payload, so we need to insert a fake devicemetrics with the data in the
        // payload, and we need to potentially insert a node to the nodeinfo table but if either
        // already exists then we do not do anything on conflicts.
        return new_node(ni, db, fake_msg_id, dep_loc).await;
    }

    Ok(row_insert_count)
}

/// Insert a new node's info into nodeinfo and devicemetrics
///
/// Hadles the trivial cases when the device is brand-new to us and has no entries in the database
///
/// # Arguments
/// * `ni` - `NInfo` packet
/// * `db` - The database connection
/// * `fake_msg_id` - The fake message id for the devicemetrics row
/// * `dep_loc` - The deployment location string from the configuration
///
/// # Returns
/// * Result with how many rows were inserted
///
/// # Panics
/// This function will panic if no `fake_msg_id` was provided or if the user data like longname are
/// None values.
async fn new_node(
    ni: NInfo,
    db: &DatabaseConnection,
    fake_msg_id: Option<u32>,
    dep_loc: &str,
) -> Result<u32> {
    let mut row_insert_count = 0;

    // Create device metrics model
    let dm = devicemetrics::ActiveModel {
        msg_id: ActiveValue::Set(fake_msg_id.expect("No fake_msg_id provided to db action")),
        node_id: ActiveValue::Set(ni.num),
        time: ActiveValue::Set(Utc::now().naive_utc()),
        battery_levels: ActiveValue::Set(ni.device_metrics.as_ref().and_then(|m| m.battery_level)),
        voltage: ActiveValue::Set(ni.device_metrics.as_ref().and_then(|m| m.voltage)),
        channelutil: ActiveValue::Set(
            ni.device_metrics
                .as_ref()
                .and_then(|m| m.channel_utilization),
        ),
        airutil: ActiveValue::Set(ni.device_metrics.as_ref().and_then(|m| m.air_util_tx)),
        latitude: ActiveValue::Set(ni.position.as_ref().and_then(|p| p.latitude_i)),
        longitude: ActiveValue::Set(ni.position.as_ref().and_then(|p| p.longitude_i)),
        longname: ActiveValue::Set(ni.user.as_ref().map(|u| u.long_name.clone())),
        shortname: ActiveValue::Set(ni.user.as_ref().map(|u| u.short_name.clone())),
        hwmodel: ActiveValue::Set(ni.user.as_ref().map(|u| u.hw_model)),
    };

    // Create node info model
    let ninfo = nodeinfo::ActiveModel {
        node_id: ActiveValue::Set(ni.num),
        longname: ActiveValue::Set(
            ni.user
                .as_ref()
                .map(|u| u.long_name.clone())
                .expect("Longname not provided by serial packet"),
        ),
        shortname: ActiveValue::Set(
            ni.user
                .as_ref()
                .map(|u| u.short_name.clone())
                .expect("Shortname not provided by serial packet"),
        ),
        hwmodel: ActiveValue::Set(
            ni.user
                .as_ref()
                .map(|u| u.hw_model)
                .expect("Hwmodel not provided by serial packet"),
        ),
        deployment_location: ActiveValue::Set(dep_loc.to_string()),
    };

    // Try inserting nodeinfo row
    match nodeinfo::Entity::insert(ninfo)
        .on_conflict(
            OnConflict::column(nodeinfo::Column::NodeId)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await
        .with_context(|| {
            format!(
                "Failed to insert node info row from serial payload into {} db for new_node()",
                db.get_db_name()
            )
        }) {
        Ok(_) => {
            row_insert_count += 1;
        }
        Err(e) => {
            // These are expected to error out, so lower the log level
            info!("{e:#}");
        }
    }

    // Try inserting devicemetrics row
    match devicemetrics::Entity::insert(dm)
        .on_conflict(
            OnConflict::column(devicemetrics::Column::MsgId)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await
        .with_context(|| {
            format!(
                "Failed to insert device metrics row from serial payload into {} db for new_node()",
                db.get_db_name()
            )
        }) {
        Ok(_) => {
            row_insert_count += 1;
        }
        Err(e) => {
            // These are expected to error out, so lower the log level
            info!("{e:#}");
        }
    }
    Ok(row_insert_count)
}
