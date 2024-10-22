use crate::transport::nus_uuids::*;
use crate::transport::NusTransport;

use std::error::Error;
use async_trait::async_trait;

// use btleplug::api::BDAddr;
// use btleplug::api::{Central, CentralEvent, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType};
// use btleplug::platform::Adapter;
// use btleplug::platform::Peripheral;
// use futures::stream::StreamExt;


/// NOTE: intentionally empty struct
pub struct NusTransportMock{name: String}

async fn timeout<F: std::future::Future>(future: F) -> Result<F::Output, tokio::time::error::Elapsed> {
    tokio::time::timeout(std::time::Duration::from_millis(500), future).await
}


#[async_trait]
impl NusTransport for NusTransportMock {
    /// MTU of the BLE link
    async fn mtu(&self) -> usize {  244 }
    /// Send data to data point
    async fn send(&self, bytes: &[u8]) -> Result<(), Box<dyn Error>> { Ok(()) }
    /// Exchange request with control point
    async fn recv(&self) -> Result<Vec<u8>, Box<dyn Error>> { Ok(vec![]) }
}

impl NusTransportMock {
    pub fn new(
        name: String,
    )-> NusTransportMock {

        NusTransportMock{ name }

    }
}
