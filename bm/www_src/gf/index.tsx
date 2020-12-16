import * as React from "react";
import * as Modal from "react-modal";
import * as Proto from "./types";

export interface GrainfatherProps {
}

export interface GrainfatherState {
    command_url: string;
    recipe_url: string;
    ws_url: string;

    status1: Proto.Status1Data;
    status2: Proto.Status2Data;
    temp: Proto.TempData;
    timer: Proto.TimerData;
    boil_alert_state: Proto.BoilAlertStateData;

    recipe: Proto.Recipe;
}

export class Grainfather extends React.Component<GrainfatherProps, GrainfatherState> {
    constructor(props: GrainfatherProps) {
        super(props);

        this.state = {
            command_url: `${window.location.protocol}//${window.location.host}/gf/command`,
            recipe_url: `${window.location.protocol}//${window.location.host}/gf/recipe`,
            ws_url: `ws://${window.location.host}/gf/ws`,

            status1: Proto.defaultStatus1(),
            status2: Proto.defaultStatus2(),
            temp: Proto.defaultTemp(),
            timer: Proto.defaultTimer(),
            boil_alert_state: Proto.defaultBoilAlertState(),

            recipe: {
                "boil_temperature": 55,
                "boil_time": 9,
                "mash_volume": 13.25,
                "sparge_volume": 14.64,
                "show_water_treatment_alert": false,
                "show_sparge_counter": true,
                "show_sparge_alert": true,
                "delay": { "type": "MinutesSeconds", "data": [5, 0] },
                "skip_start": false,
                "name": "STIPA",
                "hop_stand_time": 0,
                "boil_power_mode": false,
                "strike_temp_mode": false,
                "boil_steps": [
                    9,
                    6,
                    3
                ],
                "mash_steps": [
                    { "temperature": 25, "minutes": 3 },
                    { "temperature": 35, "minutes": 3 },
                    { "temperature": 45, "minutes": 3 }
                ]
            },
        };

        let ws = new WebSocket(this.state.ws_url);
        ws.onmessage = event => this.handleWebSocketMessage(event);
    }

    render = () => (
        <React.Fragment>
            <div id="bm-overview-panel">
                <h2 className="bm-overview-panel-header">{this.state.recipe.name}</h2>

                <Heat
                    command_url={this.state.command_url}
                    status1={this.state.status1}
                    temp={this.state.temp}
                />

                <Pump
                    command_url={this.state.command_url}
                    data={this.state.status1}
                />
            </div>
            <div id="bm-detail-panel">
                <Recipe
                    command_url={this.state.command_url}
                    recipe_url={this.state.recipe_url}
                    status1={this.state.status1}
                    status2={this.state.status2}
                    timer={this.state.timer}
                    boil_alert_state={this.state.boil_alert_state}
                    recipe={this.state.recipe}
                    temp={this.state.temp}
                />
            </div>
        </React.Fragment>
    );

    handleWebSocketMessage = (event: MessageEvent) => {
        let notification: Proto.Notification = JSON.parse(event.data);

        switch (notification.type) {
            case "Status1":
                this.setState({...this.state, status1: notification.data});
                break;
            case "Status2":
                this.setState({...this.state, status2: notification.data});
                break;
            case "Temp":
                this.setState({...this.state, temp: notification.data});
                break;
            case "DelayedHeatTimer":
                this.setState({...this.state, timer: notification.data});
                break;
            case "BoilAlertState":
                this.setState({...this.state, boil_alert_state: notification.data});
                break;
        }
    };
}

export interface GrainfatherData<T> {
    command_url: string,
    data: T;
}

export class Pump extends React.Component<GrainfatherData<Proto.Status1Data>, {}> {
    render = () => (
        <div className="pump-controls">
            <button
                className={"pump-button " + (this.props.data.pump_active ? "on" : "off")}
                onClick={this.handleClick}>
                Pump
            </button>
        </div>
    );

    handleClick = async () => {
        let command = {
            type: "TogglePumpActive",
        };

        await fetch(this.props.command_url, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(command),
        });
    };
}

