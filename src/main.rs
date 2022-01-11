use futures::{executor::block_on, stream::StreamExt};
use serde::{Serialize, Deserialize};
use paho_mqtt as mqtt;
use std::{env, process, time::Duration};
use env_logger::Env;
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;
use tuya_ts0601_thermostat_calibrator::RunningConfig;
#[macro_use]
extern crate log;


/////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
struct TemperatureSensorReading{
    temperature: f32
}

#[derive(Serialize, Deserialize, Debug)]
struct ThermoValveReading{
    local_temperature: f32,
    local_temperature_calibration: f32
}

fn main() {
    // Initialize the logger from the environment
    let env = Env::default()
    .filter_or("MY_LOG_LEVEL", "info")
    .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let config = RunningConfig::new("config.yaml").unwrap();
    println!("{:?}", config);
    // // mqtt part
    mqtt_part(&config);
}

fn mqtt_part(running_config: &RunningConfig) {
    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| running_config.mqtt.server.clone());

    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let client_id = Uuid::new_v4();
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(client_id.to_string())
        .finalize();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        // Get message stream before connecting.
        let mut strm = cli.get_stream(25);

        // Define the set of options for the connection
        let lwt = mqtt::Message::new("test", "Async subscriber lost connection", mqtt::QOS_1);

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(30))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(false)
            .will_message(lwt)
            .finalize();

        // Make the connection to the broker
        println!("Connecting to the MQTT server...");
        cli.connect(conn_opts).await?;

        println!("Subscribing to topics: {:?}", running_config.topics);
        cli.subscribe_many(&running_config.topics, &running_config.qos).await?;

        // Just loop on incoming messages.
        println!("Waiting for messages...");

        // Note that we're not providing a way to cleanly shut down and
        // disconnect. Therefore, when you kill this app (with a ^C or
        // whatever) the server will get an unexpected drop and then
        // should emit the LWT message.

        // Tvs = Tvm + Tvc; -> temperature show in the valve
        // Tsm; -> temperature of the sensor
        // Tvm = Tvs - Tvc
        // Tvc_new = Tsm - Tvm = Tsm - (Tvs - Tvc)
        let mut temperature_sensors: HashMap<String,f32> = running_config.devices.iter().map(|(_, value)| (value.temperature_sensor.clone(), 0.0)).collect();
        let mut temperature_calibration_old: HashMap<String,f32> = running_config.devices.iter().map(|(_, value)| (value.valve_actuator.clone(), 0.0)).collect();
        let mut temperature_show_on_valve_old: HashMap<String,f32> = running_config.devices.iter().map(|(_, value)| (value.valve_actuator.clone(), 0.0)).collect();
        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                for (_, value) in &running_config.devices {
                    let temp_sensor_name = value.temperature_sensor.clone();
                    let valve_actuator_name = value.valve_actuator.clone();
                    if msg.to_string().contains(temp_sensor_name.as_str()) {
                        let temp_sensor_reading: TemperatureSensorReading = serde_json::from_str(&msg.payload_str().into_owned()).unwrap();
                        // temp_sensor =  temp_sensor_reading.temperature;
                        if let Some(value) = temperature_sensors.get_mut(&temp_sensor_name) {
                            *value = temp_sensor_reading.temperature;
                        }
                        println!("Temperature sensor reading: {:?} {:?}", temp_sensor_name, temp_sensor_reading);
                        // info!("Temperature {:?}", temp_sensor);
                        // info!("Calibration Old {:?}", temp_calibration_old);
                        // info!("Temperature show on the valve{:?}", temp_show_on_valve_old);
                    }
                    if msg.to_string().contains(valve_actuator_name.as_str()){
                        let thermo_valve_reading: ThermoValveReading = serde_json::from_str(&msg.payload_str().into_owned()).unwrap();
                        if let Some(value) = temperature_calibration_old.get_mut(&valve_actuator_name) {
                            *value = thermo_valve_reading.local_temperature_calibration;
                        }
                        if let Some(value) = temperature_show_on_valve_old.get_mut(&valve_actuator_name) {
                            *value = thermo_valve_reading.local_temperature;
                        }
                        println!("Thermo valve reading {:?} {:?}", valve_actuator_name, thermo_valve_reading);
                        // info!("Temperature {:?}", temp_sensor);
                        // info!("Calibration Old {:?}", temp_calibration_old);
                        // info!("Temperature show on the valve {:?}", temp_show_on_valve_old);
                    }
                    let temp_sensor = temperature_sensors.get(&temp_sensor_name).unwrap();
                    let temp_calibration_old = temperature_calibration_old.get(&valve_actuator_name).unwrap();
                    let temp_show_on_valve_old = temperature_show_on_valve_old.get(&valve_actuator_name).unwrap();
                    if *temp_sensor> 0.0 && *temp_show_on_valve_old > 0.0 {
                        let temp_calibration_new = compute_new_calibration(*temp_sensor, *temp_calibration_old, *temp_show_on_valve_old);
                        // Update calibrator message and publish it
                        if (temp_calibration_new - *temp_calibration_old).abs() > 0.49 {
                            println!("New calibration for actuator {:?} with value {:?}", valve_actuator_name, temp_calibration_new);
                            let publishing_topic = format!("{}/{}/{}", running_config.mqtt.base_topic, valve_actuator_name,"set/local_temperature_calibration");
                            println!("Send calibration update to topic {:?} ...", publishing_topic);
                            let msg = mqtt::Message::new(publishing_topic, temp_calibration_new.to_string(), mqtt::QOS_1);
                            cli.publish(msg).await?;
                        }
                    }
                }
            }
            else {
                // A "None" means we were disconnected. Try to reconnect...
                println!("Lost connection. Attempting reconnect.");
                while let Err(err) = cli.reconnect().await {
                    println!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    async_std::task::sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        // Explicit return type for the async block
        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}

fn compute_new_calibration(temp_sensor: f32, temp_calibration_old: f32, temp_show_on_valve_old: f32) -> f32{
    let temp_calibration_new =  temp_sensor - (temp_show_on_valve_old - temp_calibration_old );
    let fraction_correction = round_to_correct_fraction(temp_calibration_new.fract());
    let new_calibration = temp_calibration_new.trunc() + fraction_correction;
    if new_calibration.abs() <= 5.0 {
        return new_calibration
    } else {
        return 5.0*new_calibration.signum();
    }

}

fn round_to_correct_fraction(fraction: f32) -> f32{
    let fraction_abs = fraction.abs();
    if fraction_abs>=0.0 && fraction_abs<=0.33 {
        return 0.0;
    } else if fraction_abs > 0.33 && fraction_abs <= 0.66{
        return 0.5*fraction.signum();
    } else {
        return 1.0*fraction.signum();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        //Tvc_new = Tsm - (Tvs - Tvc)
        // Tvc_new = 21.6 - (18.0 - 1.0) = 4.6 => 4.5
        let result_1 = compute_new_calibration(21.6, 1.0, 18.0);
        assert_eq!(4.5, result_1);
        // Tvc_new = 20.2 - (20.0 - 1.0) = 1.2 => 1.0
        let result_2 = compute_new_calibration(20.2, 1.0, 20.0);
        assert_eq!(1.0, result_2);
        // Tvc_new = 20.3 - (22.0 - 0.0) = -1.7 => -2.0
        let result_3 = compute_new_calibration(20.3, 0.0, 22.0);
        assert_eq!(-2.0, result_3);
        // Tvc_new = 20.3 - (27.0 - 0.0) = -6.7 => -5.0
        let result_4 = compute_new_calibration(20.3, 0.0, 27.0);
        assert_eq!(-5.0, result_4);
        // Tvc_new = 24.0 - (13.2 - 0.0) = 10.8 => +5.0
        let result_5 = compute_new_calibration(24.0, 0.0, 13.2);
        assert_eq!(5.0, result_5);
     }
}