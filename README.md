# Mega Uploader Auth 🚀

`mega-uploader-auth` is an authentication service designed to facilitate the login flow for CLI (Command Line Interface)
applications using **AWS Cognito** and **AWS STS** to provide temporary AWS credentials.

The service uses **Redis** to manage temporary authentication states and sessions, ensuring a secure and efficient flow.

## ✨ Features

- 🔐 **AWS Cognito Integration**: Handles the OAuth2 flow (Authorization Code Grant).
- 🔑 **AWS STS (Security Token Service)**: Generates temporary credentials (`AccessKeyId`, `SecretAccessKey`,
  `SessionToken`) for clients.
- 🔄 **Session Renewal**: Supports the use of `refresh_token` to obtain new credentials without re-authenticating the
  user.
- 🚀 **High Performance**: Built with **Rust** and **Actix Web**.
- 📦 **Redis Persistence**: Session and state management with automatic TTL.

## 🛠️ Technologies

- **Language:** [Rust](https://www.rust-lang.org/) (2024 Edition)
- **Web Framework:** [Actix Web 4](https://actix.rs/)
- **Database:** [Redis](https://redis.io/) (with `bb8` connection pool)
- **AWS SDK:** `aws-sdk-sts` and `aws-config`
- **Tokens:** `jsonwebtoken` for Cognito ID Token validation.

## ⚙️ Configuration

The service can be configured via environment variables or command-line arguments (using `clap`).

### Required Environment Variables

| Variable               | Description                           | Example                                   |
|------------------------|---------------------------------------|-------------------------------------------|
| `REDIS_URL`            | Redis connection URL                  | `redis://127.0.0.1:6379`                  |
| `SERVER_ADDR`          | Server address and port               | `127.0.0.1:8080`                          |
| `COGNITO_DOMAIN`       | AWS Cognito domain                    | `https://auth.example.com`                |
| `COGNITO_CLIENT_ID`    | Cognito App Client ID                 | `6h...`                                   |
| `COGNITO_REDIRECT_URI` | Redirect URI (callback)               | `http://localhost:8080/auth/cli/callback` |
| `COGNITO_USER_POOL_ID` | Cognito User Pool ID                  | `us-east-1_XXXXX`                         |
| `COGNITO_REGION`       | AWS Cognito Region                    | `us-east-1`                               |
| `STS_ROLE_ARN`         | IAM Role ARN to assume                | `arn:aws:iam::123456:role/CliRole`        |
| `STS_EXTERNAL_ID`      | (Optional) External ID for AssumeRole | `my-external-id`                          |

## 🚀 Installation and Execution

### Prerequisites

- Rust (latest stable version)
- Redis running locally or in a container.

### Steps

1. **Clone the repository:**
   ```bash
   git clone <repo-url>
   cd mega-uploader-auth
   ```

2. **Configure the environment:**
   You can create a `.env` file or export the variables mentioned above.

3. **Build and run:**
   ```bash
   cargo run --release
   ```

The server will be available by default at `http://127.0.0.1:8080`.

## 📡 API Endpoints

### Information

- **`GET /`**: Returns an informative page with the service status and available endpoints.

### CLI Authentication

1. **`POST /auth/cli/start`**: Initiates the process. The client sends device information and receives a Cognito
   authorization URL.
2. **`GET /auth/cli/callback`**: Endpoint where Cognito redirects the user after successful login. Processes the code
   and saves the session in Redis.
3. **`GET /auth/cli/status?state=<uuid>`**: The CLI client polls here to verify if the user completed the login and to
   obtain AWS STS credentials.
4. **`POST /auth/cli/renew`**: Allows the client to renew their AWS credentials using the stored `refresh_token`.

---

Developed by **DPAAS**.
