-- NanoMesh schema migration
-- Replaces Meshtastic protobuf tables with embedded-nano-mesh packet types

-- Node registry
CREATE TABLE IF NOT EXISTS nano_mesh_nodes (
    node_id SMALLINT PRIMARY KEY,
    name TEXT,
    deployment_location TEXT NOT NULL
);

-- One row per Measurement in a SensorPacket
-- SCD30 produces: Temperature(1), Humidity(2), Co2(6)
-- BME688 produces: Temperature(1), Humidity(2), Pressure(3), GasResistance(4), Iaq(5)
CREATE TABLE IF NOT EXISTS sensor_readings (
    id BIGSERIAL PRIMARY KEY,
    node_id SMALLINT NOT NULL REFERENCES nano_mesh_nodes(node_id),
    epoch BIGINT NOT NULL,
    received_at TIMESTAMP NOT NULL DEFAULT now(),
    sensor_id SMALLINT NOT NULL,
    kind SMALLINT NOT NULL,
    value REAL NOT NULL,
    deployment_location TEXT NOT NULL,
    UNIQUE (node_id, epoch, sensor_id, kind)
);

CREATE INDEX IF NOT EXISTS sensor_readings_node_epoch
    ON sensor_readings (node_id, epoch DESC);

CREATE INDEX IF NOT EXISTS sensor_readings_kind
    ON sensor_readings (kind);

-- One row per NodeStatsPacket
CREATE TABLE IF NOT EXISTS node_stats (
    id BIGSERIAL PRIMARY KEY,
    node_id SMALLINT NOT NULL REFERENCES nano_mesh_nodes(node_id),
    epoch BIGINT NOT NULL,
    received_at TIMESTAMP NOT NULL DEFAULT now(),
    reboot_count SMALLINT NOT NULL,
    tx_fail INTEGER NOT NULL,
    rx_drop INTEGER NOT NULL,
    rx_useful INTEGER NOT NULL,
    rx_overlap INTEGER NOT NULL,
    queue_full INTEGER NOT NULL,
    rx_bad INTEGER NOT NULL,
    num_online_nodes SMALLINT NOT NULL,
    num_total_nodes SMALLINT NOT NULL,
    channel_util REAL NOT NULL,
    air_util_tx REAL NOT NULL,
    deployment_location TEXT NOT NULL,
    UNIQUE (node_id, epoch)
);

CREATE INDEX IF NOT EXISTS node_stats_node_epoch
    ON node_stats (node_id, epoch DESC);

-- Sensor metadata
-- sensor_type maps to MeasurementKind discriminant (u8)
CREATE TABLE IF NOT EXISTS sensor_info (
    id SERIAL PRIMARY KEY,
    sensor_type SMALLINT NOT NULL,
    sensor_id_type TEXT NOT NULL,
    accurate_op_range_min INTEGER,
    accurate_op_range_max INTEGER,
    abs_accuracy_tolerance TEXT,
    measure_range_min INTEGER,
    measure_range_max INTEGER,
    resolution NUMERIC,
    rms_noise NUMERIC,
    response_time INTEGER,
    year_drift NUMERIC,
    unit TEXT
);

-- Accuracy functions
CREATE OR REPLACE FUNCTION scd30_temp_accuracy(T NUMERIC)
RETURNS NUMERIC AS $$
BEGIN
    RETURN 0.4 + 0.023 * (T - 25);
END;
$$ LANGUAGE PLpgSQL;

CREATE OR REPLACE FUNCTION bme688_temp_accuracy(T NUMERIC)
RETURNS NUMERIC AS $$
BEGIN
    IF T >= 0 AND T <= 65 THEN
        RETURN 0.5;
    ELSIF T > -40 AND T < 0 THEN
        RETURN 1.0;
    ELSE
        RETURN NULL;
    END IF;
END;
$$ LANGUAGE PLpgSQL;

CREATE OR REPLACE FUNCTION bme280_temp_accuracy(T NUMERIC)
RETURNS NUMERIC AS $$
BEGIN
    IF T <= -20 THEN
        RETURN 1.5;
    ELSIF T > -20 AND T <= 0 THEN
        RETURN 1.5 - 0.0125 * (T + 20);
    ELSIF T > 0 AND T <= 65 THEN
        RETURN 0.5;
    ELSE
        RETURN NULL;
    END IF;
