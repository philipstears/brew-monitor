use super::*;

pub(crate) const COMMAND_LEN: usize = 19;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Delay {
    Minutes(u32),
    MinutesSeconds(u32, u8),
}

/// Indicates the type of disconnection to perform.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum DisconnectOption {
    /// Disconnects the controller, canceling the session (recipe) if one is active.
    ManualMode,

    /// Cancels the session (recipe), but leaves the controller connected.
    CancelSession,

    /// Disconnects the controller, and leaves it running in automatic (recipe) mode.
    AutomaticMode,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Command {
    Reset,

    /// Requests the firmware version from the controller, the response is returned in a
    /// [FirmwareVersion](crate::Notification::FirmwareVersion) notification.
    GetFirmwareVersion,

    /// Requests the voltage and temperature units from the controller, the response is returned in a
    /// [VoltageAndUnits](crate::Notification::VoltageAndUnits) notification.
    GetVoltageAndUnits,

    /// Requests the active boil temperature from the controller, the response is returned in a
    /// [Boil](crate::Notification::Boil) notification.
    GetBoilTemperature,

    ToggleHeatActive,
    SetHeatActive(bool),

    TogglePumpActive,
    SetPumpActive(bool),

    // NOTE: minutes is odd, {2, 0} will only run for 1 minute, and {2, 30} will run for 1 minute
    // 30 seconds, {1, 30} and {0, 30} will both run for 30 seconds
    EnableDelayedHeatTimer {
        minutes: u32,
        seconds: u8,
    },

    CancelActiveTimer,

    UpdateActiveTimer(Delay),
    PauseOrResumeActiveTimer,

    IncrementTargetTemperature,
    DecrementTargetTemperature,
    SetTargetTemperature(f64),
    SetLocalBoilTemperature(f64),

    DismissBoilAdditionAlert,
    CancelOrFinishSession,
    PressSet,
    DisableSpargeWaterAlert,
    ResetRecipeInterrupted,
    Disconnect(DisconnectOption),

    // TODO: what is the value here?
    SetSpargeProgress(SpargeProgress),

    UpdateStep {
        step_number: StepNumber,
        temperature: f64,

        // TODO: is this actually minutes?
        time_minutes: u8,
    },
    SkipToStep {
        step_number: StepNumber,
        can_edit_minutes: u8,
        time_left_minutes: u8,
        time_left_seconds: u8,
        skip_ramp: bool,
        disable_add_grain: bool,
    },

    InteractionComplete,
    SkipToInteraction(InteractionCode),

    SetSpargeCounterActive(bool),
    SetBoilControlActive(bool),
    SetManualPowerControlActive(bool),
    SetSpargeAlertModeActive(bool),
}

impl Command {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut output = String::with_capacity(COMMAND_LEN);

        match self {
            Self::Reset => {
                output.push('Z');
                output.push(',');
            }

            Self::GetFirmwareVersion => {
                output.push('X');
                output.push(',');
            }

            Self::GetVoltageAndUnits => {
                output.push('g');
                output.push(',');
            }

            Self::GetBoilTemperature => {
                output.push('M');
                output.push(',');
            }

            Self::ToggleHeatActive => {
                output.push('H');
                output.push(',');
            }

            Self::SetHeatActive(active) => {
                output.push('K');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }
                output.push(',');
            }

            Self::TogglePumpActive => {
                output.push('P');
            }

