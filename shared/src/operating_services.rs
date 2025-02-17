pub struct ServiceValue {
    pub value: bool,
}

pub trait OperatingServiceManager {
    fn new() -> Self;

    fn update_service(&mut self, service: ClusterService, value: bool) -> Result<(), String>;

    fn get_service(&self, service: ClusterService) -> bool;
}
