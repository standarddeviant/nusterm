
use async_trait::async_trait;
use std::error::Error;

// NOTE: BLE UUIDs for NUS copied from bleak example, uart_service.py
// UART_SERVICE_UUID = "6E400001-B5A3-F393-E0A9-E50E24DCCA9E"
// UART_RX_CHAR_UUID = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E"
// UART_TX_CHAR_UUID = "6E400003-B5A3-F393-E0A9-E50E24DCCA9E"



/// nRF DFU service & characteristic UUIDs
///
/// from [DFU BLE Service](https://infocenter.nordicsemi.com/topic/sdk_nrf5_v17.1.0/group__nrf__dfu__ble.html)
/// and [Buttonless DFU Service](https://infocenter.nordicsemi.com/topic/sdk_nrf5_v17.1.0/service_dfu.html)
#[allow(dead_code)]
pub mod nus_uuids {
    use uuid::Uuid;
    /// NUS Service UUID
    pub const NUS_SVC_UUID: Uuid = Uuid::from_u128(0x6E400001_B5A3_F393_E0A9_E50E24DCCA9E);
    /// NUS RX UUID
    pub const NUS_RX_CHAR_UUID: Uuid = Uuid::from_u128(0x6E400002_B5A3_F393_E0A9_E50E24DCCA9E);
    /// NUS TX UUID
    pub const NUS_TX_CHAR_UUID: Uuid = Uuid::from_u128(0x6E400003_B5A3_F393_E0A9_E50E24DCCA9E);
}

/// nRF NUS transport interface
#[async_trait]
#[allow(dead_code)]
pub trait NusTransport {
    /// MTU of the BLE link
    async fn mtu(&self) -> usize;
    /// Send data to data point
    async fn send(&self, bytes: &[u8]) -> Result<(), Box<dyn Error>>;
    /// Exchange request with control point
    async fn recv(&self) -> Result<Vec<u8>, Box<dyn Error>>;

    // Disconnect function
    // async fn disconnect(&self) -> Result<(), Box<dyn Error>>;
}
