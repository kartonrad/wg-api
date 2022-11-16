# API Documentation
For the wg-api in general in particular

# Notes
- API uses permissive CORS settings (can be accessed from any origin)
- API accepts authentication via cookies, and Authorization header

# Routes

## Root Scope:
- GET `/` 
    - Greeting
- `/auth` 
    - See scope [auth](#auth-scope)
- `/public/...` 
    - Serve assets in `./src/public`, which are embedded into the executable
- `/api` 
    - See scope [API](#api-scope)
- `/uploads/...` 
    - Serve [Uploads](#upload-object) in `./uploads`, requires Auth for wg-scoped uploads

## Auth Scope:
Every route starts with `/auth`

- POST `/login` 
    - json param `username`
    - json param `password`
- GET `/login_unsafe`- Unsafe Login via Cookies 
    - **(⚠ password may end up in browser history ⚠)**
    - query param `username`
    - query param `password`
- GET `/hash_password_unsafely_use_only_for_tests` - hashes the provided password 
    - **(⚠ password may end up in browser history ⚠)**
    - query param `password`
- GET `/priviledged` - Example Route
    - *AUTH required*
- GET `/classist` - Example Route
    - *AUTH optional*

## API Scope:
Every route starts with `/api`

### Private User Endpoints
- GET  `/me` 
    - *AUTH required*
    - Get your own user information.
    - Returns: [Intimate User](#intimate-user-object)
- **PUT**  `/me` 
    - Edit your user information
    - Makes individual Database call for each field.
    - returning null for failed database insert.
    - fails only when there exist fields that are empty but provided, unknown, malformed, larger than 40mb - or when writing a file fails
    - Multipart-Form Data Body:
        - (optional) `name` - String
        - (optional) `username` - String
        - (optional) `bio` - String
        - (optional) `profile_pic` - File
    - Returns: changed fields ( same as request fields, each containing: null for omitted/failure, updated value for success )

### Public WG Endpoints
- GET  `/wg/{url}`
    - {url} must be alphanumeric-_ wg-identifier 
    - Returns: WG
- GET  `/wg/{url}/users`
    - {url} must be alphanumeric-_ wg-identifier 
    - Returns Array of [Users](#regular-user-object)

### Private WG Endpoints (*AUTH required* for all, User must join a WG)
- GET  `/my_wg` 
    - Get your own WG
    - Returns: [WG](#wg-object)
- **PUT**  `/my_wg` 
    - Edit your own WG
    - Makes individual Database call for each field.
    - returning null for failed database insert.
    - fails only when there exist fields that are empty but provided, unknown, malformed, larger than 40mb - or when writing a file fails
    - Multipart-Form Data Body:
        - (optional) `name` - String
        - (optional) `url` - String
        - (optional) `description` - String
        - (optional) `profile_pic` - File
        - (optional) `header_pic` - File
    - Returns: changed fields ( above fields, null for omitted/failure, updated value for success )
- GET  `/my_wg/users`
    - Returns Array of [Users](#regular-user-object)
- GET  `/my_wg/costs`
    - optional query param `balance` - Integer (id of Balance)
    - Return Array of [Costs](#cost-object)
- **POST** `/my_wg/costs`
    - Creates a new Cost, and all it's Shares according to the `debtors` field
    - `name`: String
    - `amount`: Number (Decimal Value, preferrably only two digits behind comma)
    - `added_on`: DateTime in iso8601 format (js: `new Date().toISOString()`, example: `2022-11-16T00:01:42.763Z`)
    - `debtors`: Array of Touples (Arrays again) with two values each
        - First Value: Integer - Debtor Id (id of [User](#intimate-user-object), wrong id's are silently ignored)
        - Second Value: Boolean - Is the debt paid already? (will be force-set to true, if this debtor id is you (poster/creditor) )
        - -Example: `[[1, false], [2, true], [3, false], [69, true]]`
- GET  `/my_wg/costs/{id}/shares`
    - url param `{id}` must be id of a [Cost](#cost-object) 
    - id of accessed cost must be in you wg or Array will be empty.
    - Returns Array of [Shares](#cost-share-object)
- **PUT**  `/my_wg/costs/{id}/receit`
    - You must be the one to have posted this Cost (creditor)
    - Multipart-Form Data Body:
        - `receit` - File
- GET  `/my_wg/costs/stats`
    - optional query param `balance` - Integer (id of Balance)
- GET  `/my_wg/costs/balance`
    - Returns Array of Balances
- **POST** `/my_wg/costs/balance`
    - Creates a balance and assigns all unbalanced Costs to it
    - You will be set as issuer, with the current timestamp
- GET  `/my_wg/costs/over_time/{interval}`
    - Return a bunch of cost statistics per time interval

# API Return Types
## Upload Object
Either all fields will be present, or all fields will be `null`.

URL for this upload is `<api-url>/uploads/<id>.<extension>`

It is somewhatttt guaranteed that this upload will be available in the wg it is referenced in. Uploads can't be moved and are only ever locked to the wg they are created for...
But this is not *strictly* enforced... 
Usually, only Receits require authentication, since wgs/users are [publicly acessible](#public-wg-endpoints)

- `id`: Integer
- `extension`: String -> Length: max 5
- `original_filename`: String -> Length: max 200 
- `size_kb`: Integer

## Intimate User Object
"Intimate" Representation of the logged-in user.
The intimate representation is only shown to your yourself.
It maps exactly onto the `users` Table in the database, though the password hash is redacted.
- `id`: Integer
- `username`: String -> Length: max 40, alphanumeric-_ 
- `name`: String -> Length: max 200
- `bio`: String
- `password_hash`: String -> Always `"<Not Provided>"`
- `revoke_before`: DateTime in RFC3339 Format (example: `"2019-10-12T07:20:50.52Z"`)
- `profile_pic`: Integer (id of [Upload](#upload-object)) or null 
- `wg`: Integer (id of [WG](#wg-object)) or null

## (Regular) User Object
"Regular" Representation of a User inside a WG
Usually, the Client should know for which WG this User is being retrieved
- `id`: Integer
- `username`: String -> Length: max 40, alphanumeric-_
- `name`: String -> Length: max 200
- `bio`: String
- `profile_pic`: [Upload](#upload-object) or null

## WG Object
Maps exactly onto the `wgs` Table in the Database
- `id`: Integer
- `url`: String -> Length: max 40, alphanumeric-_
- `name`: String -> Length: max 200
- `description`: String
- `profile_pic`: [Upload](#upload-object) or null
- `header_pic`: [Upload](#upload-object) or null

## Cost Object
Maps exactly onto the `costs` Table in the Database, except for the last three fields, which are only aggregated in the Query to get this Object.
- `id`: Integer,
- `wg_id` : Integer (id of [WG](#wg-object))
- `name`: String
- `amount`: (Decimal) Number as **String**!!
- `creditor_id`: Integer (id of [User](#intimate-user-object))
- `added_on`: DateTime in RFC3339 Format (example: `"2019-10-12T07:20:50.52Z"`
- `equal_balances`: Integer (id of Balance) or null
- `receit`: [Upload](#upload-object) or null
- `my_share`: [Share](#cost-share-object) or null
- `nr_shares`: Integer (theoretically 64bit!)
- `nr_unpaid_shares`: Integer (theoretically 64bit!)

## Cost Share Object
Maps exactly onto the `cost_shares` Table in the Database.
If this Share Object is returned as part of a different Object:
Then either all fields will be present, or all fields will be `null`.
- `cost_id`: Integer (id of [Cost](#cost-object)), 
- `debtor_id`: Integer (id of [User](#intimate-user-object)),
- `paid`: Boolean
