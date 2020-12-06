pub mod notifications;
use notifications::*;

use super::*;

/// Represents the Grainfather controller's supported power supply.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Voltage {
    V110,
    V230,
}

/// Represents the temperature units in use by the Grainfather for display.
///
/// Note that the units used in commands, notifications, and recipes always use
/// degrees celsius.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Units {
    Fahrenheit,
    Celsius,
}

/// Represents a notification received asynchronously from the Grainfather controller.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Notification {
    /// Indicates the current and target temperature measured by the controller.
    Temp(Temp),

    /// Indicates the status of the active timer on the controller.
    DelayedHeatTimer(Timer),

    /// Provides a collection of related status information - this is the
    /// controller's "Y" status report.
    Status1(Status1),

    /// Provides a collection of related status information - this is the
    /// controller's "W" status report.
    Status2(Status2),

    TemperatureReached(TemperatureReached),

    /// Indicates that the controller has prompted to add a boil addition.
    PromptBoilAddition(PromptBoilAddition),

    /// Indicates that the controller has prompted to heat the sparge water.
    PromptSpargeWater(PromptSpargeWater),

    /// Indicates that the controller has prompted an interaction and is awaiting
    /// the user to confirm.
    Interaction(Interaction),

    /// Provides a response to the [GetBoilTemperature](crate::Command::GetBoilTemperature) command.
    Boil(Boil),

    /// Provides a response to the [GetVoltageAndUnits](crate::Command::GetVoltageAndUnits) command.
    VoltageAndUnits(VoltageAndUnits),

    /// Provides a response to the [GetFirmwareVersion](crate::Command::GetFirmwareVersion) command.
    FirmwareVersion(FirmwareVersion),

    /// Represents an unknown notification from the controller.
    Other(Other),
}

#[derive(Debug)]
pub enum NotificationConvertError {
    InvalidUtf8(std::str::Utf8Error),
}

impl TryFrom<&[u8]> for Notification {
    type Error = NotificationConvertError;

    fn try_from(message: &[u8]) -> Result<Self, Self::Error> {
        let ndata = std::str::from_utf8(message).map_err(Self::Error::InvalidUtf8)?;
        let mut ndata_chars = ndata.chars();
        let ndata_type = ndata_chars.next().unwrap();
        let mut ndata_fields = ndata_chars.as_str().split(",");

        match ndata_type {
            'A' => Ok(Self::PromptBoilAddition(PromptBoilAddition)),

            'B' => Ok(Self::PromptSpargeWater(PromptSpargeWater)),

            'E' => Ok(Self::TemperatureReached(TemperatureReached)),

            'X' => {
                let desired = ndata_fields.next().unwrap().parse().unwrap();
                let current = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::Temp(Temp {
                    desired,
                    current,
                }))
            }

            'T' => {
                let active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let remaining_minutes = ndata_fields.next().unwrap().parse().unwrap();
                let total_start_time = ndata_fields.next().unwrap().parse().unwrap();
                let remaining_seconds = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::DelayedHeatTimer(Timer {
                    active,
                    remaining_minutes,
                    remaining_seconds,
                    total_start_time,
                }))
            }

            'Y' => {
                let heat_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let pump_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let auto_mode_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let step_ramp_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let interaction_mode_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let interaction_code = ndata_fields.next().unwrap().parse().unwrap();
                let step_number = ndata_fields.next().unwrap().parse().unwrap();
                let delayed_heat_mode_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                Ok(Self::Status1(Status1 {
                    heat_active,
                    pump_active,
                    auto_mode_active,
                    step_ramp_active,
                    interaction_mode_active,
                    interaction_code,
                    step_number,
                    delayed_heat_mode_active,
                }))
            }

            'W' => {
                let heat_power_output_percentage = ndata_fields.next().unwrap().parse().unwrap();
                let timer_paused = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let step_mash_mode = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let recipe_interrupted = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let manual_power_mode = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let sparge_water_alert_displayed = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                Ok(Self::Status2(Status2 {
                    heat_power_output_percentage,
                    timer_paused,
                    step_mash_mode,
                    recipe_interrupted,
                    manual_power_mode,
                    sparge_water_alert_displayed,
                }))
            }

            'I' => {
                let interaction_code = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::Interaction(Interaction {
                    interaction_code,
                }))
            }

            'C' => {
                let boil_temperature = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::Boil(Boil {
                    boil_temperature,
                }))
            }

            'F' => {
                let firmware_version = ndata_fields.next().unwrap().to_string();
                Ok(Self::FirmwareVersion(FirmwareVersion {
                    firmware_version,
                }))
            }

            'V' => {
                let voltage_is_110 = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let units_are_celsius = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;

                Ok(Self::VoltageAndUnits(VoltageAndUnits {
                    voltage: if voltage_is_110 {
                        Voltage::V110
                    } else {
                        Voltage::V230
                    },
                    units: if units_are_celsius {
                        Units::Celsius
                    } else {
                        Units::Fahrenheit
                    },
                }))
            }

            _ => Ok(Self::Other(Other {
                r#type: ndata_type,
                data: ndata_chars.as_str().to_string(),
            })),
        }
    }
}
