//! Data types for specific notifications.

use crate::{InteractionCode, StepNumber, Units, Voltage};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Temp {
    pub desired: f64,
    pub current: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Timer {
    pub active: bool,
    // If zero, the time is inactive, otherwise, it's always the number of remaining minutes +
    // 1, ergo, if it reads 2, there's 1 minute remaining, and possibly some seconds too.
    pub remaining_minutes: u32,
    pub remaining_seconds: u32,
    // The total number of minutes remaining + 1
    pub total_start_time: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Status1 {
    pub heat_active: bool,
    pub pump_active: bool,
    pub auto_mode_active: bool,
    pub step_ramp_active: bool,
    pub interaction_mode_active: bool,
    pub interaction_code: InteractionCode,
    pub step_number: StepNumber,
    pub delayed_heat_mode_active: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Status2 {
    pub heat_power_output_percentage: u8,
    pub timer_paused: bool,
    pub step_mash_mode: bool,
    pub recipe_interrupted: bool,
    pub manual_power_mode: bool,
    pub sparge_water_alert_displayed: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TemperatureReached;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PromptBoilAddition;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PromptSpargeWater;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Interaction {
    pub interaction_code: InteractionCode,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Boil {
    pub boil_temperature: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VoltageAndUnits {
    pub voltage: Voltage,
    pub units: Units,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FirmwareVersion {
    pub firmware_version: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Other {
    pub r#type: char,
    pub data: String,
}
