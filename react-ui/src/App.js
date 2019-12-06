import React from 'react';
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

  return (
    <Router>
      <div className="app">
        <header>
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
