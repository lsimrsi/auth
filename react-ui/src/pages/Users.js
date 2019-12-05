import React, { useState, useEffect } from 'react';
import { useHistory } from "react-router-dom";

function Users() {
    let history = useHistory();
    const [users, setUsers] = useState([]);

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

    useEffect(() => {
        getUsers();
    }, []);

    return (
        <main>
            {users.map((item) => {
                return <p>{item}</p>
            })}
        </main>
    )
}

export default Users;