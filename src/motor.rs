use crate::pwm;
use crate::pwm::PwmChannel;
use crate::gpio;

use crate::encoder;

use core::convert::TryInto;
use core::cell::Cell;



#[derive(Copy, Clone, PartialEq)]
pub enum MotorDirection {
    MotorDirLeft,  //Motor Position Rising
    MotorDirRight, //Motor Position Decreasing
}

pub struct Motor {
    pub motor_position: Cell<i32>,
    pub prev_error: Cell<f64>,
    pub total_error: Cell<f64>,
    pub motor_speed: Cell<i32>,
    pub motor_requested_speed: Cell<i32>,
}
impl Motor {
    pub fn update_motor_position(&self, enc: &dyn encoder::EncoderPins) {
        let position_diff = enc.get_current_value();
        let position_total = self.motor_position.get();

        self.motor_position.set(position_diff + position_total);
        self.motor_speed.set(position_diff);
    }
    pub fn get_current_speed(&self) -> i32 {
        return self.motor_speed.get();
    }

    pub fn set_speed(&self, speed: i32) {
        self.motor_requested_speed.set(speed);
    }
    pub fn get_req_speed(&self) -> i32 {
        return self.motor_requested_speed.get();
    }


    fn normalize_motor_pwr(&self, pwr: i32) -> i32 {
        return pwr.abs();
    }

    fn regulate(&self) -> i32 {
        let current_speed = self.get_current_speed();
        let prev_error = self.prev_error.get();
        let requested_speed = self.motor_requested_speed.get(); 

        let error = requested_speed - current_speed;

        let err_dif: f64 = ((prev_error + f64::from(error))/2.0)*0.01;
        let err_total = self.total_error.get() + err_dif;

        let p_part = error;
        let d_part = err_total as i32;

        let regulator_value = (p_part * 30) + (d_part * 10);

        self.prev_error.set(f64::from(error));
        self.total_error.set(err_total);

        return regulator_value;
    }

    pub fn regulate_speed(&self, pwmch: &pwm::PwmTim2Ch1, gpio_a: &dyn gpio::GpioPin, gpio_b: &dyn gpio::GpioPin) {
        let req_speed = self.motor_requested_speed.get();
        let dir: MotorDirection;

        let reg = self.regulate();
        let pwr = self.normalize_motor_pwr(reg);

        if reg > 0 {
            dir = MotorDirection::MotorDirLeft;
        } else {
            dir = MotorDirection::MotorDirRight;
        }

        self.apply_direction(dir, gpio_a, gpio_b);
        //self.modify_motor_pwr(reg, pwmch);
        self.update_motor_pwr(pwr, pwmch);
    }

    fn apply_direction(&self, dir: MotorDirection, gpio_a: &dyn gpio::GpioPin, gpio_b: &dyn gpio::GpioPin) {
        match dir {
            MotorDirection::MotorDirLeft  => {gpio_b.set_pin(); gpio_a.clr_pin();},
            MotorDirection::MotorDirRight => {gpio_b.clr_pin(); gpio_a.set_pin();},
        }
    }
    fn update_motor_pwr(&self, pwr: i32, pwmch: &pwm::PwmTim2Ch1) {
        pwmch.set_pwm_value(pwr as u32);
    }
    fn modify_motor_pwr(&self, pwr: i32, pwmch: &pwm::PwmTim2Ch1) {
        pwmch.modify_pwm_value(pwr);
    }
}