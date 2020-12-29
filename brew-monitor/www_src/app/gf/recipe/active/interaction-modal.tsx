import * as React from "react";
import * as Modal from "react-modal";

import * as Proto from "../../types";
import { Client }  from "../../client";

interface InteractionModalProps {
    client: Client;
    status1: Proto.Status1Data;
}

export class InteractionModal extends React.Component<InteractionModalProps, {}> {
    constructor(props: InteractionModalProps) {
        super(props);
    }

    render = () => (
        <Modal
            isOpen={this.props.status1.interaction_mode_active}
            className="bm-modal"
            overlayClassName="bm-modal-overlay">
            {this.renderInteraction()}
        </Modal>
    );

    renderInteraction() {
        switch (this.props.status1.interaction_code.type) {
            case "AddGrain":
                return this.renderInteractionAddGrain();
            case "MashOutDoneStartSparge":
                return this.renderInteractionStartSparge();
            case "Sparge":
                return this.renderInteractionSparge();
            case "BoilReached":
                return this.renderInteractionBoilReached();
            case "BoilFinished":
                return this.renderInteractionBoilFinished();
            default:
                return this.renderInteractionUnknown();
        }
    }

    renderInteractionAddGrain = () => (
        <React.Fragment>
            <h2 className="bm-modal-header">Add Grain</h2>
            <p className="bm-modal-body">
                Press "Grain Added" to start mash.
            </p>
            <div className="bm-modal-footer">
                <button onClick={this.handleSet}>
                    Grain Added
                </button>
            </div>
        </React.Fragment>
    );

    renderInteractionStartSparge = () => (
        <React.Fragment>
            <h2 className="bm-modal-header">Mash Out Done</h2>
            <p className="bm-modal-body">
                Click "Start Sparge" to begin sparging.
            </p>
            <div className="bm-modal-footer">
                <button onClick={this.handleSet}>
                    Start Sparge
                </button>
            </div>
        </React.Fragment>
    );

    renderInteractionSparge = () => (
        <React.Fragment>
            <h2 className="bm-modal-header">Sparging</h2>
            <p className="bm-modal-body">
                Add sparge water incrementally, and click "Sparge Done" when done.
            </p>
            <div className="bm-modal-footer">
                <button onClick={this.handleSet}>
                    Sparge Done
                </button>
            </div>
        </React.Fragment>
    );

    renderInteractionBoilReached = () => (
        <React.Fragment>
            <h2 className="bm-modal-header">Boil Reached</h2>
            <p className="bm-modal-body">
                The boil temperature has been reached.
            </p>
            <div className="bm-modal-footer">
                <button onClick={this.handleSet}>
                    OK
                </button>
            </div>
        </React.Fragment>
    );

    renderInteractionBoilFinished = () => (
        <React.Fragment>
            <h2 className="bm-modal-header">Boil Done</h2>
            <p className="bm-modal-body">
                The end of the boil has been reached.
            </p>
            <div className="bm-modal-footer">
                <button onClick={this.handleSet}>
                    OK
                </button>
            </div>
        </React.Fragment>
    );

    renderInteractionUnknown = () => (
        <React.Fragment>
            <h2 className="bm-modal-header">Unknown Interaction</h2>
            <p className="bm-modal-body">
                {JSON.stringify(this.props.status1.interaction_code)}
            </p>
            <div className="bm-modal-footer">
                <button onClick={this.handleSet}>
                    OK
                </button>
            </div>
        </React.Fragment>
    );

    handleSet = async () => {
        await this.props.client.pressSet();
    };
}
