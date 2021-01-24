import * as React from "react";
import * as Proto from "./types";

import { Client }  from "./client";
import { Pump } from "./pump";
import { Heat } from "./heat";
import { Recipe } from "./recipe";

export interface GrainfatherProps {
}

export interface GrainfatherState {
    client: Client;
    ws_url: string;

    status1: Proto.Status1Data;
    status2: Proto.Status2Data;
    temp: Proto.TempData;
    timer: Proto.TimerData;
    boil_alert_state: Proto.BoilAlertStateData;
    sparge_water_alert_state: Proto.HeatSpargeWaterAlertStateData;

    recipe: Proto.Recipe;
}

export class Grainfather extends React.Component<GrainfatherProps, GrainfatherState> {
    constructor(props: GrainfatherProps) {
        super(props);

        this.state = {
            client: new Client(`${window.location.protocol}//${window.location.host}/gf`),
            ws_url: `ws://${window.location.host}/gf/ws`,

            status1: Proto.defaultStatus1(),
            status2: Proto.defaultStatus2(),
            temp: Proto.defaultTemp(),
            timer: Proto.defaultTimer(),
            boil_alert_state: Proto.defaultBoilAlertState(),
            sparge_water_alert_state: Proto.defaultHeatSpargeWaterAlertState(),

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
                    client={this.state.client}
                    status1={this.state.status1}
                    temp={this.state.temp}
                />

                <Pump
                    client={this.state.client}
                    data={this.state.status1}
                />
            </div>
            <div id="bm-detail-panel">
                <Recipe
                    client={this.state.client}
                    status1={this.state.status1}
                    status2={this.state.status2}
                    timer={this.state.timer}
                    boilAlertState={this.state.boil_alert_state}
                    spargeWaterAlertState={this.state.sparge_water_alert_state}
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
            case "Timer":
                this.setState({...this.state, timer: notification.data});
                break;
            case "BoilAlertState":
                this.setState({...this.state, boil_alert_state: notification.data});
                break;
            case "HeatSpargeWaterAlertState":
                this.setState({...this.state, sparge_water_alert_state: notification.data});
                break;
        }
    };
}
