# API Documentation

## Login Endpoint

### Overview

The login API endpoint allows users to authenticate and obtain a session token for subsequent authorized requests. Users are required to provide their username and password in the request payload. Upon successful authentication, a session token is generated and stored as an HTTP-only cookie for future use.

### Endpoint

- **URL:** `/login`
- **Method:** `POST`

### Request Payload

The login request payload must be a JSON object with the following fields:

- **username** (string): The username of the user attempting to log in.
- **password** (string): The password associated with the provided username.

Example:

```json
{
  "username": "example_user",
  "password": "secure_password"
}
```

### Response

#### Successful Authentication

Upon successful authentication, the API returns a JSON object indicating success. Additionally, a session token is stored as an HTTP-only cookie in the response.

```json
{
  "result": {
    "success": true
  }
}
```

#### Incorrect Password

If the provided password does not match the stored password hash for the given username, the API returns a JSON object indicating failure with a reason.

```json
{
  "result": {
    "success": false,
    "reason": "Incorrect password"
  }
}
```

### Implementation Details

#### Function Signature

```rust
pub async fn login_route(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>>;
```

#### Parameters

- **cookies** (Cookies): The HTTP cookies associated with the request.
- **payload** (Json<LoginPayload>): The JSON payload containing username and password.

#### Session Token Generation

- A session token is generated using a cryptographically secure random alphanumeric string.
- The length of the session token is configurable and set to 30 characters by default.

#### Cookie Creation

- The session token is stored as an HTTP-only cookie with the name `sessionToken`.
- The cookie is set to expire after 7 days from the current date.
- The cookie is restricted to the path `/` to ensure it is sent with every subsequent request to the domain.

#### Examples

##### cURL

```bash
curl -X POST \
  http://example.com/login \
  -H 'Content-Type: application/json' \
  -d '{
    "username": "example_user",
    "password": "secure_password"
  }'
```

##### Response

```json
{
  "result": {
    "success": true
  }
}
```

## Account Registration Endpoint

### Overview

The account registration API endpoint allows users to create a new account by providing a username and password. The endpoint enforces specific criteria for username and password to ensure security and compliance with standards. Upon successful registration, the API returns a success response along with the UUID assigned to the new user.

### Endpoint

- **URL:** `/register`
- **Method:** `POST`

### Request Payload

The registration request payload must be a JSON object with the following fields:

- **username** (string): The desired username for the new account.
- **password** (string): The password for the new account.

Example:

```json
{
  "username": "new_user",
  "password": "secure_password123"
}
```

### Response

#### Successful Registration

Upon successful registration, the API returns a JSON object indicating success. The response also includes the UUID assigned to the newly registered user.

```json
{
  "result": {
    "success": true,
    "uuid": "7e6f856a-73d5-4d5a-a23d-9b9ca98332b4"
  }
}
```

#### Username Validation Failure

If the provided username does not meet the required criteria, the API returns a JSON object indicating failure with a reason.

```json
{
  "result": {
    "success": false,
    "reason": "Username must be between 6 and 16 characters and contain only alphanumeric characters"
  }
}
```

#### Password Validation Failure

If the provided password does not meet the required criteria, the API returns a JSON object indicating failure with a reason.

```json
{
  "result": {
    "success": false,
    "reason": "Password must be at least 8 characters long and contain at least one uppercase letter, one lowercase letter, one digit, and one special character"
  }
}
```

#### Username Already Exists

If the provided username already exists in the system, the API returns a JSON object indicating failure with a reason.

```json
{
  "result": {
    "success": false,
    "reason": "Username already exists"
  }
}
```

### Implementation Details

#### Function Signature

```rust
pub async fn register_account(payload: Json<RegistrationPayload>) -> Result<Json<Value>>;
```

#### Parameters

- **payload** (Json<RegistrationPayload>): The JSON payload containing the username and password for account registration.

#### Username and Password Verification

- Username and password are validated against specific criteria before proceeding with registration.
- Username must be between 6 and 16 characters and contain only alphanumeric characters.
- Password must be at least 8 characters long and contain at least one uppercase letter, one lowercase letter, one digit, and one special character.

#### Username Availability Check

- The API checks if the provided username already exists in the system before proceeding with registration.

#### UUID Generation

- A UUID is generated for the new user during registration.

#### User Upload

- The new user information, including username, hashed password, salt, and UUID, is uploaded to the database.

#### Examples

##### cURL

```bash
curl -X POST \
  http://example.com/register \
  -H 'Content-Type: application/json' \
  -d '{
    "username": "new_user",
    "password": "secure_password123"
  }'
```

##### Response

```json
{
  "result": {
    "success": true,
    "uuid": "7e6f856a-73d5-4d5a-a23d-9b9ca98332b4"
  }
}
```

## Retrieving User Files

### Overview

The endpoint for retrieving all files associated with a user allows clients to obtain a list of file information for a specific user. The response includes a JSON array containing file details, with each element representing a file's ID.

### Endpoint

- **URL:** `/files`
- **Method:** `GET`

### Request Headers

- **Authorization:** Bearer token for user authentication

### Response

#### Successful Retrieval

Upon successful retrieval of user files, the API returns a JSON array containing file details.

```json
{
  "files": [
    {
      "file_id": "2abf6d7c-5571-4e07-9c3d-83b7426cc6a0"
    },
    {
      "file_id": "7e1f9cda-bd7e-4a82-8bc1-176f168a22af"
    },
    ...
  ]
}
```

#### No Files Found

If the user does not have any associated files, the API returns an empty JSON array.

```json
{
  "files": []
}
```

#### Error Handling

- If there is an error during the retrieval process, the API returns an appropriate HTTP status

# File Upload Endpoint

## Overview

The file upload API endpoint enables users to upload files to the server. This endpoint expects a multipart/form-data request with the file to be uploaded. Upon successful upload, the API returns a JSON response indicating success.

### Endpoint

- **URL:** `/upload`
- **Method:** `POST`

### Request

#### Headers

- **Authorization:** Bearer token for user authentication
- **Content-Type:** multipart/form-data

#### Form Data

- **file:** The file to be uploaded

### Response

#### Successful Upload

Upon successful file upload, the API returns a JSON object indicating success.

```json
{
  "status": "success"
}
```

#### Error Handling

If there is an error during the upload process, the API returns an appropriate HTTP status code along with an error message.

### Implementation Details

#### Function Signature

```rust
pub async fn upload(
    ctx: Ctx,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Value>>;
```

#### Parameters

- **ctx** (Ctx): The context containing user information obtained from the authentication token.
- **headers** (axum::http::HeaderMap): The headers of the multipart/form-data request.
- **multipart** (Multipart): The multipart data containing the file to be uploaded.

#### User Authentication

The endpoint requires a valid user authentication token to authorize the file upload.

#### Multipart/Form-Data Handling

The endpoint uses the `multipart` crate to handle the multipart/form-data request.

#### File Retrieval

The uploaded file is retrieved from the multipart data.

#### File Upload

The retrieved file data is passed to the `file_upload::upload` function, which handles the actual storage of the file on the server.

### Examples

#### cURL

```bash
curl -X POST \
  http://example.com/api/upload \
  --cookie 'AUTH_TOKEN: Bearer <TOKEN>' \
  -H 'Content-Type: multipart/form-data' \
  -F 'file=@/path/to/file.txt'
```

#### Response

```json
{
  "status": "success"
}
```