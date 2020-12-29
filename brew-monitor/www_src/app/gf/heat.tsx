import * as React from "react";
import * as Proto from "./types";
import { Client }  from "./client";

export interface HeatProps {
    client: Client,
    status1: Proto.Status1Data,
    temp: Proto.TempData,
}

export class Heat extends React.Component<HeatProps, {}> {
    render = () => (
        <div className="heat-controls">
            <button className="heat-decrease" onClick={this.handleDownClick}>-</button>
            <button
                className={"heat-button " + (this.props.status1.heat_active ? "on" : "off")}
                onClick={this.handleHeatToggleClick}>
                <div className="temp-display">
                    <div className="temp-display-current">{this.props.temp.current}°C</div>
                    <div className="temp-display-desired">{this.props.temp.desired > 100 ? "∞" : this.props.temp.desired.toString() + "°C"}</div>
                </div>
            </button>
            <button className="heat-increase" onClick={this.handleUpClick}>+</button>
        </div>
    );

    handleHeatToggleClick = async () => {
        this.props.client.toggleHeat();
    };

    handleUpClick = async () => {
        this.props.client.incrementTargetTemperature();
    };

    handleDownClick = async () => {
        this.props.client.decrementTargetTemperature();
    };
}
