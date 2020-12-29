import * as React from "react";
import * as Modal from "react-modal";
import * as Proto from "./types";

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
