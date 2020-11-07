use bm_bluetooth::Beacon;

const TILT_RED: u128 = 0xA495BB10C5B14B44B5121370F02D74DE;
const TILT_GREEN: u128 = 0xA495BB20C5B14B44B5121370F02D74DE;
const TILT_BLACK: u128 = 0xA495BB30C5B14B44B5121370F02D74DE;
const TILT_PURPLE: u128 = 0xA495BB40C5B14B44B5121370F02D74DE;
const TILT_ORANGE: u128 = 0xA495BB50C5B14B44B5121370F02D74DE;
const TILT_BLUE: u128 = 0xA495BB60C5B14B44B5121370F02D74DE;
const TILT_YELLOW: u128 = 0xA495BB70C5B14B44B5121370F02D74DE;
const TILT_PINK: u128 = 0xA495BB80C5B14B44B5121370F02D74DE;

#[derive(Debug)]
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

pub struct Tilt {
    pub color: TiltColor,
    pub fahrenheit: u16,
    pub gravity: u16,
    pub power: i8,
}

pub enum TiltFromBeaconError {
    UnknownUniqueId,
}

impl std::convert::TryFrom<Beacon> for Tilt {
    type Error = TiltFromBeaconError;

    fn try_from(
        Beacon {
            major: fahrenheit,
            minor: gravity,
            uuid,
            power,
        }: Beacon,
    ) -> Result<Tilt, Self::Error> {
        let color = match uuid.as_u128() {
            TILT_RED => TiltColor::Red,
            TILT_GREEN => TiltColor::Green,
            TILT_BLACK => TiltColor::Black,
            TILT_PURPLE => TiltColor::Purple,
            TILT_ORANGE => TiltColor::Orange,
            TILT_BLUE => TiltColor::Blue,
            TILT_YELLOW => TiltColor::Yellow,
            TILT_PINK => TiltColor::Pink,
            _ => return Err(Self::Error::UnknownUniqueId),
        };

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
