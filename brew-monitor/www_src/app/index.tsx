import * as React from "react";
import { Grainfather } from "./gf/index";
import { Tilt, TiltProps } from "./tilt/index";
import { DHT22, DHT22Props } from "./dht22/index";

import {
    HashRouter as Router,
    Switch,
    Route,
    useParams,
} from "react-router-dom";

export interface AppProps {
}

export const App = (_props: AppProps) => (
    <Router>
        <Switch>
            <Route path="/dht22/:alias/">
                <DHT22WithAlias />
            </Route>
            <Route path="/tilt/:color/">
                <TiltWithColor />
            </Route>
            <Route path="/">
                <Grainfather />
            </Route>
        </Switch>
    </Router>
);

function DHT22WithAlias(): React.ReactElement {
    let { alias } = useParams<DHT22Props>();

    return (
        <DHT22 alias={alias} />
    );
}

function TiltWithColor(): React.ReactElement {
    let { color } = useParams<TiltProps>();

    return (
        <Tilt color={color} />
    );
}
