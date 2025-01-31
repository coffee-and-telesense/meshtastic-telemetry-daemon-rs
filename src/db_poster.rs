use super::types::{Mesh, Payload, Pkt};
use crate::types::{NInfo, Telem};
use chrono::Utc;
use meshtastic::protobufs::{AirQualityMetrics, DeviceMetrics, EnvironmentMetrics, Position, User};
use tokio_postgres::Error;

pub trait AddData {
    // Generic update metrics handler
    async fn update_metrics(&self, packet: Pkt, fake_msg_id: Option<u32>) -> Result<u64, Error>;

    // Update metrics from received Meshtastic Packets
    async fn add_environmental_metrics(
        &self,
        pkt: Mesh,
        data: EnvironmentMetrics,
    ) -> Result<u64, Error>;
    async fn add_air_quality_metrics(
        &self,
        pkt: Mesh,
        data: AirQualityMetrics,
    ) -> Result<u64, Error>;
    async fn add_device_metrics(&self, pkt: Mesh, data: DeviceMetrics) -> Result<u64, Error>;
    async fn update_user_info(&self, pkt: Mesh, data: User) -> Result<u64, Error>;
    async fn add_node_position(&self, pkt: Mesh, data: Position) -> Result<u64, Error>;

    // Update metrics from serial connection
    async fn update_device_met(
        &self,
        pkt: NInfo,
        data: DeviceMetrics,
        fake_msg_id: Option<u32>,
    ) -> Result<u64, Error>;
    async fn update_user_pkt(
        &self,
        pkt: NInfo,
        data: User,
        fake_msg_id: Option<u32>,
    ) -> Result<u64, Error>;
    async fn update_user(&self, node_id: u32, data: User) -> Result<u64, Error>;
    async fn update_node_pos(
        &self,
        pkt: NInfo,
        data: Position,
        fake_msg_id: Option<u32>,
    ) -> Result<u64, Error>;
}

