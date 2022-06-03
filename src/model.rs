use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TemperatureSensorReading{
    pub temperature: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ThermoValveReading{
    pub local_temperature: f32,
    pub local_temperature_calibration: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoupledThermoValveAndSensorReadings{
    temp_sensor: TemperatureSensorReading,
    temp_sensor_initialized: bool,
    valve_actuator: ThermoValveReading,
    valve_actuator_initialized: bool,
}


impl CoupledThermoValveAndSensorReadings {
    pub fn new() -> Self {
        CoupledThermoValveAndSensorReadings {
            temp_sensor: TemperatureSensorReading { temperature: 0.0 },
            temp_sensor_initialized: false,
            valve_actuator: ThermoValveReading { local_temperature: 0.0, local_temperature_calibration: 0.0 },
            valve_actuator_initialized: false,
        }
    }
    pub fn set_temp_sensor(&mut self, temp_sensor: TemperatureSensorReading) {
        self.temp_sensor = temp_sensor;
        if !self.temp_sensor_initialized {
            self.temp_sensor_initialized = true;
        }
    }

    pub fn get_temp_sensor(&self) -> &TemperatureSensorReading {
        &self.temp_sensor
    }

    pub fn set_valve_actuator(&mut self, valve_actuator: ThermoValveReading) {
        self.valve_actuator = valve_actuator;
        if !self.valve_actuator_initialized {
            self.valve_actuator_initialized = true;
        }
    }

    pub fn get_valve_actuator(&self) -> &ThermoValveReading {
        &self.valve_actuator
    }

    pub fn is_ready_to_be_calibrated(&self) -> bool {
        self.temp_sensor_initialized && self.valve_actuator_initialized
    }

}