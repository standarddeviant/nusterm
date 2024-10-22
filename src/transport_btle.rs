use crate::transport::nus_uuids::*;
use crate::transport::NusTransport;

use std::error::Error;
// use std::ops::Try;

use async_trait::async_trait;
use btleplug::api::BDAddr;
use btleplug::api::{Central, CentralEvent, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::Adapter;
use btleplug::platform::Peripheral;
use futures::stream::StreamExt;

async fn find_characteristic_by_uuid(
    peripheral: &Peripheral,
    uuid: uuid::Uuid,
) -> Result<Characteristic, Box<dyn Error>> {
    for char in peripheral.characteristics() {
        if uuid == char.uuid {
            return Ok(char);
        }
    }
    Err("characteristic not found".into())
}

async fn find_peripheral(
    central: &Adapter, in_name: &str, in_addr: Result<BDAddr, btleplug::api::ParseBDAddrError>
) -> Result<Peripheral, Box<dyn Error>>
{
    println!("Searching for {:?} and {:?}...", in_name, in_addr);
    central.start_scan(ScanFilter::default()).await?;
    let mut events = central.events().await?;

    while let Some(event) = events.next().await {
        if let CentralEvent::DeviceDiscovered(id) = event {
            let props = central.peripheral(&id).await?.properties().await?.unwrap();
            let loop_addr = props.address;
            let loop_name = props.local_name;

            // let local_addr= unsafe { central.peripheral(&id).await?.properties().await? }   
            if let Ok(ina) = in_addr {
                if ina == loop_addr {
                    println!("Found [{:?}] at [{}]", ina, id);
                    central.stop_scan().await?;
                    return Ok(central.peripheral(&id).await?);
                }
            }
            if let Some(loopn) = loop_name {
                if in_name.len() > 0 && in_name == loopn {
                    println!("Found [{:?}] at [{}]", in_name, id);
                    central.stop_scan().await?;
                    return Ok(central.peripheral(&id).await?);
                }
            }
        }
    }
    Err("unexpected end of stream".into())
}

async fn timeout<F: std::future::Future>(future: F) -> Result<F::Output, tokio::time::error::Elapsed> {
    tokio::time::timeout(std::time::Duration::from_millis(500), future).await
}

pub struct NusTransportBtleplug {
    peripheral: Peripheral,
    nus_rx: Characteristic,
    nus_tx: Characteristic,
}

#[async_trait]
impl NusTransport for &NusTransportBtleplug {
    async fn mtu(&self) -> usize {
        // TODO fix once btleplug supports MTU lookup
        244
    }
    async fn send(&self, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        self.nus_send(&self.nus_rx, bytes, WriteType::WithoutResponse).await
    }
    async fn recv(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        // TODO - how to organize the callback + byte vecs
        self.nus_recv(&self.nus_tx).await
    }
}

impl NusTransportBtleplug {
    async fn nus_send(&self, chr: &Characteristic, bytes: &[u8], write_type: WriteType) -> Result<(), Box<dyn Error>> {
        let res = timeout(self.peripheral.write(chr, bytes, write_type)).await?;
        Ok(res?)
    }
    async fn nus_recv(
        &self,
        chr: &Characteristic,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut notifications = self.peripheral.notifications().await.unwrap();
        // timeout(self.peripheral.write(chr, bytes, write_type)).await??;
        loop {
            let ntf = timeout(notifications.next()).await?.unwrap();
            if ntf.uuid == chr.uuid {
                return Ok(ntf.value);
            }
        }
    }
    pub async fn new(
        name: String,
        addr: Result<BDAddr, btleplug::api::ParseBDAddrError>
    )-> Result<Self, Box<dyn Error>> {

        let manager = btleplug::platform::Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().unwrap();

        let mut peripheral: Peripheral = find_peripheral(&central, name.as_str(), addr).await?;
        peripheral.connect().await?;
        peripheral.discover_services().await?;

        let nus_rx: Characteristic = find_characteristic_by_uuid(&peripheral, NUS_RX_CHAR_UUID).await?;
        let nus_tx: Characteristic = find_characteristic_by_uuid(&peripheral, NUS_TX_CHAR_UUID).await?;
        peripheral.subscribe(&nus_tx).await?;
        Ok(NusTransportBtleplug {
            peripheral,
            nus_rx, // NOTE: TX is w.r.t. to peripheral, so this char notifies central
            nus_tx, // NOTE: RX is w.r.t. to peripheral, so this char is written to by central
        })
    }
}
