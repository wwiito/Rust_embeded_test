

pub trait GpioPin {
    fn set_pin(&self);
    fn clr_pin(&self);
}

macro_rules! gpio_pin {
    ($cl: ident, $port_name: ident, $pin: expr) => {
        pub struct $cl<'a> {
            pub port: &'a stm32f1::stm32f103::$port_name
        }

        impl<'a> GpioPin for $cl<'a>{
            fn set_pin(&self) {
                self.port.odr.modify(|r,w| unsafe {w.bits(r.bits() | (1<<$pin))});
            }
            fn clr_pin(&self) {
                self.port.odr.modify(|r,w| unsafe {w.bits(r.bits() & !(1<<$pin))});
            }
        }
    };
}

gpio_pin!(GpioA1, GPIOA, 1);
gpio_pin!(GpioA2, GPIOA, 2);

gpio_pin!(GpioA5, GPIOA, 5);
gpio_pin!(GpioA6, GPIOA, 6);

gpio_pin!(GpioA8, GPIOA, 8);
gpio_pin!(GpioA9, GPIOA, 9);
gpio_pin!(GpioA10, GPIOA, 10);