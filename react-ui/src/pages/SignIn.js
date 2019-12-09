import React, { useState, useEffect } from 'react';
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


    const onSignupSubmit = async e => {
        e.preventDefault();

        let data = {
            email: signupEmail,
            username,
            password: signupPassword,
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
            authenticatedSet(true);
        }
    }

    const onSigninSubmit = async e => {
        e.preventDefault();

        let data = {
            email: signinEmail,
            username: "",
            password: signinPassword,
        }

        let res = await fetch(`/auth-db/verify-user`, {
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
            authenticatedSet(true);
        }
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

    const checkErrors = (json) => {
        signinEmailErrorSet("");
        signinPasswordErrorSet("");
        signupEmailErrorSet("");
        usernameErrorSet("");
        signupPasswordErrorSet("");
        signinErrorSet("");

        if (!json) return;
        if (json.type !== "error") return;

        switch (json.context) {
            case "signinEmail": signinEmailErrorSet(json.data); break;
            case "signinPassword": signinPasswordErrorSet(json.data); break;
            case "signupEmail": signupEmailErrorSet(json.data); break;
            case "username": usernameErrorSet(json.data); break;
            case "signupPassword": signupPasswordErrorSet(json.data); break;
            case "signin": signinErrorSet(json.data); break;
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

            if (json && json.type === "success") {
                localStorage.setItem('authapp', json.data);
                authenticatedSet(true);
            }
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

    }, [authenticated, authenticatedSet]);

    useEffect(() => {
        let mounted = true;

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

            if (!mounted) return;
            checkErrors(json);
        }

        clearTimeout(usernameTimer);
        setUsernameTimer(setTimeout(onUsernameInputChange, 500));

        return () => {
            clearTimeout(usernameTimer);
            mounted = false;
        };
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [username, authenticated]);

    return (
        <main id="auth">
            {!props.authenticated &&
                <section id="signin">
                    <h1>Sign In</h1>
                    <form onSubmit={onSigninSubmit}>
                        <input name="signinEmail" placeholder="Email" onChange={onInputChange} value={signinEmail} />
                        <p className="error">{signinEmailError}</p>
                        <input name="signinPassword" placeholder="Password" onChange={onInputChange} value={signinPassword} type="signinPassword" />
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
                        <input name="signupEmail" placeholder="Email" onChange={onInputChange} value={signupEmail} />
                        <p className="error">{signupEmailError}</p>
                        <input name="username" placeholder="Username" onChange={onInputChange} value={username} />
                        <p className="error">{usernameError}</p>
                        <input name="signupPassword" placeholder="Password" onChange={onInputChange} value={signupPassword} type="password" />
                        <p className="error">{signupPasswordError}</p>
                        <input type="submit" value="Submit" />
                    </form>
                    <div id="gs2"></div>
                </section>}

            {props.authenticated &&
                <div id="success-content">
                    <h1>Success!</h1>
                </div>}
        </main>
    )
}

export default SignIn;