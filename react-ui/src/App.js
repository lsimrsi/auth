import React, { useState, useEffect } from 'react';
import jwt from 'jsonwebtoken';
import {
  BrowserRouter as Router,
  Switch,
  Route,
  NavLink
} from "react-router-dom";

import SignIn from './pages/SignIn';
import Users from './pages/Users';
import Home from './pages/Home';

import 'normalize.css';
import './App.css';

function App() {
  const [username, usernameSet] = useState("");
  const [authenticated, authenticatedSet] = useState(false);

  useEffect(() => {
    let token = localStorage.getItem("authapp");
    let res = jwt.decode(token);
    console.log('res', res);
    console.log('res.exp', res.exp);
    console.log('Date.now()', Date.now());
    if (res.exp * 1000 > Date.now()) {
      usernameSet(res.sub);
      authenticatedSet(true);
    }
  }, []);

  return (
    <Router>
      <div className="app">
        <header>
          {`Welcome back ${username}`}!
          <nav>
            <NavLink activeClassName="active" to="/home">Home</NavLink>
            <NavLink activeClassName="active" to="/sign-in">Sign In</NavLink>
            <NavLink activeClassName="active" to="/users">Users</NavLink>
          </nav>
        </header>
        <Switch>
          <Route path="/sign-in">
            <SignIn />
          </Route>
          <Route path="/users">
            <Users />
          </Route>
          <Route path="/">
            <Home />
          </Route>
        </Switch>
      </div>
    </Router>
  );
}

export default App;
