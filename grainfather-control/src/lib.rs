use byteorder::{ByteOrder, LittleEndian};
use uuid::Uuid;

const EIR_HEADER_SIZE: usize = 2;

pub struct EIRData<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for EIRData<'a> {
    fn from(value: &'a [u8]) -> EIRData<'a> {
        Self(value)
    }
}

impl<'a> IntoIterator for EIRData<'a> {
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
    Other(u8, Vec<u8>),
}

impl EIREntry {
    fn parse(eir_data: &[u8]) -> Self {
        match eir_data[0] {
            1 => EIREntry::Flags(eir_data[1]),
            9 => EIREntry::Name(String::from_utf8_lossy(&eir_data[1..]).into()),
            7 => EIREntry::ServiceIds(
                (&eir_data[1..]).chunks(16).map(LittleEndian::read_u128).map(Uuid::from_u128).collect::<Vec<_>>(),
            ),
            other => EIREntry::Other(other, (&eir_data[1..]).to_vec()),
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
