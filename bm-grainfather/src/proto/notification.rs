use super::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Voltage {
    V110,
    V230,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Units {
    Fahrenheit,
    Celsius,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GrainfatherNotification {
    Temp {
        desired: f64,
        current: f64,
    },
    DelayedHeatTimer {
        active: bool,
        // If zero, the time is inactive, otherwise, it's always the number of remaining minutes +
        // 1, ergo, if it reads 2, there's 1 minute remaining, and possibly some seconds too.
        remaining_minutes: u32,
        remaining_seconds: u32,
        // The total number of minutes remaining + 1
        total_start_time: u32,
    },
    Status1 {
        heat_active: bool,
        pump_active: bool,
        auto_mode_active: bool,
        step_ramp_active: bool,
        interaction_mode_active: bool,
        interaction_code: InteractionCode,
        step_number: StepNumber,
        delayed_heat_mode_active: bool,
    },
    Status2 {
        heat_power_output_percentage: u8,
        timer_paused: bool,
        step_mash_mode: bool,
        recipe_interrupted: bool,
        manual_power_mode: bool,
        sparge_water_alert_displayed: bool,
    },
    TemperatureReached,
    PromptBoilAddition,
    PromptSpargeWater,
    Interaction {
        interaction_code: InteractionCode,
    },
    Boil {
        boil_temperature: f64,
    },
    VoltageAndUnits {
        voltage: Voltage,
        units: Units,
    },
    FirmwareVersion {
        firmware_version: String,
    },
    Other(char, String),
}

#[derive(Debug)]
pub enum GrainfatherNotificationConvertError {
    InvalidUtf8(std::str::Utf8Error),
}

impl TryFrom<&[u8]> for GrainfatherNotification {
    type Error = GrainfatherNotificationConvertError;

    fn try_from(message: &[u8]) -> Result<Self, Self::Error> {
        let ndata = std::str::from_utf8(message).map_err(Self::Error::InvalidUtf8)?;
        let mut ndata_chars = ndata.chars();
        let ndata_type = ndata_chars.next().unwrap();
        let mut ndata_fields = ndata_chars.as_str().split(",");

        match ndata_type {
            'A' => Ok(Self::PromptBoilAddition),

            'B' => Ok(Self::PromptSpargeWater),

            'E' => Ok(Self::TemperatureReached),

            'X' => {
                let desired = ndata_fields.next().unwrap().parse().unwrap();
                let current = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::Temp {
                    desired,
                    current,
                })
            }

            'T' => {
                let active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let remaining_minutes = ndata_fields.next().unwrap().parse().unwrap();
                let total_start_time = ndata_fields.next().unwrap().parse().unwrap();
                let remaining_seconds = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::DelayedHeatTimer {
                    active,
                    remaining_minutes,
                    remaining_seconds,
                    total_start_time,
                })
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
                Ok(Self::Status1 {
                    heat_active,
                    pump_active,
                    auto_mode_active,
                    step_ramp_active,
                    interaction_mode_active,
                    interaction_code,
                    step_number,
                    delayed_heat_mode_active,
                })
            }

            'W' => {
                let heat_power_output_percentage = ndata_fields.next().unwrap().parse().unwrap();
                let timer_paused = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let step_mash_mode = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let recipe_interrupted = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let manual_power_mode = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let sparge_water_alert_displayed = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                Ok(Self::Status2 {
                    heat_power_output_percentage,
                    timer_paused,
                    step_mash_mode,
                    recipe_interrupted,
                    manual_power_mode,
                    sparge_water_alert_displayed,
                })
            }

            'I' => {
                let interaction_code = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::Interaction {
                    interaction_code,
                })
            }

            'C' => {
                let boil_temperature = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::Boil {
                    boil_temperature,
                })
            }

            'F' => {
                let firmware_version = ndata_fields.next().unwrap().to_string();
                Ok(Self::FirmwareVersion {
                    firmware_version,
                })
            }

            'V' => {
                let voltage_is_110 = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let units_are_celsius = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;

                Ok(Self::VoltageAndUnits {
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
                })
            }

            _ => Ok(Self::Other(ndata_type, ndata_chars.as_str().to_string())),
        }
    }
}
