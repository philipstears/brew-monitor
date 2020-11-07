use bm_bluetooth::*;
use std::convert::TryFrom;

pub const SERVICE_ID: u128 = 0x0000cdd000001000800000805f9b34fb;
pub const CHARACTERISTIC_ID_READ: u128 = 0x0003cdd100001000800000805f9b0131;
pub const CHARACTERISTIC_ID_WRITE: u128 = 0x0003cdd200001000800000805f9b0131;

#[derive(Debug)]
pub struct Grainfather {}

#[derive(Debug)]
pub enum GrainfatherConvertError {
    ServiceIdNotFound,
}

impl TryFrom<EIRData<'_>> for Grainfather {
    type Error = GrainfatherConvertError;

    fn try_from(report: EIRData) -> Result<Self, Self::Error> {
        for entry in report.into_iter() {
            if let EIREntry::ServiceIds(ids) = entry {
                if let Some(_) = ids.iter().find(|id| id.as_u128() == SERVICE_ID) {
                    return Ok(Self {});
                }
            }
        }

        Err(Self::Error::ServiceIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
