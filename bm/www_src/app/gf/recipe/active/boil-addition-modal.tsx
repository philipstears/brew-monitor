import * as React from "react";
import * as Modal from "react-modal";

import * as Proto from "../../types";
import { Client }  from "../../client";

interface BoilAdditionModalProps {
    client: Client;
    status1: Proto.Status1Data;
    boilAlertState: Proto.BoilAlertStateData;
}

export class BoilAdditionModal extends React.Component<BoilAdditionModalProps, {}> {
    constructor(props: BoilAdditionModalProps) {
        super(props);
    }

    render = () => (
        <Modal
            isOpen={this.props.boilAlertState.visible}
            className="bm-modal"
            overlayClassName="bm-modal-overlay"
        >
            <h2 className="bm-modal-header">Add Boil Addition</h2>
            <p className="bm-modal-body">
                Press "Addition Added" to dismiss alert.
            </p>
            <div className="bm-modal-footer">
                <button onClick={this.handleDismissBoilAlert}>
                    Addition Added
                </button>
            </div>
        </Modal>
    );

    handleDismissBoilAlert = async () => {
        await this.props.client.dismissBoilAlert();
    };
}
