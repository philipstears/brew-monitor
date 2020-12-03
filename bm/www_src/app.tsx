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
      <nav>
        <ul>
          <li><Link to="/">Home</Link></li>
        </ul>
      </nav>

    <div id="content">
      <Switch>
        <Route path="/">
          <Home />
        </Route>
      </Switch>
    </div>
  </Router>
);

const Home = () => (
    <div>
      <article>
        <h2>Home</h2>
        <Grainfather />
      </article>
    </div>
);
