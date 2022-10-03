use std::{sync::{Mutex, Arc}, collections::HashMap};

use bytes::{Bytes};

use crate::{state::State, config::DeviceConfig, model::{TemperatureSensorReading, ThermoValveReading}};

pub struct MessageHandler {
  devices: HashMap<String, DeviceConfig>,
}

impl MessageHandler {
  pub fn new (devices: HashMap<String, DeviceConfig>) -> Self {
    MessageHandler {
      devices
    }
  }

  pub fn handle_message(&self, message: Bytes, topic: &str, state: Arc<Mutex<State>>) {
     if let Some(key_temp_sensor) = self.find_key_for_temp_sensor(&topic) {
      let temp_sensor_reading: TemperatureSensorReading = serde_json::from_slice(&message).unwrap();
      let mut state_lock = state.lock().unwrap();
      log::debug!("update state for temp_sensor for key {} with value {:?}", key_temp_sensor, temp_sensor_reading);
      state_lock.update_temp_sensor_value(&key_temp_sensor, temp_sensor_reading);
     }
     if let Some(key_valve_actuator) = self.find_key_for_valve_actuator(&topic) {
      let valve_actuator_reading: ThermoValveReading = serde_json::from_slice(&message).unwrap();
      let mut state_lock = state.lock().unwrap();
      log::debug!("update state for temp_sensor for key {} with value {:?}", key_valve_actuator, valve_actuator_reading);
      state_lock.update_valve_actuator_value(&key_valve_actuator, valve_actuator_reading);
     }

  }

  fn find_key_for_temp_sensor(&self, topic: &str) -> Option<String> {
    let mut key_found = None;
    for (key, device) in self.devices.iter() {
      if topic.contains(device.temperature_sensor.as_str()) {
        key_found = Some(key.to_string());
      }
    }
    key_found
  }

  fn find_key_for_valve_actuator(&self, topic: &str) -> Option<String> {
    let mut key_found = None;
    for (key, device) in self.devices.iter() {
      if topic.contains(device.valve_actuator.as_str()) {
        key_found = Some(key.to_string());
      }
    }
    key_found
  }
}
