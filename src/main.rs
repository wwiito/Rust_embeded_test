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
static MUTEX_USART2: Mutex<RefCell<Option<stm32f1::stm32f103::USART3>>> = Mutex::new(RefCell::new(None));

static MUTEX_TIM2_DIVIDER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

static MUTEX_TEST: Mutex<motor::Motor> = Mutex::new(motor::Motor{motor_requested_speed: Cell::new(30),
                                                                 prev_error: Cell::new(0.0),
                                                                 total_error: Cell::new(0.0),
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
    let musart = &peripherals.USART3;

    let mut nvic = cortexm_peripherals.NVIC;
    let mut tmp_pos: i32 = 0;
    let mut req_speed: i32 = 0;
    let mut div = 0;
    let mut div1 = 0;

    clocks::setup_clock(rcc);
    peripherals_config::setup_pwm_timer(tim_pwm, APB2_FREQ, TIM2_REQ_FREQ);
    peripherals_config::setup_gpio_a(port_a);
    peripherals_config::setup_gpio_b(&peripherals.GPIOB);
    peripherals_config::setup_encoder_timer(tim_enc_a);

    musart.cr1.write(|w| w.ue().set_bit());
    musart.brr.write(|w| unsafe{w.div_mantissa().bits(17)});
    //musart.brr.modify(|_,w| unsafe{w.div_fraction().bits(6)});
    musart.cr1.modify(|_,w| w.te().set_bit());
    musart.dr.write(|w| unsafe{w.dr().bits(0x30)});
    hprintln!("PBrr: {}", musart.brr.read().bits()).unwrap();
    hprintln!("CR1: {}", musart.cr1.read().bits()).unwrap();
    hprintln!("SR1: {}", musart.sr.read().bits()).unwrap();

    cortex_m::interrupt::free(|cs| {
        MUTEX_GPIOA.borrow(cs).replace(Some(peripherals.GPIOA));
        MUTEX_TIM4.borrow(cs).replace(Some(peripherals.TIM4));
        MUTEX_TIM2.borrow(cs).replace(Some(peripherals.TIM2));
        MUTEX_TIM3.borrow(cs).replace(Some(peripherals.TIM3));
        MUTEX_USART2.borrow(cs).replace(Some(peripherals.USART3));
    });

    nvic.enable(stm32f103::Interrupt::TIM2);

    hprintln!("Config done!").unwrap();

    loop{
        cortex_m::asm::wfi();
        if div == 2000 {
            cortex_m::interrupt::free(|cs| {
                let u = MUTEX_USART2.borrow(cs).borrow();
                let tmp = MUTEX_TEST.borrow(cs);
                tmp_pos = tmp.get_current_speed();
                req_speed = tmp.get_req_speed();
                tmp.set_speed(20);
                u.as_ref().unwrap().dr.write(|w| unsafe{w.dr().bits(0x35)});
            });
            hprintln!("Position: {}, req: {}", tmp_pos, req_speed).unwrap();
            div = 0;
        } else {
            div += 1;
        }
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