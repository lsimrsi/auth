import React, { useState, useEffect } from 'react';
import './SignUp.css';

function SignUp() {
    const [email, setEmail] = useState("");
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [authenticated, setAuthenticated] = useState(false);

    const [usernameTimer, setUsernameTimer] = useState("");

    const [emailError, setEmailError] = useState("");
    const [usernameError, setUsernameError] = useState("");
    const [passwordError, setPasswordError] = useState("");
    const [generalError, setGeneralError] = useState("");

    const onSubmit = async e => {
        e.preventDefault();

        let data = {
            email,
            username,
            password,
        }

        let res = await fetch(`/auth-db/add-user`, {
            method: 'POST',
            body: JSON.stringify(data),
            headers: {
                'Content-Type': 'application/json'
            }
        });

        let json = await res.json();

        checkErrors(json);
        if (json && json.type === "success") {
            localStorage.setItem('authapp', json.data);
            setAuthenticated(true);
        }
    }

    const onInputChange = e => {
        switch (e.target.name) {
            case "email": setEmail(e.target.value); break;
            case "username": setUsername(e.target.value); break;
            case "password": setPassword(e.target.value); break;
            default: break;
        }
    }

    const checkErrors = (json) => {
        setEmailError("");
        setUsernameError("");
        setPasswordError("");
        setGeneralError("");

        if (!json) return;
        if (!json.type === "error") return;

        switch (json.context) {
            case "email": setEmailError(json.data); break;
            case "username": setUsernameError(json.data); break;
            case "password": setPasswordError(json.data); break;
            case "general": setGeneralError(json.data); break;
            default: break;
        }
    }

    useEffect(() => {
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

            let json = await res.json();

            checkErrors(json);
        }

        const onGoogleSignInFailed = (e) => {
            console.log('e', e);
        }

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

    useEffect(() => {
        const onUsernameInputChange = async () => {
            let data = {
                email: "",
                username,
                password: "",
            }

            let res = await fetch(`/auth-db/check-username`, {
                method: 'POST',
                body: JSON.stringify(data),
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            let json = await res.json();

            checkErrors(json);
        }
        clearTimeout(usernameTimer);
        setUsernameTimer(setTimeout(onUsernameInputChange, 500));
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [username]);

    return (
        <main id="sign-up">
            {!authenticated &&
            <div id="sign-up-content">
                <h1>Sign Up</h1>
                <form onSubmit={onSubmit}>
                    <input name="email" placeholder="Email" onChange={onInputChange} value={email} />
                    <p className="error">{emailError}</p>
                    <input name="username" placeholder="Username" onChange={onInputChange} value={username} />
                    <p className="error">{usernameError}</p>
                    <input name="password" placeholder="Password" onChange={onInputChange} value={password} type="password" />
                    <p className="error">{passwordError}</p>
                    <input type="submit" value="Submit" />
                    <p className="error">{generalError}</p>
                </form>
                <div id="gs2"></div>
            </div>}

            {authenticated &&
            <div id="success-content">
                <h1>Success!</h1>
            </div>}
        </main>
    )
}

export default SignUp;