export interface HeatProps {
    command_url: string,
    status1: Proto.Status1Data,
    temp: Proto.TempData,
}

export class Heat extends React.Component<HeatProps, {}> {
            // <div>
            //     {this.props.status1.step_ramp_active ? "Ramping Heat for Step" : ""}
            // </div>

    render = () => (
        <div className="heat-controls">
            <button className="heat-decrease" onClick={this.handleDownClick}>-</button>
            <button
                className={"heat-button " + (this.props.status1.heat_active ? "on" : "off")}
                onClick={this.handleHeatToggleClick}>
                <div className="temp-display">
                    <div className="temp-display-current">{this.props.temp.current}°C</div>
                    <div className="temp-display-desired">{this.props.temp.desired}°C</div>
                </div>
            </button>
            <button className="heat-increase" onClick={this.handleUpClick}>+</button>
        </div>
    );

    handleHeatToggleClick = async () => {
        this.command({
            type: "ToggleHeatActive",
        });
    };

    handleUpClick = async () => {
        this.command({
            type: "IncrementTargetTemperature",
        });
    };

    handleDownClick = async () => {
        this.command({
            type: "DecrementTargetTemperature",
        });
    };

    command = async (command: any) => {
        await fetch(this.props.command_url, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(command),
        });
    };
}

interface RecipeProps {
    command_url: string;
    recipe_url: string;
    status1: Proto.Status1Data;
    status2: Proto.Status2Data;
    timer: Proto.TimerData;
    boil_alert_state: Proto.BoilAlertStateData;
    recipe: Proto.Recipe;
    temp: Proto.TempData;
}

export class Recipe extends React.Component<RecipeProps, {}> {
    constructor(props: RecipeProps) {
        super(props);
    }

    render() {
        if (this.props.status1.auto_mode_active) {
            if (this.props.status1.delayed_heat_mode_active) {
                return this.renderRecipeCountDown();
            }
            else {
                return this.renderRecipeActive();
            }
        }
        else {
            return this.renderRecipeInactive();
        }
    }

    renderRecipeCountDown() {
        if (this.props.timer.remaining_minutes == 0) {
            return <div>Timer Finished</div>;
        }

        return <React.Fragment>
            <h2 className="bm-detail-panel-header">
                Recipe Delay
            </h2>
            <div className="bm-detail-panel-body">
                <TimerProgress timer={this.props.timer} />
            </div>
            <div className="bm-detail-panel-footer">
                <button onClick={this.handleSkipTimer}>
                    Skip Timer
                </button>

                <button onClick={this.handleCancelRecipe}>
                    Cancel Recipe
                </button>
            </div>
        </React.Fragment>
    }

    renderRecipeActive = () => (
        <React.Fragment>
            <Modal
                isOpen={this.props.status1.interaction_mode_active}
                className="bm-modal"
                overlayClassName="bm-modal-overlay"
            >
                {this.renderInteraction()}
            </Modal>
            <Modal
                isOpen={this.props.boil_alert_state.boil_alert_visible}
                className="bm-modal"
                overlayClassName="bm-modal-overlay"
            >
                {this.renderBoilAlert()}
            </Modal>
            <h2 className="bm-detail-panel-header">
                Recipe Active: {this.stepName()}
            </h2>
            <div className="bm-detail-panel-body">
                {this.renderHeatingMashingOrBoiling()}
            </div>
            <div className="bm-detail-panel-footer">
                {this.maybeRenderSkipToAddGrain()}
                {this.maybeRenderSkipHeating()}
                {this.maybeRenderSkipToNextStep()}
                <button onClick={this.handleCancelRecipe}>
                    Cancel Recipe
                </button>
            </div>
        </React.Fragment>
    );

    stepName() {
        let step_number = this.props.status1.step_number;
        let mash_steps = this.props.recipe.mash_steps.length;

        if (this.props.timer.active == false && step_number == 1) {
            return "Mash In";
        }
        if (step_number < mash_steps) {
            return "Mash " + step_number.toString();
        }
        else if (step_number == mash_steps) {
            if (mash_steps > 1) {
                return "Mash Out";
            }
            else {
                return "Mash " + step_number.toString();
            }
        }
        else {
            return "Boil";
        }
    }

