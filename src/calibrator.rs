pub struct Calibrator {}

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