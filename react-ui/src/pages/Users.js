import React, { useState, useEffect } from 'react';

function Users() {
    const [users, setUsers] = useState([]);

    const getUsers = async () => {
      let res = await fetch(`/auth-db/get-users`, {
        method: 'GET',
      });
      let json = await res.json();
  
      if (json && json.type === "success") {
        setUsers(json.data);
      }
    }

    useEffect(() => {
        getUsers();
    }, []);

    return(
        <main>
        {users.map((item) => {
          return <p>{item}</p>
        })}
        </main>
    )
}

export default Users;