    renderRecipeInactive = () => (
        <React.Fragment>
            <h2>
                Recipe Inactive
            </h2>
            <div>
                <button onClick={this.handleSendRecipe}>
                    Send Recipe
                </button>
            </div>
        </React.Fragment>
    );

    renderInteraction() {
        switch (this.props.status1.interaction_code.type) {
            case "AddGrain":
                return this.renderInteractionAddGrain();
            case "MashOutDoneStartSparge":
                return this.renderInteractionStartSparge();
            case "Sparge":
                return this.renderInteractionSparge();
            case "BoilReached":
                return this.renderInteractionBoilReached();
            case "BoilFinished":
                return this.renderInteractionBoilFinished();
            default:
                return this.renderInteractionUnknown();
        }
    }

    renderInteractionAddGrain = () => (
        <React.Fragment>
            <h2 className="bm-modal-header">Add Grain</h2>
            <p className="bm-modal-body">
                Press "Grain Added" to start mash.
            </p>
            <div className="bm-modal-footer">
                <button onClick={this.handleSet}>
                    Grain Added
                </button>
            </div>
        </React.Fragment>
    );

    renderInteractionStartSparge = () => (
        <React.Fragment>
            <h2>Mash Out Done</h2>
            <button onClick={this.handleSet}>
                Start Sparge
            </button>
        </React.Fragment>
    );

    renderInteractionSparge = () => (
        <React.Fragment>
            <h2>Sparging</h2>
            <button onClick={this.handleSet}>
                Sparge Done
            </button>
        </React.Fragment>
    );

    renderInteractionBoilReached = () => (
        <React.Fragment>
            <h2>Boil Reached</h2>
            <button onClick={this.handleSet}>
                OK
            </button>
        </React.Fragment>
    );

    renderInteractionBoilFinished = () => (
        <React.Fragment>
            <h2>Boil Done</h2>
            <button onClick={this.handleSet}>
                OK
            </button>
        </React.Fragment>
    );

    renderInteractionUnknown = () => (
        <React.Fragment>
            <h2>Unknown Interaction: {JSON.stringify(this.props.status1.interaction_code)}</h2>
            <button onClick={this.handleSet}>
                OK
            </button>
        </React.Fragment>
    );

    renderBoilAlert = () => (
        <React.Fragment>
            <h2>Add Boil Addition</h2>
            <p>
                Press "Addition Added" to dismiss alert.
            </p>
            <button onClick={this.handleDismissBoilAlert}>
                Addition Added
            </button>
        </React.Fragment>
    );

    renderHeatingMashingOrBoiling() {
        if (this.props.status1.step_number > this.props.recipe.mash_steps.length) {
            if (this.props.timer.active) {
                return this.renderBoiling();
            }
            else {
                return this.renderHeatingToBoil();
            }
        }
        else if (this.props.timer.active) {
            return this.renderMashing();
        }
        else {
            return this.renderHeating();
        }
    }

    renderHeatingToBoil = () => (
        <>
            <div>Heating to Boil Temperature...</div>
            <HeatProgress temp={this.props.temp} />
        </>
    );

    renderBoiling = () => (
        <>
            <div>Boiling...</div>
            <TimerProgress timer={this.props.timer} />
        </>
    );

    renderHeating = () => (
        <>
            <div>Heating to Mash Temperature...</div>
            <HeatProgress temp={this.props.temp} />
        </>
    );

    renderMashing = () => (
        <>
            <div>Mashing...</div>
            <TimerProgress timer={this.props.timer} />
        </>
    );

    maybeRenderSkipToAddGrain() {
        if (this.props.status1.step_number == 1 && !this.props.timer.active) {
            return <button onClick={this.handleSkipToAddGrain}>
                Skip to Add Grain
            </button>;
        }
        else {
            return <></>;
        }
    }

