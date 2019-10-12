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
mod encoder;


const APB2_FREQ: u32 = 32000000;
const MOTOR_CHECK_FREQ: u32 = 100;
const TIM2_REQ_FREQ: u32 = 1000;

static MUTEX_GPIOA: Mutex<RefCell<Option<stm32f1::stm32f103::GPIOA>>> = Mutex::new(RefCell::new(None));
static MUTEX_TIM2: Mutex<RefCell<Option<stm32f1::stm32f103::TIM2>>> = Mutex::new(RefCell::new(None));
static MUTEX_TIM3: Mutex<RefCell<Option<stm32f1::stm32f103::TIM3>>> = Mutex::new(RefCell::new(None));
static MUTEX_TIM4: Mutex<RefCell<Option<stm32f1::stm32f103::TIM4>>> = Mutex::new(RefCell::new(None));

static MUTEX_TIM2_DIVIDER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

static MUTEX_TEST: Mutex<motor::Motor> = Mutex::new(motor::Motor{motor_requested_speed: Cell::new(30),
                                                                 motor_speed: Cell::new(0),
                                                                 motor_position: Cell::new(0)});

#[entry]
fn main() -> ! {
    // get handles to the hardware
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32f103::Peripherals::take().unwrap();
    let port_a = &peripherals.GPIOA;
    let rcc = &peripherals.RCC;
    let tim_pwm = &peripherals.TIM2;
    let tim_enc_a = &peripherals.TIM3;

    let mut nvic = cortexm_peripherals.NVIC;
    let mut tmp_pos: i32 = 0;
    let mut div = 0;
    let mut div1 = 0;
    let mut speed = 30;

    clocks::setup_clock(rcc);
    peripherals_config::setup_pwm_timer(tim_pwm, APB2_FREQ, TIM2_REQ_FREQ);
    peripherals_config::setup_gpio_a(port_a);
    peripherals_config::setup_encoder_timer(tim_enc_a);

    cortex_m::interrupt::free(|cs| {
        MUTEX_GPIOA.borrow(cs).replace(Some(peripherals.GPIOA));
        MUTEX_TIM4.borrow(cs).replace(Some(peripherals.TIM4));
        MUTEX_TIM2.borrow(cs).replace(Some(peripherals.TIM2));
        MUTEX_TIM3.borrow(cs).replace(Some(peripherals.TIM3));
    });

    nvic.enable(stm32f103::Interrupt::TIM2);

    hprintln!("Config done!").unwrap();

    loop{
        cortex_m::asm::wfi();
        if div == 500 {
            cortex_m::interrupt::free(|cs| {
                let tmp = MUTEX_TEST.borrow(cs);
                tmp_pos = tmp.get_current_speed();
            });
            hprintln!("Position: {}", tmp_pos).unwrap();
            div = 0;
        } else {
            div += 1;
        }
        if div1 == 1500 {
            cortex_m::interrupt::free(|cs| {
                let tmp = MUTEX_TEST.borrow(cs);
                tmp.set_speed(30);
            });
        } else if div1 == 3000 {
            cortex_m::interrupt::free(|cs| {
                let tmp = MUTEX_TEST.borrow(cs);
                tmp.set_speed(40);
            });
        } else if div1 == 4500 {
            cortex_m::interrupt::free(|cs| {
                let tmp = MUTEX_TEST.borrow(cs);
                tmp.set_speed(-30);
            });
        } else if div1 == 6000 {
            cortex_m::interrupt::free(|cs| {
                let tmp = MUTEX_TEST.borrow(cs);
                tmp.set_speed(-40);
            });
            div1 = 0;
        }
        div1 += 1;
    }
}

#[interrupt]
fn TIM2() {
        cortex_m::interrupt::free(|cs| {
            let t = MUTEX_TIM2.borrow(cs).borrow();
            let tmp = MUTEX_TEST.borrow(cs);
            let divider = MUTEX_TIM2_DIVIDER.borrow(cs).get();
            let g = MUTEX_GPIOA.borrow(cs).borrow();
            let tim_encoder = MUTEX_TIM3.borrow(cs).borrow();

            //let gpio_pin_a = gpio::GpioA10{port: &g.as_ref().unwrap()};
            let gpio_pin_a = gpio::GpioA9{port: &g.as_ref().unwrap()};
            let gpio_pin_b = gpio::GpioA8{port: &g.as_ref().unwrap()};
            let enc = encoder::EncoderTIM3{timer: &tim_encoder.as_ref().unwrap()};

            t.as_ref().unwrap().sr.modify(|_,w| w.uif().clear_bit());

            if divider == TIM2_REQ_FREQ / MOTOR_CHECK_FREQ {
                tmp.update_motor_position(&enc);
                tmp.regulate_speed(&pwm::PwmTim2Ch1{timer: t.as_ref().unwrap()}, &gpio_pin_a, &gpio_pin_b);
                MUTEX_TIM2_DIVIDER.borrow(cs).set(0);
            } else {
                MUTEX_TIM2_DIVIDER.borrow(cs).set(divider + 1);
            }
           
        });
}