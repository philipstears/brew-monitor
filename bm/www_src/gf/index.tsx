import * as React from "react";
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
        };

        let ws = new WebSocket(this.state.ws_url);
        ws.onmessage = event => this.handleWebSocketMessage(event);
    }

    render = () => (
        <React.Fragment>
            <div id="bm-overview-panel">
                <Heat
                    command_url={this.state.command_url}
                    status1={this.state.status1}
                    temp={this.state.temp} />

                <Pump
                    command_url={this.state.command_url}
                    data={this.state.status1} />
            </div>
            <div id="bm-detail-panel">
                <div>
                    <Recipe
                        command_url={this.state.command_url}
                        recipe_url={this.state.recipe_url}
                        status1={this.state.status1}
                        status2={this.state.status2}
                    />
                </div>
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

interface RecipeState {
    recipe: Proto.Recipe;
}

interface RecipeProps {
    command_url: string;
    recipe_url: string;
    status1: Proto.Status1Data;
    status2: Proto.Status2Data;
}

export class Recipe extends React.Component<RecipeProps, RecipeState> {
    constructor(props: RecipeProps) {
        super(props);

        this.state = {
            recipe: {
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
                    { "temperature": 70, "minutes": 3 },
                    { "temperature": 75, "minutes": 3 },
                    { "temperature": 80, "minutes": 3 }
                ]
            },
        };
    }

    render = () => (
        <React.Fragment>
            <div>
                {this.props.status1.auto_mode_active ? "Recipe Active" : "Recipe Not Active"}
            </div>
            <div>
                <button onClick={this.handleSendRecipe}>
                    Send Recipe
                </button>
            </div>
            <div>
                <button onClick={this.handleCancelTimer}>
                    Cancel Timer
                </button>
            </div>
            <div>
                <button onClick={this.handleSkipToAddGrain}>
                    Skip to Add Grain
                </button>
            </div>
            <div>
                <button onClick={this.handleCancelRecipe}>
                    Cancel Recipe
                </button>
            </div>
        </React.Fragment>
    );

    handleSendRecipe = async () => {
        await fetch(this.props.recipe_url, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(this.state.recipe),
        });
    };

    handleCancelTimer = async () => {
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

    handleCancelRecipe = async () => {
        await this.command({
            type: "Disconnect",
            data: {
                type: "CancelSession",
            }
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
