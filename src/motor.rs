use crate::pwm;
use crate::pwm::PwmChannel;

pub enum MotorDirection {
    MotorDirLeft,
    MotorDirRight,
}

pub struct Motor {
    pub speed: u32,
}
impl Motor {
    pub fn next_speed(&mut self) {
        self.speed += 1;
        if self.speed > 100 {
            self.speed = 0;
        }
    }
    pub fn funny(&self, pwmch: &pwm::PwmTim2Ch1) {
        pwmch.set_pwm_value(self.speed);
    }
}