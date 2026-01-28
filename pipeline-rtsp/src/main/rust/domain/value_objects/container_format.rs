#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerFormat {
    MP4,
    MKV,
}

impl ContainerFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContainerFormat::MP4 => "MP4",
            ContainerFormat::MKV => "MKV",
        }
    }
}

impl Default for ContainerFormat {
    fn default() -> Self {
        ContainerFormat::MP4
    }
}
