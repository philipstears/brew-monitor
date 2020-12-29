#[cfg(feature = "btleplug")]
pub mod btleplug;

use bm_bluetooth::*;

/// The service identifier for the grainfather controller.
pub const SERVICE_ID: u128 = 0x0000cdd000001000800000805f9b34fb;

/// The identifier of the characteristic used for receiving notifications
/// from the grainfather controller.
pub const CHARACTERISTIC_ID_READ: u128 = 0x0003cdd100001000800000805f9b0131;

/// The identifier of the characteristic used to issue commands to the
/// grainfather controller.
pub const CHARACTERISTIC_ID_WRITE: u128 = 0x0003cdd200001000800000805f9b0131;

/// Searches for the presence of the Grainfather's [service id](crate::SERVICE_ID)
/// in the provided extended information report.
pub fn has_grainfather_service_id(report: &EIRData) -> bool {
    report.into_iter().any(|entry| {
        if let EIREntry::ServiceIds(ids) = entry {
            return ids.iter().any(|id| id.as_u128() == SERVICE_ID);
        }

        return false;
    })
}
