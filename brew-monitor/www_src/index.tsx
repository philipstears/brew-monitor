import "./index.css";

import * as React from "react";
import * as ReactDOM from "react-dom";
import * as Modal from "react-modal";
import { App } from "./app/index";

Modal.setAppElement("#bm-app");

ReactDOM.render(
    <App />,
    document.getElementById("bm-app")
    );
