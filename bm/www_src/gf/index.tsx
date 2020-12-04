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
            <div>Grainfather!</div>
            <div><Heat command_url={this.state.command_url} data={this.state.status1} /></div>
            <div><Pump command_url={this.state.command_url} data={this.state.status1} /></div>
            <div><Temp command_url={this.state.command_url} data={this.state.temp} /></div>
            <div>
                <Recipe
                    recipe_url={this.state.recipe_url}
                    status1={this.state.status1}
                    status2={this.state.status2}
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
        }
    };
}

export interface GrainfatherData<T> {
    command_url: string,
    data: T;
}

export class Pump extends React.Component<GrainfatherData<Proto.Status1Data>, {}> {
    render = () => (
        <button onClick={this.handleClick}>
            {this.props.data.pump_active ? "Pump On" : "Pump Off"}
        </button>
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

export class Heat extends React.Component<GrainfatherData<Proto.Status1Data>, {}> {
    render = () => (
        <div>
            <button onClick={this.handleClick}>
                {this.props.data.heat_active ? "Heat On" : "Heat Off"}
            </button>
            <div>
                {this.props.data.step_ramp_active ? "Ramping Heat for Step" : ""}
            </div>
        </div>
    );

    handleClick = async () => {
        let command = {
            type: "ToggleHeatActive",
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

export class Temp extends React.Component<GrainfatherData<Proto.TempData>, {}> {
    render = () => (
        <div className="temperature-controller">
            <button onClick={this.handleDownClick}>-</button>
            <div className="temperature-controller-display">{this.props.data.current} / {this.props.data.desired}</div>
            <button onClick={this.handleUpClick}>+</button>
        </div>
    );

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
            <button onClick={this.handleSendRecipe}>
                Send Recipe
            </button>
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
}
