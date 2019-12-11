import React, { useState } from 'react';
// import './ForgotPassword.css';

function ForgotPassword() {
    const [username, usernameSet] = useState("");
    const [usernameError, usernameErrorSet] = useState("");
    const [emailSent, emailSentSet] = useState(false);
    const [successMessage, successMessageSet] = useState("Email Sent.");

    const onSubmit = async e => {
        e.preventDefault();

        let data = {
            email: "",
            username,
            password: "",
        };

        let res = await fetch(`/auth/forgot-password`, {
            method: 'POST',
            body: JSON.stringify(data),
            headers: {
                'Content-Type': 'application/json'
            }
        });

        let json = await res.json();

        checkErrors(json);
        if (json && json.type === "success") {
            emailSentSet(true);
            successMessageSet(json.data);
        }
    }

    const onInputChange = e => {
        usernameSet(e.target.value)
    }

    const checkErrors = (json) => {
        usernameErrorSet("");

        if (!json) return;
        if (json.type !== "error") return;

        usernameErrorSet(json.data);
    }

    return (
        <main id="forgot-password">
            {!emailSent && <section id="signup">
                <h1>Forgot Password</h1>
                <form onSubmit={onSubmit}>
                    <input name="username" placeholder="Username" onChange={onInputChange} value={username} />
                    <p className="error">{usernameError}</p>
                    <input type="submit" value="Submit" />
                </form>
            </section>}

            {emailSent && <section id="success-content">
                <h1>{successMessage}</h1>
                <p>Please check your spam folder.</p>
            </section>}
        </main>
    )
}

export default ForgotPassword;