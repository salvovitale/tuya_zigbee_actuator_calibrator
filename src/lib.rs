use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::error::Error;


#[derive(Serialize, Deserialize, Debug)]
struct ReadingConfig{
    mqtt: MqttConfig,
    devices: HashMap<String, DeviceConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RunningConfig{
    pub mqtt: MqttConfig,
    pub devices: HashMap<String, DeviceConfig>,
    // topics: HashMap<String, Vec<String>>,
    pub topics: Vec<String>,
    pub qos: Vec<i32>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MqttConfig{
    pub server: String,
    pub base_topic: String,
    qos_value: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceConfig{
    pub temperature_sensor: String,
    pub valve_actuator: String
}




impl RunningConfig{
    pub fn new(file_name: &str) -> Result<RunningConfig, Box<dyn Error>>{
      let input_file = std::fs::File::open(file_name)?;
      let init_config: ReadingConfig = serde_yaml::from_reader(input_file)?;
      // println!("Read YAML string: {:?}", init_config);

      // let topics : HashMap<String, Vec<String>> =
      //   init_config.devices.iter().map(
      //     |(key, value)|
      //     {
      //         (key.clone(),
      //           vec![
      //             format!("{}/{}", init_config.mqtt.base_topic, value.temperature_sensor),
      //             format!("{}/{}", init_config.mqtt.base_topic, value.valve_actuator)])
      //     }
      //   ).collect();
      let mut topics : Vec<String> =
        init_config.devices.iter().map(
          |(_, value)|
          {
              format!("{}/{}", init_config.mqtt.base_topic, value.temperature_sensor)
          }
        ).collect();
        topics.extend(
          init_config.devices.iter().map(
            |(_, value)|
            {
                format!("{}/{}", init_config.mqtt.base_topic, value.valve_actuator)
            }
          )
        );
      let qos = vec![init_config.mqtt.qos_value; topics.len()];
      let config = RunningConfig{
        mqtt: init_config.mqtt,
        devices: init_config.devices,
        topics,
        qos
      };

      Ok(config)
    }
}