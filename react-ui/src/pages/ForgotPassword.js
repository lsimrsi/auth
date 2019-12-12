import React, { useState } from 'react';
// import './ForgotPassword.css';

function ForgotPassword() {
    const [email, emailSet] = useState("");
    const [emailError, emailErrorSet] = useState("");
    const [emailSent, emailSentSet] = useState(false);
    const [successMessage, successMessageSet] = useState("Email Sent.");

    const onSubmit = async e => {
        e.preventDefault();

        let data = {
            email,
            username: "",
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
        emailSet(e.target.value)
    }

    const checkErrors = (json) => {
        emailErrorSet("");

        if (!json) return;
        if (json.type !== "error") return;

        emailErrorSet(json.data);
    }

    return (
        <main id="forgot-password">
            {!emailSent && <section id="signup">
                <h1>Forgot Password</h1>
                <form onSubmit={onSubmit}>
                    <input name="email" placeholder="email" onChange={onInputChange} value={email} type="email" />
                    <p className="error">{emailError}</p>
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