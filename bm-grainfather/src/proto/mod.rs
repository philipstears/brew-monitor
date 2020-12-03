use std::{convert::TryFrom, str::FromStr};

pub mod command;
pub mod notification;
pub mod recipe;

// NOTE: this is sometimes a number,
// and othertimes not. For example,
// when the sparge water is added, and the user
// presses "Set" to confirm its addition, we receive
// an interaction notification with code "C".
#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum InteractionCode {
    None,
    SkipDelayedRecipe,
    AddGrain,
    MashOutDoneStartSparge,
    Sparge,
    BoilFinished,
    Other(String),
}

impl Default for InteractionCode {
    fn default() -> Self {
        Self::None
    }
}

impl FromStr for InteractionCode {
    type Err = ();

    fn from_str(other: &str) -> Result<Self, Self::Err> {
        match other {
            "0" => Ok(Self::None),
            "1" => Ok(Self::SkipDelayedRecipe),
            "2" => Ok(Self::AddGrain),
            "3" => Ok(Self::MashOutDoneStartSparge),
            "4" => Ok(Self::Sparge),
            "6" => Ok(Self::BoilFinished),
            _ => Ok(Self::Other(other.into())),
        }
    }
}

impl ToString for InteractionCode {
    fn to_string(&self) -> String {
        match self {
            Self::None => "0".into(),
            Self::SkipDelayedRecipe => "1".into(),
            Self::AddGrain => "2".into(),
            Self::MashOutDoneStartSparge => "3".into(),
            Self::Sparge => "4".into(),
            Self::BoilFinished => "6".into(),
            Self::Other(other) => other.into(),
        }
    }
}

// TODO: what is the value here?
pub type SpargeProgress = u8;

pub type StepNumber = u8;
