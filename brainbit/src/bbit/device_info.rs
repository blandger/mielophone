/// Contains common information about device like:
/// model, serial number, HW, SW revision
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    model_number: String,
    serial_number: String,
    hardware_revision: String,
    firmware_revision: String,
}

impl DeviceInfo {
    pub fn new(
        model_number: String,
        serial_number: String,
        hardware_revision: String,
        firmware_revision: String,
    ) -> Self {
        Self {
            model_number,
            serial_number,
            hardware_revision,
            firmware_revision,
        }
    }
}