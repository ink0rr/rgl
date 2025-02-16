use serde::Deserialize;

#[derive(Deserialize)]
pub struct PacketRequest {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub packet_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<PacketEventType>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum PacketEventType {
    PrintEvent,
    ProtocolEvent(ProtocolEvent),
    StatEvent2,
    #[serde(rename = "terminated")]
    Terminated,
    ThreadEvent,
}

#[derive(Deserialize)]
pub struct ProtocolEvent {
    pub version: u32,
    pub plugins: Vec<Plugin>,
    pub require_passcode: bool,
}

#[derive(Deserialize)]
pub struct Plugin {
    #[allow(dead_code)]
    pub name: String,
    pub module_uuid: String,
}
