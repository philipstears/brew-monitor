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

    recipe_request: Proto.RecipeRequest;
    recipe: Proto.Recipe | null;
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

            recipe_request: {
                "name": "DPC",
                "params": {
                    "show_sparge_counter": true,
                    "show_sparge_alert": true,
                    "delay": { "type": "MinutesSeconds", "data": [120, 0] },
                    "boil_power_mode": true,
                }
            },

            recipe: null,
        };

        this.openWebSocket();
    }

    openWebSocket = () => {
        let ws = new WebSocket(this.state.ws_url);

        ws.onmessage = event => this.handleWebSocketMessage(event);

        ws.onerror = event => {
            console.error("The websocket encountered an error", event);
        };

        ws.onclose = event => {
            console.error("The websocket closed, re-opening", event);
            this.openWebSocket();
        }
    }

    render = () => (
        <React.Fragment>
            <div id="bm-overview-panel">
                <h2 className="bm-overview-panel-header">{this.state.recipe_request.name}</h2>

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
                    recipeRequest={this.state.recipe_request}
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
            case "ActiveRecipeChanged":
                this.setActiveRecipe(notification.data);
                break;
        }
    };

    setActiveRecipe = async (data: Proto.ActiveRecipeChangedData) => {
        let recipeUrl = `${window.location.protocol}//${window.location.host}/recipes/by-id/${data.id}/${data.version}`;

        let response = await fetch(recipeUrl, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        let recipe = await response.json();

        console.log("Got recipe", recipe);
    }
}
