// Useful information at
//   https://byo.com/article/calculating-water-usage-advanced-brewing/

// NOTE: sparge heater takes 20m to raise 18 litres to 75 deg
// docs recommend to start heating once doughed in

pub fn mash_water_metric(grain_bill_kg: f64) -> f64 {
    // Litres
    3.5 + 2.7 * grain_bill_kg
}

pub fn mash_water_imperial(grain_bill_lb: f64) -> f64 {
    // US gallons
    0.9 + 0.34 * grain_bill_lb
}

pub fn sparge_water_metric(final_volume_l: f64, grain_bill_kg: f64) -> f64 {
    let water_loss_in_boil_and_trub = 5.0;
    let pre_boil_volume = final_volume_l + water_loss_in_boil_and_trub;
    let mash_water_volume = mash_water_metric(grain_bill_kg);

    // NOTE: https://byo.com/article/calculating-water-usage-advanced-brewing/ uses 0.9
    // litres per kg for this
    let water_loss_in_grain = grain_bill_kg * 0.8;

    (pre_boil_volume - mash_water_volume) + water_loss_in_grain
}
