import * as React from "react";
import * as Modal from "react-modal";

import * as Proto from "../../types";
import { Client }  from "../../client";

interface HeatSpargeWaterModalProps {
    client: Client;
    status1: Proto.Status1Data;
    spargeWaterAlertState: Proto.HeatSpargeWaterAlertStateData;
}

export class HeatSpargeWaterModal extends React.Component<HeatSpargeWaterModalProps, {}> {
    constructor(props: HeatSpargeWaterModalProps) {
        super(props);
    }

    render = () => (
        <Modal
            isOpen={this.props.spargeWaterAlertState.visible}
            className="bm-modal"
            overlayClassName="bm-modal-overlay"
        >
            <h2 className="bm-modal-header">Heat Sparge Water</h2>
            <p className="bm-modal-body">
                Start heating your sparge water ready for the sparge.
            </p>
            <div className="bm-modal-footer">
                <button onClick={this.handleDismissAlert}>
                    OK
                </button>
            </div>
        </Modal>
    );

    handleDismissAlert = async () => {
        await this.props.client.dismissAlert();
    };
}
