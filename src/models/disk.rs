use crate::Partition;

#[derive(Debug, Clone, PartialEq)]
pub enum DiskKind {
    HDD,
    SSD,
    SCM,
    Unknown(isize),
}

impl Default for DiskKind {
    fn default() -> Self {
        DiskKind::Unknown(-1)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Disk {
    device_name: String,
    model: String,
    serial: String,
    kind: DiskKind,
    size: usize,
    removable: bool,
    partitions: Vec<Partition>,
}

impl Disk {
    pub fn new(
        device_name: String,
        model: String,
        serial: String,
        kind: DiskKind,
        size: usize,
        removable: bool,
        partitions: Vec<Partition>,
    ) -> Disk {
        Disk {
            device_name,
            model,
            serial,
            kind,
            size,
            removable,
            partitions,
        }
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn serial(&self) -> &str {
        &self.serial
    }

    pub fn kind(&self) -> &DiskKind {
        &self.kind
    }

    pub fn removable(&self) -> bool {
        self.removable
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn partitions(&self) -> &[Partition] {
        &self.partitions
    }
}
