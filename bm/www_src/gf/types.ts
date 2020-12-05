// -----------------------------------------------------------------------------
// Common
// -----------------------------------------------------------------------------
export type InteractionCode
    = InteractionNone
    | InteractionSkipDelayedRecipe
    | InteractionAddGrain
    | InteractionMashOutDoneStartSparge
    | InteractionSparge
    | InteractionBoilReached
    | InteractionBoilFinished
    | InteractionOther;

interface InteractionNone {
    type: "None";
}

interface InteractionSkipDelayedRecipe {
    type: "SkipDelayedRecipe";
}

interface InteractionAddGrain {
    type: "AddGrain";
}

interface InteractionMashOutDoneStartSparge {
    type: "MashOutDoneStartSparge";
}

interface InteractionSparge {
    type: "Sparge";
}

interface InteractionBoilReached {
    type: "BoilReached";
}

interface InteractionBoilFinished {
    type: "BoilFinished";
}

interface InteractionOther {
    type: "Other";
    data: string[];
}

// -----------------------------------------------------------------------------
// Notitifications
// -----------------------------------------------------------------------------
export type Notification
    = Status1Notification
    | Status2Notification
    | TempNotification
    | TimerNotification
    ;

export interface Status1Notification {
    type: "Status1";
    data: Status1Data;
}

export interface Status1Data {
    heat_active: boolean;
    pump_active: boolean;
    auto_mode_active: boolean;
    step_ramp_active: boolean;
    interaction_mode_active: boolean;
    interaction_code: InteractionCode;
    step_number: number;
    delayed_heat_mode_active: boolean;
}

export interface Status2Notification {
    type: "Status2";
    data: Status2Data;
}

export interface Status2Data {

}

export interface TempNotification {
    type: "Temp";
    data: TempData;
}

export interface TempData {
    desired: number;
    current: number;
}

export interface TimerNotification {
    type: "DelayedHeatTimer";
    data: TimerData;
}

export interface TimerData {
    active: boolean;
    remaining_minutes: number;
    remaining_seconds: number;
}

// -----------------------------------------------------------------------------
// Recipes
// -----------------------------------------------------------------------------
type RecipeDelay = RecipeDelayNone | RecipeDelayMinutesSeconds;

export interface RecipeDelayNone {
    type: "None";
}

export interface RecipeDelayMinutesSeconds {
    type: "MinutesSeconds";
    data: number[];
}

export interface RecipeMashStep {
    temperature: number;
    minutes: number;
}

export interface Recipe {
    boil_temperature: number;
    boil_time: number;
    mash_volume: number;
    sparge_volume: number;
    show_water_treatment_alert: boolean;
    show_sparge_counter: boolean;
    show_sparge_alert: boolean;
    delay: RecipeDelay;
    skip_start: boolean;
    name: string;
    hop_stand_time: number;
    boil_power_mode: boolean;
    strike_temp_mode: boolean;
    boil_steps: number[];
    mash_steps: RecipeMashStep[];
}

export function defaultStatus1(): Status1Data {
    return {
        heat_active: false,
        pump_active: false,
        auto_mode_active: false,
        step_ramp_active: false,
        interaction_mode_active: false,
        interaction_code: { type: "None" },
        step_number: 0,
        delayed_heat_mode_active: false,
    };
}

export function defaultStatus2(): Status2Data {
    return {};
}

export function defaultTemp(): TempData {
    return { desired: 0, current: 0 };
}

export function defaultTimer(): TimerData {
    return {
        active: false,
        remaining_minutes: 0,
        remaining_seconds: 0,
    };
}
