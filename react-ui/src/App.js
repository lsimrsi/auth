import React, {useState, useEffect} from 'react';
import './App.css';

function App() {
  const [email, setEmail] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const [users, setUsers] = useState([]);

  const [emailError, setEmailError] = useState("");
  const [usernameError, setUsernameError] = useState("");
  const [passwordError, setPasswordError] = useState("");
  const [generalError, setGeneralError] = useState("");

  const onSubmit = async e => {
    e.preventDefault();

    let data = {
      email,
      username,
      pw: password,
    }

    let res = await fetch(`/auth-db/add-user`, {
      method: 'POST',
      body: JSON.stringify(data),
      headers: {
        'Content-Type': 'application/json'
      }
    });
    if (!res) return;

    let json = await res.json();
    if (!json) return;

    checkErrors(json);
  }

  const onInputChange = e => {
    switch (e.target.name) {
      case "email": setEmail(e.target.value); break;
      case "username": setUsername(e.target.value); break;
      case "password": setPassword(e.target.value); break;
      default: break;
    }
  }

  useEffect(() => {
    function addBtn() {
      window.gapi.signin2.render('gs2', {
        'scope': 'https://www.googleapis.com/auth/plus.login',
        'width': 200,
        'height': 50,
        'longtitle': true,
        'theme': 'dark',
        'onsuccess': onGoogleSignIn,
        'onfailure': onGoogleSignInFailed
      })
    }
    if (window.gapi) {
      addBtn();
    } else {
      setTimeout(addBtn, 200);
    }
  }, []);

  const onGoogleSignIn = async (googleUser) => {
    let data = {
      id_token: googleUser.getAuthResponse().id_token
    }

    let res = await fetch(`/auth/google`, {
      method: 'POST',
      body: JSON.stringify(data),
      headers: {
        'Content-Type': 'application/json'
      }
    });
    if (!res) return;
    let json = await res.json();
    console.log('res.json', json);
    if (!json) return;

    checkErrors(json);
  }

  const checkErrors = (json) => {
    setEmailError("");
    setUsernameError("");
    setPasswordError("");
    setGeneralError("");

    if (!json.type == "error") return;

    switch (json.context) {
      case "email": setEmailError(json.message); break;
      case "username": setUsernameError(json.message); break;
      case "password": setPasswordError(json.message); break;
      case "general": setGeneralError(json.message); break;
      default: break;
    }
  }

  const onGoogleSignInFailed = (e) => {
    console.log('e', e);
  }

  const getUsers = async () => {
    let res = await fetch(`/auth-db/get-users`, {
      method: 'GET',
    });
    if (!res) return;
    let json = await res.json();
    console.log('getUsers json', json);
    if (!json) return;
    checkErrors(json);
    setUsers(json.message);
  }

  return (
    <div className="app">
      <header>
      </header>
      <p className="error">{generalError}</p>
      <form onSubmit={onSubmit}>
        <input name="email" placeholder="Email" onChange={onInputChange} value={email} />
        <p className="error">{emailError}</p>
        <input name="username" placeholder="Username" onChange={onInputChange} value={username} />
        <p className="error">{usernameError}</p>
        <input name="password" placeholder="Password" onChange={onInputChange} value={password} type="password" />
        <p className="error">{passwordError}</p>
        <input type="submit" value="Submit" />
      </form>
      <div id="gs2"></div>
      <button onClick={getUsers}>Get Users</button>
      {users.map((item) => {
        return <p>{item}</p>
      })}
    </div>
  );
}

export default App;
