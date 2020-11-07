use bm_bluetooth::*;
use std::convert::TryFrom;

const GF: u128 = 0x0000cdd000001000800000805f9b34fb;

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
                if let Some(_) = ids.iter().find(|id| id.as_u128() == GF) {
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
