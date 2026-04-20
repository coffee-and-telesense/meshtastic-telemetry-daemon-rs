// @generated automatically by Diesel CLI.

use diesel::{allow_tables_to_appear_in_same_query, joinable, table};

table! {
    nano_mesh_nodes (node_id) {
        node_id -> Int2,
        name -> Nullable<Text>,
        deployment_location -> Text,
    }
}

table! {
    node_stats (id) {
        id -> Int8,
        node_id -> Int2,
        epoch -> Int8,
        received_at -> Timestamp,
        reboot_count -> Int2,
        tx_fail -> Int4,
        rx_drop -> Int4,
        rx_useful -> Int4,
        rx_overlap -> Int4,
        queue_full -> Int4,
        rx_bad -> Int4,
        num_online_nodes -> Int2,
        num_total_nodes -> Int2,
        channel_util -> Float4,
        air_util_tx -> Float4,
        deployment_location -> Text,
    }
}

table! {
    sensor_info (id) {
        id -> Int4,
        sensor_type -> Int2,
        sensor_id_type -> Text,
        accurate_op_range_min -> Nullable<Int4>,
        accurate_op_range_max -> Nullable<Int4>,
        abs_accuracy_tolerance -> Nullable<Text>,
        measure_range_min -> Nullable<Int4>,
        measure_range_max -> Nullable<Int4>,
        resolution -> Nullable<Numeric>,
        rms_noise -> Nullable<Numeric>,
        response_time -> Nullable<Int4>,
        year_drift -> Nullable<Numeric>,
        unit -> Nullable<Text>,
    }
}

table! {
    sensor_readings (id) {
        id -> Int8,
        node_id -> Int2,
        epoch -> Int8,
        received_at -> Timestamp,
        sensor_id -> Int2,
        kind -> Int2,
        value -> Float4,
        deployment_location -> Text,
    }
}

joinable!(node_stats -> nano_mesh_nodes (node_id));
joinable!(sensor_readings -> nano_mesh_nodes (node_id));

allow_tables_to_appear_in_same_query!(nano_mesh_nodes, node_stats, sensor_info, sensor_readings,);
