# Prover server

The prover server allows a seamless interface to request proofs from a server. It also allows you to encrypt your requests to the server by making use of the NSM attestation API.

## Build

build `rapidsnark` following their `README` first
then use `Makefile` to build the prover backend

## Running the server

The following options can be used to run the server:

```sh
Options:
  -s, --server-address <SERVER_ADDRESS>
          Web server bind address (e.g., 0.0.0.0:3001) [default: 0.0.0.0:3001]
  -d, --database-url <DATABASE_URL>
          PostgreSQL database connection URL [default: postgres://postgres:mysecretpassword@localhost:5433/db]
  -c, --circuit-folder <CIRCUIT_FOLDER>
          Circuit folder path [default: ../circuits]
  -k, --zkey-folder <ZKEY_FOLDER>
          ZKey folder path [default: ./zkeys]
  -z, --circuit-zkey-map <CIRCUIT_ZKEY_MAP>
          Witness calc circuit to zkey mapper
  -r, --rapidsnark-path <RAPIDSNARK_PATH>
          Rapidsnark path [default: ./rapidsnark]
  -h, --help
          Print help
```

```sh
# Install socat
sudo dnf install socat -y
socat tcp-listen:8888,fork,reuseaddr vsock-connect:<ENCLAVE_ID>:8888 # for the rpc server
```

# API

This API follows the JSON-RPC 2.0 protocol and operates under the `iawia` namespace.

### 1. `hello`

**Description:**
The first part of an ECDH handshake. The user sends their public key along with a UUID that is linked to their session ID when scanning the QR code.

**Method Name:** `iawia_hello`

**Request Parameters:**

- `user_pubkey` (Vec<u8>): The public key of the user.
- `uuid` (String): A unique identifier for the request.

**Response:**
Returns a `ResponsePayload` containing `HelloResponse` which is the request UUID and the attestation response. Please verify the response before making the second request.

---

### 2. `submit_request`

**Description:**
Submits an encrypted request along with authentication data. The encryption scheme used is AES-GCM.

**Method Name:** `iawia_submit_request`

**Request Parameters:**

- `uuid` (String): A unique identifier for the request.
- `nonce` (Vec<u8>): A cryptographic nonce.
- `cipher_text` (Vec<u8>): The encrypted request payload.
- `auth_tag` (Vec<u8>): The authentication tag for integrity verification.

**Response:**
Returns a `ResponsePayload` containing the UUID.

**Request Parameters:**

- `user_data` (Option<Vec<u8>>): Optional user-related data.
- `nonce` (Option<Vec<u8>>): Optional cryptographic nonce.
- `public_key` (Option<Vec<u8>>): Optional public key.

**Response:**
Returns a `ResponsePayload` containing attestation data as a vector of bytes.

## Usage

Clients can send JSON-RPC requests to the IAWIA API endpoint, following the standard JSON-RPC 2.0 format:

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "method": "iawia_hello",
  "params": {
    "user_pubkey": "...",
    "uuid": "550e8400-e29b-41d4-a716-446655440000"
  },
  "id": 1
}
```

**Example Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "uuid": "550e8400-e29b-41d4-a716-446655440000",
    "attestation": [...]
  }
}
```
