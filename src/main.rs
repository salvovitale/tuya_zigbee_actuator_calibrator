use tokio::{task, signal};
use rumqttc::{self, MqttOptions, QoS};

use std::sync::{Arc, Mutex};
use std::error::Error;
extern crate log;

mod config;
mod model;
mod mqtt_client;
mod calibrator;
mod message_handler;
mod state;
mod server;


/////////////////////////////////////////////////////////////////////////////
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // Initialize the logger from the environment
    pretty_env_logger::init();

    // Read main configuration file
    let config = config::RunningConfig::new("config.yaml")?;

    // Setup configuration for MQTT client
    let client_id = format!("mqttui-{:x}", rand::random::<u32>());
    let host = config.mqtt.host.clone();
    let port = config.mqtt.port;
    let mut mqttoptions = MqttOptions::new(client_id, host, port);

    // Setup credentials if available
    if let Some(credentials) = config.mqtt.username.zip(config.mqtt.password) {
        mqttoptions.set_credentials(credentials.0, credentials.1);
    }
    log::debug!("{:?}", mqttoptions);

    // Specify message handlers
    let _calibrator = Arc::new(calibrator::Calibrator {});
    let handler = Arc::new(message_handler::MessageHandler::new(config.devices.clone()));

    // create initial state
    let state = Arc::new(Mutex::new(state::State::new(&config.devices)));
    let running_state = state.clone();
    // run client
    let mut mqtt_client = mqtt_client::MqttClient::new(mqttoptions, 10, &config.topics, handler.clone()).await;
    // clone async client to be used in signal handler
    let client = mqtt_client.client.clone();
    task :: spawn(async move {
        mqtt_client.run(running_state).await;
    });

    // setup web server
    server::set_up_web_server(state.clone());

    // register signal handler
    match signal::ctrl_c().await {
        Ok(()) => {
            log::debug!("Shutting Down");
            client.cancel().await?;
            client.disconnect().await?;
        },
        Err(err) => {
            log::error!("Unable to listen for shutdown signal: {}", err);
        },
    }
    Ok(())
}


