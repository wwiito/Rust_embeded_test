pub trait PwmChannel {
    fn set_pwm_value(&self, value: u32);
    fn modify_pwm_value(&self, value: i32);
}

macro_rules! pwm_pin {
    ($pwmname: ident, $tim_name: ident, $channel: ident) => {
        pub struct $pwmname<'a> {
            pub timer: &'a stm32f1::stm32f103::$tim_name
        }

        impl<'a>PwmChannel for $pwmname<'a>{
            fn set_pwm_value(&self, value: u32) {
                let arr = self.timer.arr.read().arr().bits() as u32;
                if value >= 100 {
                    self.timer.$channel.write(|w| unsafe {w.bits(arr)});
                } else {
                    let tmp_val: u32 = (arr * value) / 100;
                    self.timer.$channel.write(|w| unsafe {w.bits(tmp_val)});
                }
            }
            fn modify_pwm_value(&self, value: i32) {
                let arr = self.timer.arr.read().arr().bits() as i32;
                let mut current_val = self.timer.$channel.read().bits() as i32;

                let modificator = (arr*value)/100;

                if current_val + modificator > arr {
                    current_val = arr;
                } else if current_val + modificator < 0 {
                    current_val = 0;
                }

                self.timer.$channel.write(|w| unsafe {w.bits(current_val as u32)});
            }
        }
    };
}

pwm_pin!(PwmTim2Ch1, TIM2, ccr1);