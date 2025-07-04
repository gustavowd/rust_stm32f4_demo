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
use embassy_stm32::bind_interrupts;
use embassy_stm32::usart::{self, Uart};
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

// Declare async tasks
#[embassy_executor::task]
async fn uart_task(mut uart: Uart<'static, embassy_stm32::mode::Async>) {
    info!("UART started, type something...");
    uart.write("UART started, type something...".as_bytes()).await.unwrap();

    let mut buffer = [0u8; 1];

    // Loop to read from UART and echo back
    loop {
        uart.read(&mut buffer).await.unwrap();
        uart.write(&buffer).await.unwrap();
    }
}


bind_interrupts!(struct Irqs {
    USART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART1>;
});

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

    let mut config = usart::Config::default();
    config.baudrate = 115_200;
    let usart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA2_CH7, p.DMA2_CH2, config).unwrap();
    spawner.spawn(uart_task(usart)).unwrap();


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

