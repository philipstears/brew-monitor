import * as React from "react";
import * as Modal from "react-modal";
import * as Proto from "./types";

export interface TimerProgressProps {
    timer: Proto.TimerData,
}

export class TimerProgress extends React.Component<TimerProgressProps, {}> {
    render = () => (
        <div className="progress-bar-outer">
            <div className="progress-bar-inner time" style={ { width: this.percentage().toString() + "%" } } >
            </div>
            <div className="progress-bar-label">
                {this.renderTime()}
            </div>
        </div>
    );

    renderTime = () => {
        let timer = this.props.timer;

        if ( timer.remaining_minutes == 0 ) {
            return <>&nbsp;</>;
        }
        else if ( timer.remaining_minutes == 1 ) {
            // The final minute gets rendered as a 60 second countdown
            return <>{timer.remaining_seconds} seconds remaining</>;
        }
        else {
            return <>{timer.remaining_minutes - 1} minutes and {timer.remaining_seconds} seconds remaining</>;
        }
    };

    percentage() {
        let timer = this.props.timer;

        if ( timer.remaining_minutes == 0 ) {
            return 100;
        }
        else if ( timer.remaining_minutes == 1 ) {
            // The final minute gets rendered as a 60 second countdown
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
