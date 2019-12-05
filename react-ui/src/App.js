import React from 'react';
import {
  BrowserRouter as Router,
  Switch,
  Route,
  NavLink
} from "react-router-dom";

import SignUp from './pages/SignUp';
import Users from './pages/Users';
import Home from './pages/Home';

import './App.css';

function App() {

  return (
    <Router>
      <div className="app">
        <header>
          <nav>
            <NavLink activeClassName="active" to="/home">Home</NavLink>
            <NavLink activeClassName="active" to="/sign-up">Sign Up</NavLink>
            <NavLink activeClassName="active" to="/users">Users</NavLink>
          </nav>
        </header>
        <Switch>
          <Route path="/sign-up">
            <SignUp />
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
