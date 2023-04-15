# Backend Todo
- impl actix_web::error:ResponseError for sqlx::Error  ✔
    - proper DB error handling, not just ctrl+v "Database quirked up :(" 🤡 ✔
- add FromRequest for WGIdentity (requires WG) ✔
- DELETE cost ✔

- Register User + create WG
- Send invite link to create user for existing wg
- Change Password Route
- **POST** `/my_wg/costs` with multipart/form attatchment endpoint
- Rethink mulipart/formdata implementation, that shits crusty afff

- Scaling Options maybe, worker threads or smth
