import React, { useState, useEffect } from 'react';
import { useHistory } from "react-router-dom";
import './Users.css';

function Users() {
    let history = useHistory();
    const [users, setUsers] = useState([]);

    useEffect(() => {
        const getUsers = async () => {
            let token = localStorage.getItem("authapp");
            let res = await fetch(`/auth-db/get-users`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${token}`
                },
            });
            let json = await res.json();

            if (json && json.type === "success") {
                setUsers(json.data);
            } else {
                history.push("/sign-up");
            }
        }
        getUsers();
    }, [history]);

    return (
        <main id="users">
            <h1>Users</h1>
            {users.map((item) => {
                return <p><span>{item}</span></p>
            })}
        </main>
    )
}

export default Users;