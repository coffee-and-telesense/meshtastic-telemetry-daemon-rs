use super::types::{Payload, Pkt};
use crate::types::{Mesh, Telem};
use crate::{entities::*, types::NInfo};
use anyhow::{Context, Result};
use chrono::Utc;
use sea_orm::prelude::Expr;
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait,
};
use sea_orm::{ColumnTrait, QueryFilter, QuerySelect};

pub async fn update_metrics(
    db: &DatabaseConnection,
    packet: Pkt,
    fake_msg_id: Option<u32>,
    dep_loc: &String,
) -> Result<u32> {
    match packet {
        Pkt::Mesh(mp) => {
            if let Some(p) = mp.payload.clone() {
                match p {
                    Payload::TelemetryApp(t) => match t {
                        Telem::Environment(data) => {
                            match (environmentmetrics::ActiveModel {
                                msg_id: ActiveValue::Set(mp.id),
                                node_id: ActiveValue::Set(mp.from),
                                time: ActiveValue::Set(Utc::now().naive_utc()),
                                relative_humidity: ActiveValue::Set(data.relative_humidity),
                                tempurature: ActiveValue::Set(data.temperature),
                                barometric_pressure: ActiveValue::Set(data.barometric_pressure),
                                gas_resistance: ActiveValue::Set(data.gas_resistance),
                                iaq: ActiveValue::Set(data.iaq),
                                wind_direction: ActiveValue::Set(data.wind_direction),
                                wind_gust: ActiveValue::Set(data.wind_gust),
                                wind_speed: ActiveValue::Set(data.wind_speed),
                                wind_lull: ActiveValue::Set(data.wind_lull),
                                rainfall_1h: ActiveValue::Set(data.rainfall_1h),
                                rainfall_24h: ActiveValue::Set(data.rainfall_24h),
                            }
                            .insert(db)
                            .await
                            .with_context(|| {
                                "Failed to insert environment metrics row from mesh payload"
                            })) {
                                Ok(_) => Ok(1),
                                Err(e) => {
                                    error!("{:#}", e);
                                    Ok(0)
                                }
                            }
                        }

                        Telem::AirQuality(data) => {
                            match (airqualitymetrics::ActiveModel {
                                msg_id: ActiveValue::Set(mp.id),
                                node_id: ActiveValue::Set(mp.from),
                                time: ActiveValue::Set(Utc::now().naive_utc()),
                                pm10standard: ActiveValue::Set(data.pm10_standard),
                                pm25standard: ActiveValue::Set(data.pm25_standard),
                                pm100standard: ActiveValue::Set(data.pm100_standard),
                                pm10environmental: ActiveValue::Set(data.pm10_environmental),
                                pm25environmental: ActiveValue::Set(data.pm25_environmental),
                                pm100environmental: ActiveValue::Set(data.pm100_environmental),
                                particles03um: ActiveValue::Set(data.particles_03um),
                                particles05um: ActiveValue::Set(data.particles_05um),
                                particles10um: ActiveValue::Set(data.particles_10um),
                                particles25um: ActiveValue::Set(data.particles_25um),
                                particles50um: ActiveValue::Set(data.particles_50um),
                                particles100um: ActiveValue::Set(data.particles_100um),
                                co2: ActiveValue::Set(data.co2),
                            }
                            .insert(db)
                            .await
                            .with_context(|| {
                                "Failed to insert air quality metrics row from mesh payload"
                            })) {
                                Ok(_) => Ok(1),
                                Err(e) => {
                                    error!("{:#}", e);
                                    Ok(0)
                                }
                            }
                        }

                        Telem::Device(data) => {
                            match (devicemetrics::ActiveModel {
                                msg_id: ActiveValue::Set(mp.id),
                                node_id: ActiveValue::Set(mp.from),
                                time: ActiveValue::Set(Utc::now().naive_utc()),
                                battery_levels: ActiveValue::Set(data.battery_level),
                                voltage: ActiveValue::Set(data.voltage),
                                channelutil: ActiveValue::Set(data.channel_utilization),
                                airutil: ActiveValue::Set(data.air_util_tx),
                                latitude: ActiveValue::NotSet, //TODO: investigate default values
                                longitude: ActiveValue::NotSet,
                                longname: ActiveValue::NotSet,
                                shortname: ActiveValue::NotSet,
                                hwmodel: ActiveValue::NotSet,
                            }
                            .insert(db)
                            .await
                            .with_context(|| {
                                "Failed to insert device metrics row from mesh payload"
                            })) {
                                Ok(_) => Ok(1),
                                Err(e) => {
                                    error!("{:#}", e);
                                    Ok(0)
                                }
                            }
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
                        return node_info_conflict(ni, Some(mp), db, fake_msg_id, dep_loc).await;
                    }

                    Payload::PositionApp(data) => {
                        // Updates the position for a given node id that is included in the
                        // packet sent from the mesh
                        match (devicemetrics::ActiveModel {
                            msg_id: ActiveValue::Set(mp.id),
                            node_id: ActiveValue::Set(mp.from),
                            time: ActiveValue::Set(Utc::now().naive_utc()),
                            latitude: ActiveValue::Set(data.latitude_i),
                            longitude: ActiveValue::Set(data.longitude_i),
                            battery_levels: ActiveValue::NotSet, //TODO: investigate default values
                            voltage: ActiveValue::NotSet,
                            channelutil: ActiveValue::NotSet,
                            airutil: ActiveValue::NotSet,
                            longname: ActiveValue::NotSet,
                            shortname: ActiveValue::NotSet,
                            hwmodel: ActiveValue::NotSet,
                        }
                        .insert(db)
                        .await
                        .with_context(|| "Failed to insert device metrics row from mesh payload"))
                        {
                            Ok(_) => Ok(1),
                            Err(e) => {
                                error!("{:#}", e);
                                Ok(0)
                            }
                        }
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
                        // request traceroutes), NeighborinfoApp (adjacency matrix data and
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
            // This is a NodeInfo payload from serial but not received over the mesh, meaning it is
            // the output from our initial serial connection when we receive a dump of all the
            // nodes in the nodedb of the connected Meshtastic node that is our network bridge.
            // These packets possibly have user info, in which case we treat it the same as those
            // from the mesh and pass it to the conflict resoltuion function.
            return node_info_conflict(ni, None, db, fake_msg_id, dep_loc).await;
        }

        _ => {
            // Only other type implemented at this time is MyNodeInfo, which just provides our
            // node's ID number which could be used for the managing of local state if needed,
            // but we do not need it when making database updates for now
            Ok(0)
        }
    }
}

async fn node_info_conflict(
    ni: NInfo,
    pkt: Option<Mesh>,
    db: &DatabaseConnection,
    fake_msg_id: Option<u32>,
    dep_loc: &String,
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
            // Our local state already has the updated node entry from the bridge (either serial or
            // mesh), so we just need to determine if we should insert or update an entry in the
            // database.
            let curr = nodeinfo::Entity::find_by_id(ni.num)
                .one(db)
                .await
                .with_context(|| "Could not get entry in db, connection error?");
            match curr {
                Ok(c) => {
                    if let Some(u) = c {
                        // Found an entry in the db, check if any nodeinfo columns need to be
                        // updated, and if so update them.

                        if u.shortname != user.short_name
                            || u.longname != user.long_name
                            || u.hwmodel != user.hw_model
                            || &u.deployment_location != dep_loc
                        {
                            // Update the nodeinfo row values, node_id remains the same
                            let mut upd_ni: nodeinfo::ActiveModel = u.into();
                            upd_ni.longname = ActiveValue::Set(user.long_name.clone());
                            upd_ni.shortname = ActiveValue::Set(user.short_name.clone());
                            upd_ni.hwmodel = ActiveValue::Set(user.hw_model);
                            upd_ni.deployment_location = ActiveValue::Set(dep_loc.to_string());

                            // Get the largest fake_msg_id from the table, then add one
                            let new_fake_msg_id = devicemetrics::Entity::find()
                                .filter(devicemetrics::Column::MsgId.lt(u8::MAX))
                                .having(Expr::col(devicemetrics::Column::MsgId).max())
                                .select_only()
                                .column(devicemetrics::Column::MsgId)
                                .one(db)
                                .await
                                .expect("Failed to connect to the database")
                                .expect("Failed to find max fake_msg_id < 255")
                                .msg_id
                                + 1;

                            // Create updated devicemetrics row
                            let upd_dm = devicemetrics::ActiveModel {
                                msg_id: ActiveValue::Set(new_fake_msg_id),
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
                                format!("Failed to update nodeinfo row entry for {}", mp.from)
                            }) {
                                Ok(_) => {
                                    row_insert_count += 1;
                                }
                                Err(e) => {
                                    error!("{:#}", e);
                                }
                            }
                            // Try inserting the new devicemetrics row
                            match devicemetrics::Entity::insert(upd_dm)
                                .on_conflict(OnConflict::column(devicemetrics::Column::MsgId))
                                .do_nothing()
                                .exec(db)
                                .await
                                .with_context(|| {
                                    "Failed to insert devicemetrics row for updated nodeinfo from mesh payload"
                                }) {
                                Ok(_) => {
                                    row_insert_count += 1;
                                }
                                Err(e) => {
                                    error!("{:#}", e);
                                }
                            }
                        }
                    } else {
                        // No entry in db, so we insert a new unheard node into both devicemetrics
                        // and the nodeinfo table.
                        return new_node(ni, db, fake_msg_id, dep_loc).await;
                    }
                }
                Err(e) => {
                    error!("{:#}", e);
                }
            }
        } else {
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

async fn new_node(
    ni: NInfo,
    db: &DatabaseConnection,
    fake_msg_id: Option<u32>,
    dep_loc: &String,
) -> Result<u32> {
    let mut row_insert_count = 0;
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

    // Try inserting devicemetrics row
    match devicemetrics::Entity::insert(dm)
        .on_conflict(OnConflict::columns([
            devicemetrics::Column::MsgId,
            devicemetrics::Column::NodeId,
            devicemetrics::Column::Time,
            devicemetrics::Column::BatteryLevels,
            devicemetrics::Column::Voltage,
            devicemetrics::Column::Channelutil,
            devicemetrics::Column::Airutil,
            devicemetrics::Column::Latitude,
            devicemetrics::Column::Longitude,
            devicemetrics::Column::Shortname,
            devicemetrics::Column::Longname,
            devicemetrics::Column::Hwmodel,
        ]))
        .do_nothing()
        .exec(db)
        .await
        .with_context(|| "Failed to insert device metrics row from serial payload")
    {
        Ok(_) => {
            row_insert_count += 1;
        }
        Err(e) => {
            error!("{:#}", e);
        }
    }

    // Try inserting nodeinfo row
    match nodeinfo::Entity::insert(ninfo)
        .on_conflict(OnConflict::columns([
            nodeinfo::Column::NodeId,
            nodeinfo::Column::Longname,
            nodeinfo::Column::Shortname,
            nodeinfo::Column::Hwmodel,
            nodeinfo::Column::DeploymentLocation,
        ]))
        .do_nothing()
        .exec(db)
        .await
        .with_context(|| "Failed to insert node info row from serial payload")
    {
        Ok(_) => {
            row_insert_count += 1;
        }
        Err(e) => {
            error!("{:#}", e);
        }
    }
    Ok(row_insert_count)
}
