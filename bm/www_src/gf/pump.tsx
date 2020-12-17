import * as React from "react";
import * as Proto from "./types";
import { Client }  from "./client";

export interface PumpProps {
    client: Client,
    data: Proto.Status1Data;
}

export class Pump extends React.Component<PumpProps, {}> {
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
        await this.props.client.togglePump();
    };
}
