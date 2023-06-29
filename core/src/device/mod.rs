pub mod discoverer;
pub mod link;

#[derive(Debug, Clone, Hash)]
pub struct DeviceInfo {
    pub name: String,
}