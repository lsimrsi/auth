import React, { useState, useEffect } from 'react';
import jwt from 'jsonwebtoken';
// import './ResetPassword.css';

function ResetPassword() {
    const [newPass1, newPass1Set] = useState("");
    const [newPass1Error, newPass1ErrorSet] = useState("");
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
            password: newPass1,
        };

        let res = await fetch(`/auth-db/reset-password`, {
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
        }
    }

    const onInputChange = e => {
        newPass1Set(e.target.value)
    }

    const checkErrors = (json) => {
        newPass1ErrorSet("");

        if (!json) return;
        if (json.type !== "error") return;

        newPass1ErrorSet(json.data);
    }

    return (
        <main>
            {!reset && !tokenExpired && <section>
                <h1>Reset Password</h1>
                <form onSubmit={onSubmit}>
                    <input name="newPass1" placeholder="New password" onChange={onInputChange} value={newPass1} />
                    <p className="error">{newPass1Error}</p>
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