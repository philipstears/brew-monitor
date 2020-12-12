use bm_bluetooth::*;
use std::convert::{TryFrom, TryInto};

const TILT_RED: u128 = 0xA495BB10C5B14B44B5121370F02D74DE;
const TILT_GREEN: u128 = 0xA495BB20C5B14B44B5121370F02D74DE;
const TILT_BLACK: u128 = 0xA495BB30C5B14B44B5121370F02D74DE;
const TILT_PURPLE: u128 = 0xA495BB40C5B14B44B5121370F02D74DE;
const TILT_ORANGE: u128 = 0xA495BB50C5B14B44B5121370F02D74DE;
const TILT_BLUE: u128 = 0xA495BB60C5B14B44B5121370F02D74DE;
const TILT_YELLOW: u128 = 0xA495BB70C5B14B44B5121370F02D74DE;
const TILT_PINK: u128 = 0xA495BB80C5B14B44B5121370F02D74DE;

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum TiltColor {
    Red,
    Green,
    Black,
    Purple,
    Orange,
    Blue,
    Yellow,
    Pink,
}

impl std::string::ToString for TiltColor {
    fn to_string(&self) -> String {
        match self {
            Self::Red => "red".into(),
            Self::Green => "green".into(),
            Self::Black => "black".into(),
            Self::Purple => "purple".into(),
            Self::Orange => "orange".into(),
            Self::Blue => "blue".into(),
            Self::Yellow => "yellow".into(),
            Self::Pink => "pink".into(),
        }
    }
}

impl TryFrom<&str> for TiltColor {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let color = match value {
            "red" => TiltColor::Red,
            "green" => TiltColor::Green,
            "black" => TiltColor::Black,
            "purple" => TiltColor::Purple,
            "orange" => TiltColor::Orange,
            "blue" => TiltColor::Blue,
            "yellow" => TiltColor::Yellow,
            "pink" => TiltColor::Pink,
            _ => {
                return Err(());
            }
        };

        Ok(color)
    }
}

impl TryFrom<u128> for TiltColor {
    type Error = ();

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let color = match value {
            TILT_RED => TiltColor::Red,
            TILT_GREEN => TiltColor::Green,
            TILT_BLACK => TiltColor::Black,
            TILT_PURPLE => TiltColor::Purple,
            TILT_ORANGE => TiltColor::Orange,
            TILT_BLUE => TiltColor::Blue,
            TILT_YELLOW => TiltColor::Yellow,
            TILT_PINK => TiltColor::Pink,
            _ => {
                return Err(());
            }
        };

        Ok(color)
    }
}

pub struct Tilt {
    pub color: TiltColor,
    pub fahrenheit: u16,
    pub gravity: u16,
    pub power: i8,
}

pub enum TiltConvertError {
    NoBeaconFound,
    UnknownUniqueId,
}

impl TryFrom<&EIRData<'_>> for Tilt {
    type Error = TiltConvertError;

    fn try_from(report: &EIRData) -> Result<Self, Self::Error> {
        for entry in report.into_iter() {
            if let EIREntry::ManufacturerSpecific(ms) = entry {
                if let ManufacturerSpecificEntry::Apple(apple) = ms {
                    if let AppleEntry::Beacon(beacon) = apple {
                        return beacon.try_into();
                    }
                }
            }
        }

        Err(Self::Error::NoBeaconFound)
    }
}

impl TryFrom<Beacon> for Tilt {
    type Error = TiltConvertError;

    fn try_from(
        Beacon {
            major: fahrenheit,
            minor: gravity,
            uuid,
            power,
        }: Beacon,
    ) -> Result<Self, Self::Error> {
        let color = uuid.as_u128().try_into().map_err(|_| Self::Error::UnknownUniqueId)?;

        Ok(Self {
            color,
            fahrenheit,
            gravity,
            power,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