            Self::SetPumpActive(active) => {
                output.push('L');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }
                output.push(',');
            }

            Self::EnableDelayedHeatTimer {
                minutes,
                seconds,
            } => {
                output.push('B');
                output.push_str(minutes.to_string().as_ref());
                output.push(',');
                output.push_str(seconds.to_string().as_ref());
                output.push(',');
            }

            Self::CancelActiveTimer => {
                output.push('C');
                output.push(',');
            }

            Self::UpdateActiveTimer(delay) => match delay {
                Delay::MinutesSeconds(minutes, seconds) => {
                    output.push('W');
                    output.push_str(minutes.to_string().as_ref());
                    output.push(',');
                    output.push_str(seconds.to_string().as_ref());
                    output.push(',');
                }

                Delay::Minutes(minutes) => {
                    output.push('S');
                    output.push_str(minutes.to_string().as_ref());
                    output.push(',');
                }
            },

            Self::PauseOrResumeActiveTimer => {
                output.push('G');
                output.push(',');
            }

            Self::IncrementTargetTemperature => {
                output.push('U');
                output.push(',');
            }

            Self::DecrementTargetTemperature => {
                output.push('D');
                output.push(',');
            }

            Self::SetTargetTemperature(temp) => {
                output.push('$');
                output.push_str(temp.to_string().as_ref());
                output.push(',');
            }

            Self::SetLocalBoilTemperature(temp) => {
                output.push('E');
                output.push_str(temp.to_string().as_ref());
                output.push(',');
            }

            Self::DismissBoilAdditionAlert => {
                output.push('A');
                output.push(',');
            }

            Self::CancelOrFinishSession => {
                output.push('F');
                output.push(',');
            }

            Self::PressSet => {
                output.push('T');
                output.push(',');
            }

            Self::DisableSpargeWaterAlert => {
                output.push('V');
                output.push(',');
            }

            Self::ResetRecipeInterrupted => {
                output.push('!');
                output.push(',');
            }

            Self::SetSpargeProgress(progress) => {
                output.push_str("b$");
                output.push_str(progress.to_string().as_ref());
                output.push(',');
            }

            Self::UpdateStep {
                step_number,
                temperature,
                time_minutes,
            } => {
                output.push('a');
                output.push_str(step_number.to_string().as_ref());
                output.push(',');
                output.push_str(temperature.to_string().as_ref());
                output.push(',');
                output.push_str(time_minutes.to_string().as_ref());
                output.push(',');
            }

            Self::SkipToStep {
                step_number,
                can_edit_minutes,
                time_left_minutes,
                time_left_seconds,
                skip_ramp,
                disable_add_grain,
            } => {
                output.push('N');
                output.push_str(step_number.to_string().as_ref());
                output.push(',');
                output.push_str(can_edit_minutes.to_string().as_ref());
                output.push(',');
                output.push_str(time_left_minutes.to_string().as_ref());
                output.push(',');
                output.push_str(time_left_seconds.to_string().as_ref());
                output.push(',');
                output.push(if *skip_ramp {
                    '1'
                } else {
                    '0'
                });
                output.push(',');
                output.push(if *disable_add_grain {
                    '1'
                } else {
                    '0'
                });
                output.push(',');
            }

            Self::InteractionComplete => {
                output.push('I');
                output.push(',');
            }

            Self::SkipToInteraction(code) => {
                output.push('c');
                output.push_str(code.to_string().as_ref());
                output.push(',');
            }

            Self::Disconnect(option) => {
                output.push('Q');

                match option {
                    DisconnectOption::ManualMode => output.push('0'),
                    DisconnectOption::CancelSession => output.push('1'),
                    DisconnectOption::AutomaticMode => output.push('2'),
                }

                output.push(',');
            }

            Self::SetSpargeCounterActive(active) => {
                output.push('d');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }

                output.push(',');
            }

            Self::SetBoilControlActive(active) => {
                output.push('e');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }

                output.push(',');
            }

            Self::SetManualPowerControlActive(active) => {
                output.push('f');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }

                output.push(',');
            }

            Self::SetSpargeAlertModeActive(active) => {
                output.push('h');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }

                output.push(',');
            }
        }

        finish_command(output)
    }
}

pub(crate) fn finish_command(mut command_str: String) -> Vec<u8> {
    for _ in 0..(COMMAND_LEN - command_str.len()) {
        command_str.push(' ');
    }

    command_str.into()
}
