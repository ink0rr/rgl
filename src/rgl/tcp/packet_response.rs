use anyhow::Result;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

#[derive(Serialize)]
pub struct PacketResponse {
    #[serde(flatten)]
    pub packet_type: PacketResponseType,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PacketResponseType {
    Protocol(PacketProtocol),
    StopOnException(PacketStopOnException),
    Resume(PacketResume),
}

#[derive(Serialize)]
pub struct PacketProtocol {
    pub version: u32,
    pub target_module_uuid: String,
}

#[derive(Serialize)]
pub struct PacketStopOnException {
    #[serde(rename = "stopOnException")]
    pub stop_on_exception: bool,
}

#[derive(Serialize)]
pub struct PacketResume {}

impl PacketResponse {
    pub async fn write(&self, writer: &mut (impl AsyncWriteExt + Unpin)) -> Result<()> {
        let str = serde_json::to_string(self)?;
        let size = str.len() + 1;
        let header = format!("{:08x}", size);
        let data = format!("{}\n{}\n", header, str);
        writer.write_all(data.as_bytes()).await?;
        Ok(())
    }
}
