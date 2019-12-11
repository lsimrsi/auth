// use crate::auth::User;
// use crate::auth::AuthError;

// pub fn insert(user: User) -> Result<String, AuthError> {

//     match conn.execute(
//         "INSERT INTO users (email, username, password) VALUES ($1, $2, $3)",
//         &[&user.email, &user.username, &hashed_password],
//     ) {
//         Ok(_) => auth.create_token(user.username.clone()),
//         Err(err) => {
//             if let Some(dberr) = err.as_db() {
//                 println!("some dberr");
//                 // unique violation
//                 if !(dberr.code.code() == "23505") {
//                     println!("code doesn't equal 23505");
//                     return Err(AuthError::internal_error(&err.to_string()));
//                 }
//                 if let Some(constraint) = &dberr.constraint {
//                     match constraint.as_ref() {
//                         "users_email_key" => {
//                             return Err(AuthError::new(
//                                 "signupEmail",
//                                 "This email has already been registered.",
//                                 "",
//                                 500,
//                             ))
//                         }
//                         "users_username_key" => {
//                             return Err(AuthError::new(
//                                 "username",
//                                 "This username has already been taken.",
//                                 "",
//                                 500,
//                             ))
//                         }
//                         _ => return Err(AuthError::internal_error(&err.to_string())),
//                     }
//                 }
//             }
//             Err(AuthError::internal_error(&err.to_string()))
//         }
//     }
// }