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
    pub fn get_current_motor_position(&self) -> i32 {
        return self.motor_position.get();
    }
    pub fn get_current_speed(&self) -> i32 {
        return self.motor_speed.get();
    }

    pub fn set_speed(&self, speed: i32) {
        self.motor_requested_speed.set(speed);
    }

    pub fn regulate_speed(&self, pwmch: &pwm::PwmTim2Ch1, gpio_a: &dyn gpio::GpioPin, gpio_b: &dyn gpio::GpioPin) {
        let req_speed = self.motor_requested_speed.get();
        let speed = self.get_current_speed();
        let error = req_speed - speed;
        let mut dir = MotorDirection::MotorDirLeft;

        if req_speed > 0 {
            dir = MotorDirection::MotorDirLeft;
        } else {
            dir = MotorDirection::MotorDirRight;
        }
        self.direction.set(dir);
        self.direction_has_changed.set(true);

        let mut tmp = (error * 8).abs();

        if tmp < 0 {
            tmp = 0
        }
        if tmp > 100 {
            tmp = 100;
        }

        self.motor_pwr.set(tmp.try_into().unwrap());
        self.apply_direction(dir, gpio_a, gpio_b);
        self.update_motor_pwr(tmp.try_into().unwrap(), pwmch);
    }

    pub fn apply_direction(&self, dir: MotorDirection, gpio_a: &dyn gpio::GpioPin, gpio_b: &dyn gpio::GpioPin) {
        match dir {
            MotorDirection::MotorDirLeft  => {gpio_b.set_pin(); gpio_a.clr_pin();},
            MotorDirection::MotorDirRight => {gpio_b.clr_pin(); gpio_a.set_pin();},
        }
    }
    pub fn update_motor_pwr(&self, pwr: u32, pwmch: &pwm::PwmTim2Ch1) {
        pwmch.set_pwm_value(pwr);
    }
}