    maybeRenderSkipHeating() {
        let canSkipHeating =
            this.props.status1.step_number > 1 &&
            this.props.status1.step_number <= this.props.recipe.mash_steps.length &&
            !this.props.timer.active;

        if (canSkipHeating) {
            return <button onClick={this.handleSkipHeating}>
                Skip Heating
            </button>;
        }
        else {
            return <></>;
        }
    }

    maybeRenderSkipToNextStep() {
        let canSkip =
            this.props.status1.step_number > 1 &&
            this.props.status1.step_number <= this.props.recipe.mash_steps.length &&
            this.props.timer.active;

        if (canSkip) {
            return <button onClick={this.handleSkipToNextStep}>
                Skip Step
            </button>;
        }
        else {
            return <></>;
        }
    }

    handleSendRecipe = async () => {
        await fetch(this.props.recipe_url, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(this.props.recipe),
        });
    };

    handleSkipTimer = async () => {
        await this.command({
            type: "UpdateActiveTimer",
            data: {
                type: "MinutesSeconds",
                data: [0, 1],
            }
        });
    };

    handleSkipToAddGrain = async () => {
        await this.command({
            type: "SkipToInteraction",
            data: {
                type: "AddGrain",
            }
        });
    };

    handleSkipHeating = async () => {
        await this.command({
            type: "SkipToStep",
            data: {
                step_number: this.props.status1.step_number,
                can_edit_minutes: 0,
                time_left_minutes: 0,
                time_left_seconds: 0,
                skip_ramp: true,
                disable_add_grain: true
            }
        });
    };

    handleSkipToNextStep = async () => {
        await this.command({
            type: "SkipToStep",
            data: {
                step_number: this.props.status1.step_number + 1,
                can_edit_minutes: 0,
                time_left_minutes: 0,
                time_left_seconds: 0,
                skip_ramp: false,
                disable_add_grain: true
            }
        });
    };

    handleCancelRecipe = async () => {
        await this.command({
            type: "Disconnect",
            data: {
                type: "CancelSession",
            }
        });
    };

    handleSet = async () => {
        await this.command({
            type: "PressSet",
        });
    };

    handleDismissBoilAlert = async () => {
        await this.command({
            type: "DismissBoilAdditionAlert",
        });
    };

    command = async (command: any) => {
        await fetch(this.props.command_url, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(command),
        });
    };
}

export interface HeatProgressProps {
    temp: Proto.TempData,
}

export class HeatProgress extends React.Component<HeatProgressProps, {}> {
    render = () => (
        <div className="progress-bar-outer">
            <div className="progress-bar-inner heat" style={ { width: this.percentage().toString() + "%" } } >
                &nbsp;
            </div>
        </div>
    );

    percentage() {
        return ((this.props.temp.current / this.props.temp.desired) * 100) << 0;
    }
}

export interface TimerProgressProps {
    timer: Proto.TimerData,
}

export class TimerProgress extends React.Component<TimerProgressProps, {}> {
    render = () => {
        if (this.props.timer.remaining_minutes == 0) {
            return <></>;
        }

        return <div className="progress-bar-outer">
            <div className="progress-bar-inner time" style={ { width: this.percentage().toString() + "%" } } >
            </div>
            <div className="progress-bar-label">
                {this.renderTime()}
            </div>
        </div>
    };

    renderTime = () => {
        let timer = this.props.timer;

        // The final minute gets rendered as a 60 second countdown
        if ( timer.remaining_minutes == 1 ) {
            return <>{timer.remaining_seconds} seconds remaining</>;
        }
        else {
            return <>{timer.remaining_minutes - 1} minutes and {timer.remaining_seconds} seconds remaining</>;
        }
    };

    percentage() {
        let timer = this.props.timer;

        // The final minute gets rendered as a 60 second countdown
        if ( timer.remaining_minutes == 1 ) {
            let remaining_seconds = timer.remaining_seconds;
            return (((60 - remaining_seconds) / 60) * 100) << 0;
        }
        else {
            let total_minutes = timer.total_start_time - 1;
            let remaining_minutes = timer.remaining_minutes - 1;
            return (((total_minutes - remaining_minutes) / total_minutes) * 100) << 0;
        }
    }
}
