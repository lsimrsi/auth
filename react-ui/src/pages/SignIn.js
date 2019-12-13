import React, { useState, useEffect, useCallback } from 'react';
import {Link} from 'react-router-dom'
import './SignIn.css';

function SignIn(props) {
    let { authenticated, authenticatedSet } = props;

    const [signinEmail, signinEmailSet] = useState("");
    const [signinPassword, signinPasswordSet] = useState("");

    const [signinEmailError, signinEmailErrorSet] = useState("");
    const [signinPasswordError, signinPasswordErrorSet] = useState("");

    const [signupEmail, signupEmailSet] = useState("");
    const [username, usernameSet] = useState("");
    const [usernameTimer, setUsernameTimer] = useState("");
    const [signupPassword, signupPasswordSet] = useState("");

    const [signupEmailError, signupEmailErrorSet] = useState("");
    const [usernameError, usernameErrorSet] = useState("");
    const [signupPasswordError, signupPasswordErrorSet] = useState("");

    const [signinError, signinErrorSet] = useState("");
    const [signupError, signupErrorSet] = useState("");

    const keyTimeout = 510;

    const onSignupSubmit = async e => {
        e.preventDefault();

        let data = {
            email: signupEmail,
            username,
            password: signupPassword,
        }

        let res = await fetch(`/auth/add-user`, {
            method: 'POST',
            body: JSON.stringify(data),
            headers: {
                'Content-Type': 'application/json'
            }
        });

        let json = await res.json();
        checkJson(json);
    }

    const onSigninSubmit = async e => {
        e.preventDefault();

        let data = {
            email: signinEmail,
            username: "",
            password: signinPassword,
        }

        let res = await fetch(`/auth/verify-user`, {
            method: 'POST',
            body: JSON.stringify(data),
            headers: {
                'Content-Type': 'application/json'
            }
        });

        let json = await res.json();
        checkJson(json);
    }

    const onInputChange = e => {
        switch (e.target.name) {
            case "signinEmail": signinEmailSet(e.target.value); break;
            case "signinPassword": signinPasswordSet(e.target.value); break;

            case "signupEmail": signupEmailSet(e.target.value); break;
            case "username": usernameSet(e.target.value); break;
            case "signupPassword": signupPasswordSet(e.target.value); break;
            default: break;
        }
    }

    const updateErrors = (json) => {
        switch (json.context) {
            case "signinEmail": signinEmailErrorSet(json.data); break;
            case "signinPassword": signinPasswordErrorSet(json.data); break;
            case "signupEmail": signupEmailErrorSet(json.data); break;
            case "username": usernameErrorSet(json.data); break;
            case "signupPassword": signupPasswordErrorSet(json.data); break;
            case "signin": signinErrorSet(json.data); break;
            case "signup": signupErrorSet(json.data); break;
            default: break;
        }
    }

    const checkJson = useCallback((json) => {
        signinEmailErrorSet("");
        signinPasswordErrorSet("");
        signupEmailErrorSet("");
        usernameErrorSet("");
        signupPasswordErrorSet("");
        signinErrorSet("");

        if (!json) return;
        if (json.type === "error") {
            updateErrors(json);
        } else if (json.type === "success") {
            if (json.context === "username_check") return;

            localStorage.setItem('authapp', json.data);
            authenticatedSet(true);
            signupEmailSet("");
            usernameSet("");
            signupPasswordSet("");
            signinEmailSet("");
            signinPasswordSet("");
        }
    }, [authenticatedSet]);

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
            checkJson(json);
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

        if (window.gapi && !authenticated) {
            addBtn();
        }
    }, [authenticated, authenticatedSet, checkJson]);

    useEffect(() => {
        let mounted = true;

        const onUsernameInputChange = async () => {
            let data = {
                email: "",
                username,
                password: "",
            }

            let res = await fetch(`/auth/check-username`, {
                method: 'POST',
                body: JSON.stringify(data),
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            let json = await res.json();
            mounted && checkJson(json);
        }

        clearTimeout(usernameTimer);
        setUsernameTimer(setTimeout(onUsernameInputChange, keyTimeout));

        return () => {
            clearTimeout(usernameTimer);
            mounted = false;
        };
    // adding usernameTimer locks up the UI
    // eslint-disable-next-line
    }, [username, authenticated, checkJson]);

    return (
        <main id="auth">
            {!props.authenticated &&
                <section id="signin">
                    <h1>Sign In</h1>
                    <form onSubmit={onSigninSubmit}>
                        <input name="signinEmail" placeholder="Email" onChange={onInputChange} value={signinEmail} type="email" />
                        <p className="error">{signinEmailError}</p>
                        <input name="signinPassword" placeholder="Password" onChange={onInputChange} value={signinPassword} type="password" />
                        <Link to="/forgot-password">Forget password?</Link>
                        <p className="error">{signinPasswordError}</p>
                        <input type="submit" value="Submit" />
                        <p className="error">{signinError}</p>
                    </form>
                    <div id="gs2"></div>
                </section>}

            {!props.authenticated &&
                <section id="signup">
                    <h1>Sign Up</h1>
                    <form onSubmit={onSignupSubmit}>
                        <input name="signupEmail" placeholder="Email" onChange={onInputChange} value={signupEmail} type="email" />
                        <p className="error">{signupEmailError}</p>
                        <input name="username" placeholder="Username" onChange={onInputChange} value={username} />
                        <p className="error">{usernameError}</p>
                        <input name="signupPassword" placeholder="Password" onChange={onInputChange} value={signupPassword} type="password" />
                        <p className="error">{signupPasswordError}</p>
                        <input type="submit" value="Submit" />
                        <p className="error">{signupError}</p>
                    </form>
                    <div id="gs2"></div>
                </section>}

            {props.authenticated &&
                <section id="success-content">
                    <h1>Success!</h1>
                </section>}
        </main>
    )
}

export default SignIn;