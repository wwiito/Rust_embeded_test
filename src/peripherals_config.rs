/*Pinout description
PA0 - TIM2_CH1 - PWM MOTOR A
PA1 - TIM2_CH2 - PWM MOTOR B

MOTOR CONTROL A:
PA9, PA8
MOTOR ENCODER A:
TIM3: PA6(CH1) PA7(CH2) 

MOTOR CONTROL B:
PA2, PA3
MOTOR ENCODER B:

*/

pub fn setup_pwm_timer(_tim: &stm32f1::stm32f103::TIM2, _bus_clock: u32, _requested_freq: u32)
{
    let clk_prescaled_hz = 1000000;
    let ratio = _bus_clock / clk_prescaled_hz;
    let psc: u16 = (ratio - 1) as u16;
    let arr: u16 = (clk_prescaled_hz / _requested_freq) as u16;

    _tim.psc.write(|w| unsafe {w.psc().bits(psc)});
    _tim.arr.write(|w| w.arr().bits(arr));
    
    _tim.ccmr1_output.write(|w| unsafe {w.oc1m().bits(0x06)});
    _tim.ccer.write(|w| w.cc1e().set_bit());
    _tim.ccer.modify(|_,w| w.cc1p().clear_bit());

    _tim.ccr1.write(|w| unsafe {w.bits(2)});

    _tim.dier.modify(|_,w| w.uie().set_bit());
    _tim.cr1.modify(|_, w| w.cen().enabled());
}

pub fn setup_encoder_timer(_tim: &stm32f1::stm32f103::TIM3){
    _tim.smcr.write(|w| unsafe {w.sms().bits(0b011)});
    _tim.arr.write(|w| w.arr().bits(40000));
    _tim.cnt.write(|w| w.cnt().bits(20000));

    _tim.cr1.modify(|_, w| w.cen().enabled());
}

pub fn setup_gpio_a(_gpio: &stm32f1::stm32f103::GPIOA) {
    //LED
    _gpio.crl.write(|w| unsafe{
        w.mode5().bits(0b11);
        w.cnf5().bits(0b00)
    });

    //MOTOR CONTROL_A
    _gpio.crh.write(|w| unsafe{
        w.mode10().bits(0b11);
        w.cnf10().bits(0b00)
    });  
    _gpio.crh.modify(|_,w| unsafe{
        w.mode8().bits(0b11);
        w.cnf8().bits(0b00)
    });    
    _gpio.crh.modify(|_,w| unsafe{
        w.mode9().bits(0b11);
        w.cnf9().bits(0b00)
    });

    //TIM2 CH1 GPIO
    _gpio.crl.modify(|_,w| unsafe{
        w.mode0().bits(0b11);
        w.cnf0().bits(0b10)
    });
    //TIM2 CH2 GPIO
    _gpio.crl.modify(|_,w| unsafe{
        w.mode1().bits(0b11);
        w.cnf1().bits(0b10)
    });

    //TIM3 CH1 GPIO - IN FLOATING
    _gpio.crl.modify(|_,w| unsafe{
        w.mode7().bits(0b01);
        w.cnf7().bits(0b01)
    });
    _gpio.odr.modify(|_,w| w.odr6().set_bit());
    //TIM3 CH2 GPIO  - IN FLOATING
    _gpio.crl.modify(|_,w| unsafe{
        w.mode6().bits(0b01);
        w.cnf6().bits(0b01)
    });
    _gpio.odr.modify(|_,w| w.odr7().set_bit());

}