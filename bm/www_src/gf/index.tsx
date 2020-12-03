import * as React from "react";
import * as Proto from "./types";

export interface GrainfatherProps {
}

export interface GrainfatherState {
    status1: Proto.Status1Data;
    status2: Proto.Status2Data;
    temp: Proto.TempData;
}

export class Grainfather extends React.Component<GrainfatherProps, GrainfatherState> {
    constructor(props: GrainfatherProps) {
        super(props);

        this.state = {
            status1: { heat_active: false, pump_active: false },
            status2: {},
            temp: { desired: 0, current: 0 },
        };

        let ws = new WebSocket(`ws://${window.location.host}/gf/ws`);
        ws.onmessage = event => this.handleWebSocketMessage(event);
    }

    render = () => (
        <React.Fragment>
            <div>Grainfather!</div>
            <div>{this.state.status1.heat_active ? "Heat On" : "Heat Off"}</div>
            <div>{this.state.status1.pump_active ? "Pump On" : "Pump Off"}</div>
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