END;
$$ LANGUAGE PLpgSQL;

CREATE OR REPLACE FUNCTION scd30_co2_accuracy(C NUMERIC, T NUMERIC)
RETURNS NUMERIC AS $$
BEGIN
    RETURN 30 + (0.03 * C) + 2.5 * (T - 25);
END;
$$ LANGUAGE PLpgSQL;

CREATE OR REPLACE FUNCTION bme280_pres_accuracy(P NUMERIC, T NUMERIC)
RETURNS NUMERIC AS $$
BEGIN
    IF T > -20 AND T <= 0 THEN
        RETURN 1.7;
    ELSIF T > 0 AND P <= 1100 THEN
        RETURN 1.0;
    ELSIF T > 25 AND T <= 40 AND P > 1100 AND P <= 1250 THEN
        RETURN 1.5;
    ELSE
        RETURN NULL;
    END IF;
END;
$$ LANGUAGE PLpgSQL;

CREATE OR REPLACE FUNCTION bme280_pres_noise(T NUMERIC)
RETURNS NUMERIC AS $$
BEGIN
    IF T <= -10 THEN
        RETURN 0.04125;
    ELSIF T >= 75 THEN
        RETURN 0.03135;
    ELSE
        RETURN NULL;
    END IF;
END;
$$ LANGUAGE PLpgSQL;

-- SCD30: kind=1 (Temperature), kind=2 (Humidity), kind=6 (Co2)
INSERT INTO sensor_info (sensor_type, sensor_id_type, accurate_op_range_min, accurate_op_range_max, measure_range_min, measure_range_max, abs_accuracy_tolerance, resolution, rms_noise, response_time, year_drift, unit)
VALUES (1, 'scd30_temperature', 0, 50, -40, 70, '±0.4 + 0.023*(T-25)', 0.00208, 0.1, 10, 0.03, '°C');

INSERT INTO sensor_info (sensor_type, sensor_id_type, accurate_op_range_min, accurate_op_range_max, measure_range_min, measure_range_max, abs_accuracy_tolerance, resolution, rms_noise, response_time, year_drift, unit)
VALUES (2, 'scd30_humidity', 0, 95, 0, 100, '±3', 0.0002441, 0.1, 8, 0.25, '% RH');

INSERT INTO sensor_info (sensor_type, sensor_id_type, accurate_op_range_min, accurate_op_range_max, measure_range_min, measure_range_max, rms_noise, response_time, unit)
VALUES (6, 'scd30_co2', 400, 10000, 0, 40000, 10, 20, 'ppm');

-- BME688: kind=1 (Temperature), kind=2 (Humidity), kind=3 (Pressure), kind=4 (GasResistance), kind=5 (Iaq)
INSERT INTO sensor_info (sensor_type, sensor_id_type, accurate_op_range_min, accurate_op_range_max, measure_range_min, measure_range_max, resolution, rms_noise, response_time, unit)
VALUES (1, 'bme688_temperature', 0, 65, -40, 85, 0.01, 0.005, 1, '°C');

INSERT INTO sensor_info (sensor_type, sensor_id_type, accurate_op_range_min, accurate_op_range_max, measure_range_min, measure_range_max, abs_accuracy_tolerance, resolution, rms_noise, response_time, year_drift, unit)
VALUES (2, 'bme688_humidity', 10, 90, 0, 100, '±3', 0.008, 0.02, 1, 0.5, '% RH');

INSERT INTO sensor_info (sensor_type, sensor_id_type, accurate_op_range_min, accurate_op_range_max, measure_range_min, measure_range_max, resolution, unit)
VALUES (3, 'bme688_pressure', 300, 1100, 300, 1250, 0.0018, 'hPa');

INSERT INTO sensor_info (sensor_type, sensor_id_type, unit)
VALUES (4, 'bme688_gas_resistance', 'Ω');

INSERT INTO sensor_info (sensor_type, sensor_id_type, measure_range_min, measure_range_max, unit)
VALUES (5, 'bme688_iaq', 0, 500, 'IAQ');
