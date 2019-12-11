import React, { useState, useEffect } from 'react';
import jwt from 'jsonwebtoken';
// import './ResetPassword.css';

function ResetPassword(props) {
    const [newPass, newPassSet] = useState("");
    const [newPassError, newPassErrorSet] = useState("");
    const [reset, resetSet] = useState(false);
    const [username, usernameSet] = useState("");
    const [token, tokenSet] = useState("");
    const [tokenExpired, tokenExpiredSet] = useState(true);

    useEffect(() => {
        let params = new URLSearchParams(window.location.search);
        let token = params.get("token");
        if (!token) return;

        let res = jwt.decode(token);
        if (res.exp * 1000 > Date.now()) {
            tokenSet(token);
            tokenExpiredSet(false);
            usernameSet(res.sub);
        }
    }, []);

    const onSubmit = async e => {
        e.preventDefault();

        let data = {
            email: "",
            username,
            password: newPass,
        };

        let res = await fetch(`/auth/reset-password`, {
            method: 'POST',
            body: JSON.stringify(data),
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            }
        });

        let json = await res.json();

        checkErrors(json);
        if (json && json.type === "success") {
            localStorage.setItem('authapp', json.data);
            resetSet(true);
            props.authenticatedSet(true);
        }
    }

    const onInputChange = e => {
        newPassSet(e.target.value)
    }

    const checkErrors = (json) => {
        newPassErrorSet("");

        if (!json) return;
        if (json.type !== "error") return;

        newPassErrorSet(json.data);
    }

    return (
        <main>
            {!reset && !tokenExpired && <section>
                <h1>Reset Password</h1>
                <form onSubmit={onSubmit}>
                    <input name="newPass" placeholder="New password" onChange={onInputChange} value={newPass} />
                    <p className="error">{newPassError}</p>
                    <input type="submit" value="Submit" />
                </form>
            </section>}

            {reset && !tokenExpired && <section>
                <h1>Success!</h1>
                <p>You are now logged in.</p>
            </section>}

            {tokenExpired && <section>
                <h1>Password reset expired.</h1>
                <p>Please initiate another request.</p>
            </section>}
        </main>
    )
}

export default ResetPassword;