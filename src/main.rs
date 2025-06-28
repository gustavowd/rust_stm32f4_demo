#![no_std]
#![no_main]

//use cortex_m::Peripherals;
//use cortex_m_rt::pre_init;
//use core::arch::asm;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_stm32::gpio::{Output, Pull, Level, Speed};
use embassy_stm32::exti::ExtiInput;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};


// Declare async tasks
#[embassy_executor::task]
async fn button_task(mut button: ExtiInput<'static>) {
    info!("Press the USER button...");

    loop {
        button.wait_for_rising_edge().await;
        info!("Pressed!");
        button.wait_for_falling_edge().await;
        info!("Released!");
    }
}



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL168,
            divp: Some(PllPDiv::DIV2),
            divq: Some(PllQDiv::DIV7), // USB clock at 48 MHz
            // Main system clock at 168 MHz
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.sys = Sysclk::PLL1_P;

        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
    }

    let p: embassy_stm32::Peripherals = embassy_stm32::init(config);

    info!("Hello World!");
    //defmt::println!("Hello, world!");

    let button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Down);


    spawner.spawn(button_task(button)).unwrap();


    let mut led = Output::new(p.PD12, Level::High, Speed::Low);

    loop {
        //info!("high");
        led.set_high();
        Timer::after_millis(500).await;

        //info!("low");
        led.set_low();
        Timer::after_millis(500).await;
    }
}

