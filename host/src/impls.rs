use std::{error::Error, fmt::{Debug, Display}};

use postcard_rpc::host_client::{WireRx, WireTx};

pub struct ProbeRttTx {

}

#[derive(Debug)]
pub enum ProbeRttTxError {

}

impl Display for ProbeRttTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Error for ProbeRttTxError {}


#[derive(Debug)]
pub enum ProbeRttRxError {

}

impl Display for ProbeRttRxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Error for ProbeRttRxError {}

pub struct ProbeRttRx {

}

impl WireTx for ProbeRttTx {
    type Error = ProbeRttTxError;

    async fn send(&mut self, data: Vec<u8>) -> Result<(), Self::Error> {
        todo!()
    }
}

impl WireRx for ProbeRttRx {
    type Error = ProbeRttRxError;

    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }
}
