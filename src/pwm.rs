pub trait PwmChannel {
    fn set_pwm_value(&self, value: u32);
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
        }
    };
}

pwm_pin!(PwmTim2Ch1, TIM2, ccr1);