fn setup_pll(_rcc: &stm32f1::stm32f103::RCC) {
    //make sure PLL is disabled
    _rcc.cr.write(|w| w.pllon().clear_bit());
    //send HSI(8MHz) clock to PLL(div2)
    _rcc.cfgr.write(|w| w.pllsrc().hsi_div2());
    //set Pll multiplexer to x8(32MHz)
    _rcc.cfgr.modify(|_,w| w.pllmul().mul8());
    //enable PLL    
    _rcc.cr.write(|w| w.pllon().set_bit());
    //wait for Lock
    while _rcc.cr.read().pllrdy().is_not_ready() {}
}
fn setup_prescalers(_rcc: &stm32f1::stm32f103::RCC) {
    //AHB prescaler -  
    _rcc.cfgr.modify(|_,w| w.hpre().div1());
    //APB1 prescaler - 1
    _rcc.cfgr.modify(|_,w| w.ppre1().div1());
    //APB2 prescaler - 1
    _rcc.cfgr.modify(|_,w| w.ppre2().div1());
}

pub fn setup_clock(_rcc: &stm32f1::stm32f103::RCC) {
 
    setup_pll(_rcc);
    setup_prescalers(_rcc);
    _rcc.cfgr.modify(|_,w| w.sw().pll());

    _rcc.apb1enr.modify(|_,w| w.tim4en().set_bit());
    _rcc.apb1enr.modify(|_,w| w.tim3en().set_bit());
    _rcc.apb1enr.modify(|_,w| w.tim2en().set_bit());
    _rcc.apb2enr.modify(|_,w| w.iopaen().set_bit());
    _rcc.apb2enr.modify(|_,w| w.iopben().set_bit());
    _rcc.apb2enr.modify(|_,w| w.afioen().set_bit());
    _rcc.apb1enr.modify(|_,w| w.usart3en().set_bit());
}