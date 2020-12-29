use byteorder::{BigEndian, ByteOrder, LittleEndian};
use uuid::Uuid;

const EIR_HEADER_SIZE: usize = 2;

pub struct EIRData<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for EIRData<'a> {
    fn from(value: &'a [u8]) -> EIRData<'a> {
        Self(value)
    }
}

impl<'a> IntoIterator for &EIRData<'a> {
    type Item = EIREntry;
    type IntoIter = EIRDataIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        EIRDataIter(self.0)
    }
}

pub struct EIRDataIter<'a>(&'a [u8]);

impl Iterator for EIRDataIter<'_> {
    type Item = EIREntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() < EIR_HEADER_SIZE {
            return None;
        }

        let entry_length = self.0[0] as usize;

        let index_type = 1;
        let index_next = index_type + entry_length;

        let this_entry = &self.0[index_type..index_next];

        self.0 = &self.0[index_next..];

        Some(EIREntry::parse(this_entry))
    }
}

#[derive(Debug)]
pub enum EIREntry {
    Flags(u8),
    Name(String),
    ServiceIds(Vec<Uuid>),
    ManufacturerSpecific(ManufacturerSpecificEntry),
    Other(u8, Vec<u8>),
}

impl EIREntry {
    fn parse(eir_data: &[u8]) -> Self {
        let entry_type = eir_data[0];
        let entry_data = &eir_data[1..];

        match entry_type {
            0x01 => Self::Flags(entry_data[0]),

            0x07 => Self::ServiceIds(
                entry_data.chunks(16).map(LittleEndian::read_u128).map(Uuid::from_u128).collect::<Vec<_>>(),
            ),

            0x09 => Self::Name(String::from_utf8_lossy(entry_data).into()),

            0xFF => Self::ManufacturerSpecific(ManufacturerSpecificEntry::parse(entry_data)),

            _ => Self::Other(entry_type, entry_data.to_vec()),
        }
    }
}

#[derive(Debug)]
pub enum ManufacturerSpecificEntry {
    Apple(AppleEntry),
    Other(u16, Vec<u8>),
}

impl ManufacturerSpecificEntry {
    fn parse(data: &[u8]) -> Self {
        let manufacturer = LittleEndian::read_u16(&data[0..2]);
        let specific_data = &data[2..];

        match manufacturer {
            0x004c => Self::Apple(AppleEntry::parse(specific_data)),
            _ => Self::Other(manufacturer, specific_data.to_vec()),
        }
    }
}

#[derive(Debug)]
pub enum AppleEntry {
    Beacon(Beacon),
    Other(u8, Vec<u8>),
}

impl AppleEntry {
    fn parse(data: &[u8]) -> Self {
        let entry_type = data[0];
        let rest = &data[1..];

        match entry_type {
            0x02 => Self::Beacon(Beacon::parse(rest)),
            _ => Self::Other(entry_type, rest.to_vec()),
        }
    }
}

#[derive(Debug)]
pub struct Beacon {
    pub uuid: Uuid,
    pub minor: u16,
    pub major: u16,
    pub power: i8,
}

impl Beacon {
    fn parse(data: &[u8]) -> Self {
        let size = data[0];
        assert_eq!(21, size);

        let beacon_data = &data[1..];

        let uuid = BigEndian::read_u128(&beacon_data[0..16]);
        let major = BigEndian::read_u16(&beacon_data[16..18]);
        let minor = BigEndian::read_u16(&beacon_data[18..20]);
        let power = beacon_data[20] as i8;

        Self {
            uuid: Uuid::from_u128(uuid),
            major,
            minor,
            power,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_eir_test() {
        let example_data = b"\x02\x01\x06\x06\tGrain\x11\x07\xfb4\x9b_\x80\0\0\x80\0\x10\0\0\xd0\xcd\0\0";
        let eir_entries = EIRData::from(&example_data[..]).into_iter().collect::<Vec<_>>();
        println!("Entries: {:?}", eir_entries);
    }
}
