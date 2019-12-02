import React, {useState, useEffect} from 'react';
import './App.css';

function App() {
  const [email, setEmail] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const onSubmit = async e => {
    e.preventDefault();
    console.log('submit');

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

    console.log('json', json);
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
    console.log('googleUser', googleUser);

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
    if (!json) return;

    console.log('json', json);
  }

  const onGoogleSignInFailed = (e) => {
    console.log('e', e);
  }

  return (
    <div className="app">
      <header>
      </header>
      <form onSubmit={onSubmit}>
        <input name="email" placeholder="Email" onChange={onInputChange} value={email} />
        <input name="username" placeholder="Username" onChange={onInputChange} value={username} />
        <input name="password" placeholder="Password" onChange={onInputChange} value={password} />
        <input type="submit" value="Submit" />
      </form>
      <div id="gs2"></div>
    </div>
  );
}

export default App;
