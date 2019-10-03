// std and main are not available for bare metal software
#![no_std]
#![no_main]

extern crate stm32f1;
extern crate panic_halt;
extern crate cortex_m_rt;

use core::cell::RefCell;
use core::cell::Cell;

use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use stm32f1::stm32f103;
use stm32f103::interrupt;

mod frequency {
    pub const APB1_TIMER: u32 = 8000000;
}
const CLK_PRESCAED_HZ: u32 = 500; 
const TIM_FREQ: u32 = 10;

fn setup_timer(_tim: &stm32f1::stm32f103::TIM1) {
    let ratio = frequency::APB1_TIMER / CLK_PRESCAED_HZ;
    let psc: u16 = (ratio - 1) as u16;
    let arr: u16 = (CLK_PRESCAED_HZ / TIM_FREQ) as u16;

    _tim.psc.write(|w| unsafe {w.psc().bits(psc)});
    _tim.arr.write(|w| unsafe {w.arr().bits(arr)});
    _tim.cr1.modify(|_, w| w.cen().enabled());
}

fn setup_pll(_rcc: &stm32f1::stm32f103::RCC) {
    //make sure PLL is disabled
    _rcc.cr.write(|w| w.pllon().clear_bit());
    //send HSI(8MHz) clock to PLL
    _rcc.cfgr.write(|w| w.pllsrc().clear_bit());
    //set Pll multiplexer to x9(72MHz)
    _rcc.cfgr.modify(|_,w| w.pllmul().mul9());
    //enable PLL
    _rcc.cr.write(|w| w.pllon().set_bit());
    //wait for Lock
    while _rcc.cr.read().pllrdy().is_not_ready() {}
}

fn setup_clock(_rcc: &stm32f1::stm32f103::RCC) {

    setup_pll(_rcc);


    _rcc.apb2enr.write(|w| w.iopaen().set_bit());
    _rcc.apb2enr.modify(|_,w| w.tim1en().set_bit());
}

fn setup_gpio(_gpio: &stm32f1::stm32f103::GPIOA) {
    _gpio.crl.write(|w| unsafe{
        w.mode5().bits(0b11);
        w.cnf5().bits(0b00)
    });
}

static COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static MUTEX_GPIOA: Mutex<RefCell<Option<stm32f1::stm32f103::GPIOA>>> = Mutex::new(RefCell::new(None));
static MUTEX_TIM1: Mutex<RefCell<Option<stm32f1::stm32f103::TIM1>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // get handles to the hardware
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32f103::Peripherals::take().unwrap();
    let led_port = &peripherals.GPIOA;
    let rcc = &peripherals.RCC;
    let tim = &peripherals.TIM1;

    let mut nvic = cortexm_peripherals.NVIC;


	hprintln!("Hello, world!").unwrap();

    setup_clock(rcc);
    setup_timer(tim);
    setup_gpio(led_port);

    tim.dier.modify(|_,w| w.uie().set_bit());

    cortex_m::interrupt::free(|cs| {
        MUTEX_GPIOA.borrow(cs).replace(Some(peripherals.GPIOA));
        MUTEX_TIM1.borrow(cs).replace(Some(peripherals.TIM1))
    });

    nvic.enable(stm32f103::Interrupt::TIM1_UP);

    loop{
        cortex_m::asm::wfi();
    }
}

#[interrupt]
fn TIM1_UP() {
        cortex_m::interrupt::free(|cs| {
            let t = MUTEX_TIM1.borrow(cs).borrow();
            let gpio = MUTEX_GPIOA.borrow(cs).borrow();
            t.as_ref().unwrap().sr.modify(|_,w| w.uif().clear_bit());

            if COUNTER.borrow(cs).get() == 0 {
                COUNTER.borrow(cs).set(1);
                gpio.as_ref().unwrap().odr.write(|w| w.odr5().clear_bit());
            } else {
                COUNTER.borrow(cs).set(0);
                gpio.as_ref().unwrap().odr.write(|w| w.odr5().set_bit());
            }
        });
}