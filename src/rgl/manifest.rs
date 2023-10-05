use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    format_version: u8,
    header: ManifestHeader,
    modules: Vec<ManifestModule>,
    dependencies: Vec<ManifestDependency>,
}

#[derive(Serialize, Deserialize)]
pub struct ManifestHeader {
    name: String,
    description: String,
    uuid: String,
    version: Vec<u8>,
    min_engine_version: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct ManifestModule {
    #[serde(rename = "type")]
    module_type: String,
    uuid: String,
    version: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct ManifestDependency {
    uuid: String,
    version: Vec<u8>,
}

pub enum PackType {
    Behavior,
    Resource,
}

impl Manifest {
    pub fn new(
        pack_type: PackType,
        header_uuid: &str,
        deps_uuid: &str,
        min_engine_version: &str,
    ) -> Self {
        let version = vec![1, 0, 0];
        Self {
            format_version: 2,
            header: ManifestHeader {
                name: "pack.name".to_owned(),
                description: "pack.description".to_owned(),
                uuid: header_uuid.to_owned(),
                version: version.to_owned(),
                min_engine_version: min_engine_version
                    .split(".")
                    .map(|s| s.parse::<u8>().unwrap())
                    .collect::<Vec<u8>>(),
            },
            modules: vec![ManifestModule {
                module_type: match pack_type {
                    PackType::Behavior => "data".to_owned(),
                    PackType::Resource => "resources".to_owned(),
                },
                uuid: Uuid::new_v4().to_string(),
                version: version.to_owned(),
            }],
            dependencies: vec![ManifestDependency {
                uuid: deps_uuid.to_owned(),
                version: version.to_owned(),
            }],
        }
    }
}
