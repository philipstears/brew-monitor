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

            status1: { heat_active: false, pump_active: false },
            status2: {},
            temp: { desired: 0, current: 0 },
        };

        let ws = new WebSocket(this.state.ws_url);
        ws.onmessage = event => this.handleWebSocketMessage(event);
    }

    render = () => (
        <React.Fragment>
            <div>Grainfather!</div>
            <div><Heat command_url={this.state.command_url} data={this.state.status1} /></div>
            <div><Pump command_url={this.state.command_url} data={this.state.status1} /></div>
            <div>{this.state.temp.current} / {this.state.temp.desired}</div>
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
        <button onClick={this.handleClick}>
            {this.props.data.heat_active ? "Heat On" : "Heat Off"}
        </button>
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
