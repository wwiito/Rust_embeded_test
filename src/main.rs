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
    pub const APB1_TIMER: u32 = 8000000;
}
const CLK_PRESCAED_HZ: u32 = 500; 
const TIM_FREQ: u32 = 1;

fn setup_timer(_tim: &stm32f1::stm32f103::TIM1) {
    let ratio = frequency::APB1_TIMER / CLK_PRESCAED_HZ;
    let psc: u16 = (ratio - 1) as u16;
    let arr: u16 = (CLK_PRESCAED_HZ / TIM_FREQ) as u16;

    _tim.psc.write(|w| unsafe {w.psc().bits(psc)});
    _tim.arr.write(|w| unsafe {w.arr().bits(arr)});
    _tim.cr1.modify(|_, w| w.cen().enabled());
}

fn setup_clock(_rcc: &stm32f1::stm32f103::RCC) {
    _rcc.apb2enr.write(|w| w.iopaen().set_bit());
    _rcc.apb2enr.modify(|_,w| w.tim1en().set_bit());
}

fn setup_gpio(_gpio: &stm32f1::stm32f103::GPIOA) {
    _gpio.crl.write(|w| unsafe{
        w.mode5().bits(0b11);
        w.cnf5().bits(0b00)
    });
}

fn wait_for_timer(_tim: &stm32f1::stm32f103::TIM1) {
    while _tim.sr.read().uif().bit_is_clear() {};
    _tim.sr.modify(|_,w| w.uif().clear());
}

#[entry]
fn main() -> ! {
    // get handles to the hardware
    let peripherals = stm32f103::Peripherals::take().unwrap();
    let led_port = &peripherals.GPIOA;
    let rcc = &peripherals.RCC;
    let tim = &peripherals.TIM1;

	hprintln!("Hello, world!").unwrap();

    setup_clock(rcc);
    setup_timer(tim);
    setup_gpio(led_port);
    
    loop{
        led_port.odr.write(|w| w.odr5().set_bit());
        wait_for_timer(tim);
        led_port.odr.write(|w| w.odr5().clear_bit());
        wait_for_timer(tim);
    }
}
