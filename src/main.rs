// std and main are not available for bare metal software
#![no_std]
#![no_main]


#![feature(lang_items)]
#[lang = "owned_box"]
pub struct Box<T>(*mut T);
#[lang = "arc"]
pub struct Arc<T>(*mut T);
#[lang = "rc"]
pub struct Rc<T>(*mut T);

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

mod clocks;
mod peripherals_config;
mod gpio;
mod pwm;
mod motor;

use crate::gpio::GpioPin;
use crate::pwm::PwmChannel;

const APB2_FREQ: u32 = 32000000;
const TIM_FREQ: u32 = 5;

static COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static MUTEX_GPIOA: Mutex<RefCell<Option<stm32f1::stm32f103::GPIOA>>> = Mutex::new(RefCell::new(None));
static MUTEX_TIM2: Mutex<RefCell<Option<stm32f1::stm32f103::TIM2>>> = Mutex::new(RefCell::new(None));
static MUTEX_TIM4: Mutex<RefCell<Option<stm32f1::stm32f103::TIM4>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // get handles to the hardware
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32f103::Peripherals::take().unwrap();
    let port_a = &peripherals.GPIOA;
    let rcc = &peripherals.RCC;
    let tim = &peripherals.TIM4;
    let tim_pwm = &peripherals.TIM2;

    let mut nvic = cortexm_peripherals.NVIC;

    let mot_dir_1 = gpio::GpioA8{port: port_a};
    let mot_dir_2 = gpio::GpioA9{port: port_a};

    let mut motor_cntrl = motor::Motor{speed: 20};

	hprintln!("Hello, world!").unwrap();
    

    clocks::setup_clock(rcc);
    peripherals_config::setup_reg_timer(tim, APB2_FREQ, TIM_FREQ);
    peripherals_config::setup_pwm_timer(tim_pwm, APB2_FREQ);
    peripherals_config::setup_gpio(port_a);

    

    cortex_m::interrupt::free(|cs| {
        MUTEX_GPIOA.borrow(cs).replace(Some(peripherals.GPIOA));
        MUTEX_TIM4.borrow(cs).replace(Some(peripherals.TIM4));
        MUTEX_TIM2.borrow(cs).replace(Some(peripherals.TIM2))
    });

    nvic.enable(stm32f103::Interrupt::TIM4);

    loop{
    motor_cntrl.next_speed();
    cortex_m::interrupt::free(|cs| {
        let pwm_timer = MUTEX_TIM2.borrow(cs).borrow();
        motor_cntrl.funny(& pwm::PwmTim2Ch1{timer: pwm_timer.as_ref().unwrap()});
    });
        cortex_m::asm::wfi();
    }
}

#[interrupt]
fn TIM4() {
        cortex_m::interrupt::free(|cs| {
            let t = MUTEX_TIM4.borrow(cs).borrow();
            let gpio = MUTEX_GPIOA.borrow(cs).borrow();
            let pwm_timer = MUTEX_TIM2.borrow(cs).borrow();

            let led_pin = gpio::GpioA5{port: gpio.as_ref().unwrap()};
            let tpwm = pwm::PwmTim2Ch1{timer: pwm_timer.as_ref().unwrap()};

            t.as_ref().unwrap().sr.modify(|_,w| w.uif().clear_bit());

            if COUNTER.borrow(cs).get() == 0 {
                COUNTER.borrow(cs).set(1);
                led_pin.set_pin();
                //tpwm.set_pwm_value(20);
            } else {
                COUNTER.borrow(cs).set(0);
                led_pin.clr_pin();
                //tpwm.set_pwm_value(75);
            }
        });
}