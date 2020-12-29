import * as React from "react";
import * as Proto from "./types";

import { Client }  from "./client";

import { Countdown } from "./recipe/countdown";
import { Inactive } from "./recipe/inactive";
import { Active } from "./recipe/active";

interface RecipeProps {
    client: Client;
    status1: Proto.Status1Data;
    status2: Proto.Status2Data;
    timer: Proto.TimerData;
    boilAlertState: Proto.BoilAlertStateData;
    spargeWaterAlertState: Proto.HeatSpargeWaterAlertStateData;
    recipe: Proto.Recipe;
    temp: Proto.TempData;
}

export class Recipe extends React.Component<RecipeProps, {}> {
    constructor(props: RecipeProps) {
        super(props);
    }

    render() {
        if (!this.props.status1.auto_mode_active) {
            return <Inactive client={this.props.client} recipe={this.props.recipe} />;
        }

        if (this.props.status1.delayed_heat_mode_active) {
            return <Countdown client={this.props.client} timer={this.props.timer} />;
        }

        return <Active
                   client={this.props.client}
                   status1={this.props.status1}
                   status2={this.props.status2}
                   timer={this.props.timer}
                   boilAlertState={this.props.boilAlertState}
                   spargeWaterAlertState={this.props.spargeWaterAlertState}
                   recipe={this.props.recipe}
                   temp={this.props.temp} />;
    }
}
