pub mod btleplug;

use bm_bluetooth::*;

pub const SERVICE_ID: u128 = 0x0000cdd000001000800000805f9b34fb;
pub const CHARACTERISTIC_ID_READ: u128 = 0x0003cdd100001000800000805f9b0131;
pub const CHARACTERISTIC_ID_WRITE: u128 = 0x0003cdd200001000800000805f9b0131;

pub fn has_grainfather_service_id(report: &EIRData) -> bool {
    report.into_iter().any(|entry| {
        if let EIREntry::ServiceIds(ids) = entry {
            return ids.iter().any(|id| id.as_u128() == SERVICE_ID);
        }

        return false;
    })
}
