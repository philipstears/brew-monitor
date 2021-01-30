use super::command::{finish_command, COMMAND_LEN};
use std::fmt::Write;

/// The amount of time to wait before automatically starting a recipe.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RecipeDelay {
    None,
    MinutesSeconds(u16, u8),
}

/// The temperature and duration for a step in the mashing process for a recipe.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct MashStep {
    pub temperature: u8,
    pub minutes: u8,
}

/// All the information required by the Grainfather controller in its
/// automatic mode.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Recipe {
    /// The total duration of the boil
    pub boil_time: u8,

    /// The volume of water present for the mash
    pub mash_volume: f64,

    /// The volume of water added during the sparge
    pub sparge_volume: f64,

    /// Determines whether the controller shows a water treatment
    /// prompt on the countdown to heating the strike water.
    ///
    /// This isn't available in the grainfather mobile app.
    #[serde(default)]
    pub show_water_treatment_alert: bool,

    /// Controls whether the on-controller sparge counter is shown during the
    /// sparge.
    pub show_sparge_counter: bool,

    /// Controls whether the controller will prompt to heat the sparge water.
    pub show_sparge_alert: bool,

    /// The amount of time to wait before starting to heat the strike water.
    pub delay: RecipeDelay,

    /// ?
    #[serde(default)]
    pub skip_start: bool,

    /// The name of the recipe shown on the controller, 19 characters maximum.
    pub name: String,

    /// ?
    #[serde(default)]
    pub hop_stand_time: u8,

    /// Controls whether the boil power can be controlled using the arrows on the
    /// controller during the boil.
    #[serde(default)]
    pub boil_power_mode: bool,

    // NOTE: according to kingpulsar, this may not be implemented
    #[serde(default)]
    pub strike_temp_mode: bool,

    /// The times (from the end of the boil) at which additions should be added to the boil
    pub boil_steps: Vec<u8>,

    /// The times and temperatures of each mash step, in order.
    pub mash_steps: Vec<MashStep>,
}

impl Recipe {
    pub fn to_commands(&self) -> Vec<Vec<u8>> {
        // TODO: this can be computed
        let mut commands = Vec::with_capacity(10);

        commands.push({
            let mut command = String::with_capacity(COMMAND_LEN);

            write!(
                command,
                "R{},{},{:.2},{:.2},",
                self.boil_time,
                self.mash_steps.len(),
                self.mash_volume,
                self.sparge_volume
            )
            .unwrap();

            finish_command(command)
        });

        commands.push({
            let mut command = String::with_capacity(COMMAND_LEN);

            write!(
                command,
                "{},{},{},{},{},",
                if self.show_water_treatment_alert {
                    '1'
                } else {
                    '0'
                },
                if self.show_sparge_counter {
                    '1'
                } else {
                    '0'
                },
                if self.show_sparge_alert {
                    '1'
                } else {
                    '0'
                },
                if let RecipeDelay::MinutesSeconds(_, _) = self.delay {
                    '1'
                } else {
                    '0'
                },
                if self.skip_start {
                    '1'
                } else {
                    '0'
                },
            )
            .unwrap();

            finish_command(command)
        });

        commands.push({
            let mut command = String::with_capacity(COMMAND_LEN);
            command.push_str(self.name.as_ref());
            finish_command(command)
        });

        commands.push({
            let mut command = String::with_capacity(COMMAND_LEN);

            write!(
                command,
                "{},{},{},{},",
                self.hop_stand_time,
                self.boil_steps.len(),
                if self.boil_power_mode {
                    '1'
                } else {
                    '0'
                },
                if self.strike_temp_mode {
                    '1'
                } else {
                    '0'
                },
            )
            .unwrap();

            finish_command(command)
        });

        for boil_step in self.boil_steps.iter() {
            commands.push({
                let mut command = String::with_capacity(COMMAND_LEN);
                write!(command, "{},", boil_step).unwrap();
                finish_command(command).into()
            })
        }

        if self.strike_temp_mode {
            commands.push({
                let mut command = String::with_capacity(COMMAND_LEN);
                command.push('0');
                finish_command(command)
            })
        }

        for MashStep {
            temperature,
            minutes,
        } in self.mash_steps.iter()
        {
            commands.push({
                let mut command = String::with_capacity(COMMAND_LEN);
                write!(command, "{}:{},", temperature, minutes).unwrap();
                finish_command(command)
            })
        }

        if let RecipeDelay::MinutesSeconds(minutes, seconds) = self.delay {
            commands.push({
                let mut command = String::with_capacity(COMMAND_LEN);
                write!(command, "{},{},", minutes, seconds).unwrap();
                finish_command(command).into()
            })
        }

        commands
    }
}

impl Default for Recipe {
    fn default() -> Self {
        Self {
            boil_time: 60,
            mash_volume: 13.25,
            sparge_volume: 14.64,
            show_water_treatment_alert: false,
            show_sparge_counter: true,
            show_sparge_alert: true,
            delay: RecipeDelay::None,
            skip_start: false,
            name: String::default(),
            hop_stand_time: 0,
            boil_power_mode: false,
            strike_temp_mode: false,
            boil_steps: Vec::with_capacity(4),
            mash_steps: Vec::with_capacity(4),
        }
    }
}
