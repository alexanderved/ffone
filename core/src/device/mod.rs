pub mod discoverer;
pub mod link;
pub mod storage;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceInfo {
    pub name: String,
}

impl DeviceInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
        }
    }
}
