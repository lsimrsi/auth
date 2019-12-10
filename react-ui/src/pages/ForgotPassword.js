import React, { useState, useEffect } from 'react';
import './SignIn.css';

function SignIn(props) {
    const [email, emailSet] = useState("");
    const [emailError, emailErrorSet] = useState("");
    const [emailSent, emailSentSet] = useState(false);

    const onSubmit = async e => {
        e.preventDefault();

        let data = {
            email,
            username: "",
            password: "",
        };

        let res = await fetch(`/auth-db/forgot-password`, {
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
                    <input name="email" placeholder="Email" onChange={onInputChange} value={email} />
                    <p className="error">{emailError}</p>
                    <input type="submit" value="Submit" />
                </form>
            </section>}

            {emailSent && <section id="success-content">
                <h1>Email Sent!</h1>
                <p>Please check your spam folder.</p>
            </section>}
        </main>
    )
}

export default SignIn;