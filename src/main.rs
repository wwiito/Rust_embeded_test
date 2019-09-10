// std and main are not available for bare metal software
#![no_std]
#![no_main]

extern crate stm32f1;
extern crate panic_halt;
extern crate cortex_m_rt;

use cortex_m_rt::entry;
use stm32f1::stm32f103;
use cortex_m_semihosting::hprintln;

mod frequency {
    pub const APB1_TIMER: u32 = 64000000;
}
const CLK_PRESCAED_HZ: u32 = 1000; 
const TIM_FREQ: u32 = 1;

#[entry]
fn main() -> ! {
    // get handles to the hardware
    let peripherals = stm32f103::Peripherals::take().unwrap();
    let led_port = &peripherals.GPIOA;
    let rcc = &peripherals.RCC;
    let tim = &peripherals.TIM6;

	hprintln!("Hello, world!").unwrap();

    // enable the GPIO clock for IO port A
    rcc.apb2enr.write(|w| w.iopaen().set_bit());
    led_port.crl.write(|w| unsafe{
        w.mode5().bits(0b11);
        w.cnf5().bits(0b00)
    });

    let ratio = frequency::APB1_TIMER / CLK_PRESCAED_HZ;
    let psc: u16 = (ratio - 1) as u16;
    let arr: u16 = (CLK_PRESCAED_HZ / TIM_FREQ) as u16;

    rcc.apb1enr.write(|w| w.tim6en().set_bit());
    tim.psc.write(|w| unsafe {w.psc().bits(psc)});
    tim.arr.write(|w| unsafe {w.arr().bits(arr)});
    tim.cr1.modify(|_, w| w.cen().enabled());

    hprintln!("rcc1: {}", rcc.apb1enr.read().bits()).unwrap();
    hprintln!("rcc2: {}", rcc.apb2enr.read().bits()).unwrap();
    hprintln!("uif: {}", tim.sr.read().uif().bit_is_set()).unwrap();
    hprintln!("psc: {}, {}", tim.psc.read().psc().bits(), psc).unwrap();
    hprintln!("arr: {}, {}", tim.arr.read().arr().bits(), arr).unwrap();
    hprintln!("cr1: {}", tim.cr1.read().bits()).unwrap();

    loop{
        led_port.odr.write(|w| w.odr5().set_bit());
        while tim.sr.read().uif().bit_is_clear() {};
        led_port.odr.write(|w| w.odr5().clear_bit());
        while tim.sr.read().uif().bit_is_clear() {};
    }
}
