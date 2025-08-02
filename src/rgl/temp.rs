use std::path::PathBuf;

pub struct Temp {
    pub bp: PathBuf,
    pub rp: PathBuf,
    pub data: PathBuf,
    pub root: PathBuf,
}

impl Temp {
    pub fn from_dot_regolith() -> Self {
        let dot_regolith = PathBuf::from(".regolith");
        let temp = dot_regolith.join("tmp");
        Self {
            bp: temp.join("BP"),
            rp: temp.join("RP"),
            data: temp.join("data"),
            root: temp,
        }
    }
}
