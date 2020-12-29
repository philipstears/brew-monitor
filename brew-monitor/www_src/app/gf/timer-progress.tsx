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
            let remaining_minutes = timer.remaining_minutes - 1;
            let remaining_seconds = timer.remaining_seconds;

            // More fun from the controller :/
            if ( remaining_seconds == 60 ) {
                remaining_minutes += 1;
                remaining_seconds = 0;
            }

            // TODO: look into the Mozilla localization library
            let minutes_phrase = remaining_minutes == 1 ? `${remaining_minutes} minute` : `${remaining_minutes} minutes`;
            let seconds_phrase = remaining_seconds == 1 ? `${remaining_seconds} second` : `${remaining_seconds} seconds`;

            return <>{minutes_phrase} and {seconds_phrase} remaining</>;
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
            let total_seconds = 60 * (timer.total_start_time - 1);
            let remaining_seconds = (60 * (timer.remaining_minutes - 1)) + timer.remaining_seconds;
            return (((total_seconds - remaining_seconds) / total_seconds) * 100) << 0;
        }
    }
}
