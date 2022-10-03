use std::collections::HashMap;

use crate::{model::{CoupledThermoValveAndSensorReadings, TemperatureSensorReading, ThermoValveReading}, config::DeviceConfig};

#[derive(Debug)]
pub struct State {
  devices_state: HashMap<String, CoupledThermoValveAndSensorReadings>
}


impl State {
  pub fn new(devices: &HashMap<String, DeviceConfig>) -> Self {
    let devices_init_state: HashMap<String, CoupledThermoValveAndSensorReadings> = devices.iter().map(|(key, _)| {
      (key.to_string(), CoupledThermoValveAndSensorReadings::new())}).collect();
    State {
      devices_state: devices_init_state
    }
  }
  pub fn update_temp_sensor_value(&mut self, key: &String, temp_sensor: TemperatureSensorReading) {
    let device_state_option = self.devices_state.get_mut(key);
    match device_state_option  {
      Some(device_state) => {
        device_state.set_temp_sensor(temp_sensor);
        log::debug!("{:?}", self.devices_state)
      },
      None => {
        log::error!("Error: key {} not found in state", key);
      }
    }
  }
  pub fn update_valve_actuator_value(&mut self, key: &String, valve_actuator: ThermoValveReading) {
    let device_state_option = self.devices_state.get_mut(key);
    match device_state_option  {
      Some(device_state) => {
        device_state.set_valve_actuator(valve_actuator);
        log::debug!("{:?}", self.devices_state)
      },
      None => {
        log::error!("Error: key {} not found in state", key);
      }
    }
  }

  pub fn get_devices_state(&self) -> &HashMap<String, CoupledThermoValveAndSensorReadings> {
    &self.devices_state
  }
}