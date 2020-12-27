import * as React from "react";
import { Grainfather } from "./gf/index";
import { Tilt, TiltProps } from "./tilt/index";

import {
    HashRouter as Router,
    Switch,
    Route,
    Link,
    useParams
} from "react-router-dom";

export interface AppProps {
}

export const App = (props: AppProps) => (
    <Router>
        <Switch>
            <Route path="/tilt/:color/">
                <TiltWithColor />
            </Route>
            <Route path="/">
                <Grainfather />
            </Route>
        </Switch>
    </Router>
);

function TiltWithColor(): React.ReactElement {
    let { color } = useParams<TiltProps>();

    return (
        <Tilt color={color} />
    );
}
