# API Documentation

## Table of Contents
- [Root](#root)
- [Upload](#upload)
- [Return Build Status](#return-build-status)
- [Get User Info](#get-user-info)
- [Get User Files](#get-user-files)

---

## Root
### `GET /`
- **Description:** Basic handler that responds with a static string.
- **Parameters:** None
- **Responses:**
  - `200 OK`: Successful response with the static string.

---

## Upload
### `POST /upload`
- **Description:** Endpoint to upload a file.
- **Parameters:**
  - Multipart form data containing the file to upload.
- **Responses:**
  - `200 OK`: Successful response with the UUID of the uploaded file.

---

## Return Build Status
### `GET /status/:fileid`
- **Description:** Endpoint to get the build status of a specific file.
- **Parameters:**
  - `fileid` (Path Parameter): UUID of the file to retrieve the build status for.
- **Responses:**
  - `200 OK`: Successful response with the build status.
  - `404 Not Found`: File not found.

---

## Get User Info
### `GET /profile`
- **Description:** Endpoint to retrieve user information.
- **Parameters:**
  - Authorization header containing the user token.
- **Responses:**
  - `200 OK`: Successful response with user information.
  - `401 Unauthorized`: User token is invalid.

---

## Get User Files
### `GET /files`
- **Description:** Endpoint to retrieve a list of user files.
- **Parameters:**
  - Authorization header containing the user token.
- **Responses:**
  - `200 OK`: Successful response with an array of file UUIDs.
  - `500 Internal Server Error`: An error occurred while processing the request.


  ## Testing
  gcc program.c -o program.o && http POST http://127.0.0.1:3000/run