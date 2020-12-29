import * as React from "react";

import { Recipe } from "../types";
import { Client }  from "../client";

interface InactiveProps {
    client: Client;
    recipe: Recipe;
}

export class Inactive extends React.Component<InactiveProps, {}> {
    constructor(props: InactiveProps) {
        super(props);
    }

    render = () => (
        <React.Fragment>
            <h2 className="bm-detail-panel-header">
                Recipe Inactive
            </h2>
            <div className="bm-detail-panel-body">
            </div>
            <div className="bm-detail-panel-footer">
                <button onClick={this.handleSendRecipe}>
                    Send Recipe
                </button>
            </div>
        </React.Fragment>
    )

    handleSendRecipe = async () => {
        await this.props.client.sendRecipe(this.props.recipe);
    };
}
