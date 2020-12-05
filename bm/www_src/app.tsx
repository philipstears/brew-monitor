import * as React from "react";
import { Grainfather } from "./gf/index";

import {
  HashRouter as Router,
  Switch,
  Route,
  Link
} from "react-router-dom";

export interface AppProps {
}

export const App = (props: AppProps) => (
    <Router>
        <Switch>
            <Route path="/">
                <Grainfather />
            </Route>
        </Switch>
    </Router>
);
