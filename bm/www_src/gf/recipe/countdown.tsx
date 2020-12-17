import * as React from "react";

import { TimerData } from "../types";
import { HeatProgress } from "../heat-progress";
import { TimerProgress } from "../timer-progress";
import { Client }  from "../client";

interface CountdownProps {
    client: Client;
    timer: TimerData;
}

export class Countdown extends React.Component<CountdownProps, {}> {
    constructor(props: CountdownProps) {
        super(props);
    }

    render() {
        if (this.props.timer.remaining_minutes == 0) {
            return <div>Timer Finished</div>;
        }

        return <React.Fragment>
            <h2 className="bm-detail-panel-header">
                Recipe Delay
            </h2>
            <div className="bm-detail-panel-body">
                <TimerProgress timer={this.props.timer} />
            </div>
            <div className="bm-detail-panel-footer">
                <button onClick={this.handleSkipTimer}>
                    Skip Timer
                </button>

                <button onClick={this.handleCancelRecipe}>
                    Cancel Recipe
                </button>
            </div>
        </React.Fragment>
    }

    handleSkipTimer = async () => {
        await this.props.client.updateActiveTimer({ minutes: 0, seconds: 1 });
    };

    handleCancelRecipe = async () => {
        await this.props.client.cancelSession();
    };
}