impl AddData for tokio_postgres::Client {
    async fn update_metrics(&self, packet: Pkt, fake_msg_id: Option<u32>) -> Result<u64, Error> {
        match packet {
            Pkt::Mesh(mp) => {
                if let Some(p) = mp.payload.clone() {
                    match p {
                        Payload::TelemetryApp(t) => match t {
                            Telem::Environment(data) => {
                                return self.add_environmental_metrics(mp, data).await;
                            }
                            Telem::AirQuality(data) => {
                                return self.add_air_quality_metrics(mp, data).await;
                            }
                            Telem::Device(data) => {
                                return self.add_device_metrics(mp, data).await;
                            }
                            Telem::Power(_data) => {
                                // Not sure what we want to do with these metrics
                                Ok(0)
                            }
                        },
                        Payload::NodeinfoApp(data) => {
                            // Only updates user information
                            return self.update_user_info(mp, data).await;
                        }
                        Payload::PositionApp(data) => {
                            // Updates the position for a given node id that is included in the
                            // packet sent from the mesh
                            return self.add_node_position(mp, data).await;
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
                // This is the NodeInfo that is communicated directly over serial to our process
                // It does not have a MeshPkt (Meshtastic Packet from LoRa)
                // So we will need to decide what to do with its User, Position, and DeviceMetrics data
                // We could provide a dummy packet, or we could just update the 'static' table to
                // reflect the node db on the device
                let mut rv = 0;
                if let Some(data) = ni.clone().device_metrics {
                    // need to change to handle errors
                    rv += self
                        .update_device_met(ni.clone(), data, fake_msg_id)
                        .await
                        .unwrap_or(0);
                }
                if let Some(data) = ni.clone().user {
                    // need to change to handle errors
                    rv += self.update_user(ni.num, data.clone()).await.unwrap_or(0);
                    rv += self
                        .update_user_pkt(ni.clone(), data, fake_msg_id)
                        .await
                        .unwrap_or(0);
                }
                if let Some(data) = ni.clone().position {
                    // need to change to handle errors
                    rv += self
                        .update_node_pos(ni, data, fake_msg_id)
                        .await
                        .unwrap_or(0);
                }
                Ok(rv)
            }
            _ => {
                // Only other type implemented at this time is MyNodeInfo, which just provides our
                // node's ID number which could be used for the managing of local state if needed,
                // but we do not need it when making database updates for now
                Ok(0)
            }
        }
    }
    // The following update from Meshtastic packets:
    async fn add_environmental_metrics(
        &self,
        pkt: Mesh,
        data: EnvironmentMetrics,
    ) -> Result<u64, Error> {
        let insert_query = "
        INSERT INTO environmentmetrics (
            msg_id, node_id, time, tempurature, relative_humidity, barometric_pressure, 
            gas_resistance, iaq
        ) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8 )";
        let datetime = Utc::now().naive_utc();
        // send to db
        self.execute(
            insert_query,
            &[
                &pkt.id,
                &pkt.from,
                &datetime,
                &data.temperature,
                &data.relative_humidity,
                &data.barometric_pressure,
                &data.gas_resistance,
                &data.iaq,
            ],
        )
        .await
    }
    async fn add_air_quality_metrics(
        &self,
        pkt: Mesh,
        data: AirQualityMetrics,
    ) -> Result<u64, Error> {
        //TODO
        Ok(0)
    }
    async fn add_device_metrics(&self, pkt: Mesh, data: DeviceMetrics) -> Result<u64, Error> {
        let datetime = Utc::now().naive_utc();
        let insert = "
        INSERT INTO devicemetrics (
            msg_id, node_id, time, battery_levels, voltage, channelUtil, airUtil
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ";
        self.execute(
            insert,
            &[
                &pkt.id,
                &pkt.from,
                &datetime,
                &data.battery_level,
                &data.voltage,
                &data.channel_utilization,
                &data.air_util_tx,
            ],
        )
        .await
    }
    async fn update_user_info(&self, pkt: Mesh, data: User) -> Result<u64, Error> {
        Ok(0)
    }
    async fn add_node_position(&self, pkt: Mesh, data: Position) -> Result<u64, Error> {
        let datetime = Utc::now().naive_utc();
        let insert = "
        INSERT INTO devicemetrics (
            msg_id, node_id, time, latitude, longitude
        )
        VALUES ($1, $2, $3, $4, $5)
        ";
        self.execute(
            insert,
            &[
                &pkt.id,
                &pkt.from,
                &datetime,
                &data.latitude_i,
                &data.longitude_i,
            ],
        )
        .await
    }
    // The following update from the serial connection
    async fn update_device_met(
        &self,
        pkt: NInfo,
        data: DeviceMetrics,
        fake_msg_id: Option<u32>,
    ) -> Result<u64, Error> {
        Ok(0)
    }
    async fn update_user_pkt(
        &self,
        pkt: NInfo,
        data: User,
        fake_msg_id: Option<u32>,
    ) -> Result<u64, Error> {
        let datetime = Utc::now().naive_utc();
        let insert_dev = "
        INSERT INTO devicemetrics (
            msg_id, node_id, time, longName, shortName, hwModel
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ";
        self.execute(
            insert_dev,
            &[
                &fake_msg_id,
                &pkt.num,
                &datetime,
                &data.long_name,
                &data.short_name,
                &data.hw_model,
            ],
        )
        .await
    }
    async fn update_user(&self, node_id: u32, data: User) -> Result<u64, Error> {
        let insert_node = "
            INSERT INTO nodeinfo (
                node_id, longName, shortName, hwModel, deployment_location
            )
            VALUES ($1, $2, $3, $4, $5)
        ";
        let tmp: String = "testing".to_string();
        self.execute(
            insert_node,
            &[
                &node_id,
                &data.long_name,
                &data.short_name,
                &data.hw_model,
                &tmp,
            ],
        )
        .await
    }
    async fn update_node_pos(
        &self,
        pkt: NInfo,
        data: Position,
        fake_msg_id: Option<u32>,
    ) -> Result<u64, Error> {
        Ok(0)
    }
}
