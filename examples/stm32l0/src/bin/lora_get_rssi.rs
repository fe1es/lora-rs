//! This example runs on the STM32 LoRa Discovery board, which has a builtin Semtech Sx1276 radio.
//! It demonstrates LORA get rssi functionality.
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{Channel, ExtiInput};
use embassy_stm32::gpio::{Input, Level, Output, Pin, Pull, Speed};
use embassy_stm32::spi;
use embassy_stm32::time::khz;
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use lora_phy::iv::GenericSx127xInterfaceVariant;
use lora_phy::sx127x::{Sx127x, Sx1276};
use lora_phy::LoRa;
use lora_phy::{mod_params::*, sx127x};
use {defmt_rtt as _, panic_probe as _};

const LORA_FREQUENCY_IN_HZ: u32 = 903_900_000; // warning: set this appropriately for the region

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.mux = embassy_stm32::rcc::ClockSrc::HSI;
    let p = embassy_stm32::init(config);

    let nss = Output::new(p.PA15.degrade(), Level::High, Speed::Low);
    let reset = Output::new(p.PC0.degrade(), Level::High, Speed::Low);
    let irq_pin = Input::new(p.PB4.degrade(), Pull::Up);
    let irq = ExtiInput::new(irq_pin, p.EXTI4.degrade());

    let mut spi_config = spi::Config::default();
    spi_config.frequency = khz(200);
    let spi = spi::Spi::new(p.SPI1, p.PB3, p.PA7, p.PA6, p.DMA1_CH3, p.DMA1_CH2, spi_config);
    let spi = ExclusiveDevice::new(spi, nss, Delay);

    let config = sx127x::Config {
        chip: Sx1276,
        tcxo_used: true,
        rx_boost: true,
        tx_boost: false,
    };
    let iv = GenericSx127xInterfaceVariant::new(reset, irq, None, None).unwrap();
    let mut lora = LoRa::new(Sx127x::new(spi, iv, config), false, Delay).await.unwrap();

    match lora.listen(LORA_FREQUENCY_IN_HZ, Bandwidth::_500KHz).await {
        Ok(()) => {}
        Err(err) => {
            info!("Radio error = {}", err);
            return;
        }
    };

    loop {
        match lora.get_rssi().await {
            Ok(rssi) => {
                info!("RSSI: {}", rssi);
            }
            Err(err) => {
                info!("Radio error = {}", err);
            }
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}
