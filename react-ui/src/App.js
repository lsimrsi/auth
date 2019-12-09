import React, { useState, useEffect } from 'react';
import jwt from 'jsonwebtoken';
import {
  Switch,
  Route,
  NavLink,
  useHistory,
  useLocation,
} from "react-router-dom";

import SignIn from './pages/SignIn';
import Users from './pages/Users';
import Home from './pages/Home';

import 'normalize.css';
import './App.css';

function App() {
  const [username, usernameSet] = useState("");
  const [authenticated, authenticatedSet] = useState(false);
  let history = useHistory();
  let location = useLocation();

  useEffect(() => {
    let token = localStorage.getItem("authapp");
    if (!token) return;

    let res = jwt.decode(token);
    if (res.exp * 1000 > Date.now()) {
      usernameSet(res.sub);
      authenticatedSet(true);
    }
  }, []);

  const signOut = () => {
    localStorage.removeItem("authapp");
    authenticatedSet(false);
    console.log('location.pathname', location.pathname);
    if (location.pathname === "/users") {
      history.push("/sign-in");
    }
  }

  return (
      <div className="app">
        <header>
          <nav>
            <NavLink activeClassName="active" to="/home">Home</NavLink>
            {authenticated && <span>{username}</span>}
            {authenticated && <button onClick={signOut}>Sign Out</button>}
            {!authenticated && <NavLink activeClassName="active" to="/sign-in">Sign In</NavLink>}
            <NavLink activeClassName="active" to="/users">Users</NavLink>
          </nav>
        </header>
        <Switch>
          <Route path="/sign-in">
            <SignIn authenticatedSet={authenticatedSet} />
          </Route>
          <Route path="/users">
            <Users />
          </Route>
          <Route path="/">
            <Home />
          </Route>
        </Switch>
      </div>
  );
}

export default App;
