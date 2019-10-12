const ENC_MIDDLE_VAL: i32 = 20000;

pub trait EncoderPins {
    fn get_current_value(&self) -> i32;
}

macro_rules! encoder_pins {
    ($cl: ident, $tim_name: ident) => {
        pub struct $cl<'a> {
            pub timer: &'a stm32f1::stm32f103::$tim_name
        }

        impl<'a> EncoderPins for $cl<'a>{
            fn get_current_value(&self) -> i32 {
                let currentVal: i32 = self.timer.cnt.read().cnt().bits() as i32;
                let diff = currentVal - ENC_MIDDLE_VAL;

                self.timer.cnt.write(|w| w.cnt().bits(ENC_MIDDLE_VAL as u16));
                return diff;
            }
        }
    };
}

encoder_pins!(EncoderTIM3, TIM3);