use futures::{executor::block_on, stream::StreamExt};
use serde::{Serialize, Deserialize};
use paho_mqtt as mqtt;
use std::{env, process, time::Duration};
use env_logger::Env;
#[macro_use]
extern crate log;

// The topics to which we subscribe.
const TOPICS: &[&str] = &["zigbee2mqtt/living_room/temp_sensor", "zigbee2mqtt/living_room/thermo_valve"];
const QOS: &[i32] = &[1, 1];

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
    .filter_or("MY_LOG_LEVEL", "trace")
    .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);


    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| "tcp://192.168.1.99:1883".to_string());

    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("client_8")
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
        info!("Connecting to the MQTT server...");
        cli.connect(conn_opts).await?;

        info!("Subscribing to topics: {:?}", TOPICS);
        cli.subscribe_many(TOPICS, QOS).await?;

        // Just loop on incoming messages.
        info!("Waiting for messages...");

        // Note that we're not providing a way to cleanly shut down and
        // disconnect. Therefore, when you kill this app (with a ^C or
        // whatever) the server will get an unexpected drop and then
        // should emit the LWT message.

        // Tvs = Tvm + Tvc; -> temperature show in the valve
        // Tsm; -> temperature of the sensor
        // Tvm = Tvs - Tvc
        // Tvc_new = Tsm - Tvm = Tsm - (Tvs - Tvc)
        let mut temp_sensor: f32 = 0.0;
        let mut temp_calibration_new: f32;
        let mut temp_calibration_old: f32 = 0.0;
        let mut temp_show_on_valve_old: f32 = 0.0;
        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                if msg.to_string().contains("temp_sensor"){
                    let temp_sensor_reading: TemperatureSensorReading = serde_json::from_str(&msg.payload_str().into_owned()).unwrap();
                    temp_sensor =  temp_sensor_reading.temperature;
                    info!("Temperature sensor reading: {:?}", temp_sensor_reading);
                    info!("Temperature {:?}", temp_sensor);
                    info!("Calibration Old {:?}", temp_calibration_old);
                    info!("Temperature show on the valve{:?}", temp_show_on_valve_old);
                }
                if msg.to_string().contains("thermo_valve"){
                    let thermo_valve_reading: ThermoValveReading = serde_json::from_str(&msg.payload_str().into_owned()).unwrap();
                    temp_calibration_old = thermo_valve_reading.local_temperature_calibration;
                    temp_show_on_valve_old = thermo_valve_reading.local_temperature;
                    info!("Thermo valve reading {:?}", thermo_valve_reading);
                    info!("Temperature {:?}", temp_sensor);
                    info!("Calibration Old {:?}", temp_calibration_old);
                    info!("Temperature show on the valve {:?}", temp_show_on_valve_old);
                }

                if temp_sensor> 0.0 && temp_calibration_old > 0.0 {
                    temp_calibration_new = compute_new_calibration(temp_sensor, temp_calibration_old, temp_show_on_valve_old);
                    info!("New calibration: {:?}", temp_calibration_new);

                    // Create a message and publish it
                    if temp_calibration_new != temp_calibration_old {
                        info!("Send calibration update...");
                        let msg = mqtt::Message::new("zigbee2mqtt/living_room/thermo_valve/set/local_temperature_calibration", temp_calibration_new.to_string(), mqtt::QOS_1);
                        cli.publish(msg).await?;
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
    temp_calibration_new.trunc() + fraction_correction
}

fn round_to_correct_fraction(fraction: f32) -> f32{
    if fraction>=0.0 && fraction<=0.25 {
        return 0.0;
    } else if fraction > 0.25 && fraction <= 0.75{
        return 0.5;
    } else {
        return 1.0;
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
        // Tvc_new = 20.3 - (19.0 - (-1.0) = 0.3 => 0.5
        let result_3 = compute_new_calibration(20.3, -1.0, 19.0);
        assert_eq!(1.0, result_3);
     }
}