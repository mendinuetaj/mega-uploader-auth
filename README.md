# Mega Uploader Auth 🚀

`mega-uploader-auth` es un servicio de autenticación diseñado para facilitar el flujo de inicio de sesión de
aplicaciones CLI (Command Line Interface) utilizando **AWS Cognito** y **AWS STS** para proporcionar credenciales
temporales de AWS.

El servicio utiliza **Redis** para gestionar estados de autenticación temporales y sesiones, asegurando un flujo seguro
y eficiente.

## ✨ Características

- 🔐 **Integración con AWS Cognito**: Maneja el flujo de OAuth2 (Authorization Code Grant).
- 🔑 **AWS STS (Security Token Service)**: Genera credenciales temporales (`AccessKeyId`, `SecretAccessKey`,
  `SessionToken`) para los clientes.
- 🔄 **Renovación de Sesión**: Soporta el uso de `refresh_token` para obtener nuevas credenciales sin re-autenticar al
  usuario.
- 🚀 **Alto Rendimiento**: Construido con **Rust** y **Actix Web**.
- 📦 **Persistencia en Redis**: Gestión de sesiones y estados con TTL automático.

## 🛠️ Tecnologías

- **Lenguaje:** [Rust](https://www.rust-lang.org/) (Edición 2024)
- **Web Framework:** [Actix Web 4](https://actix.rs/)
- **Base de Datos:** [Redis](https://redis.io/) (con pool de conexiones `bb8`)
- **AWS SDK:** `aws-sdk-sts` y `aws-config`
- **Tokens:** `jsonwebtoken` para validación de ID Tokens de Cognito.

## ⚙️ Configuración

El servicio se puede configurar mediante variables de entorno o argumentos de línea de comandos (usando `clap`).

### Variables de Entorno Necesarias

| Variable               | Descripción                            | Ejemplo                                   |
|------------------------|----------------------------------------|-------------------------------------------|
| `REDIS_URL`            | URL de conexión a Redis                | `redis://127.0.0.1:6379`                  |
| `SERVER_ADDR`          | Dirección y puerto del servidor        | `127.0.0.1:8080`                          |
| `COGNITO_DOMAIN`       | Dominio de AWS Cognito                 | `https://auth.example.com`                |
| `COGNITO_CLIENT_ID`    | Client ID de la App en Cognito         | `6h...`                                   |
| `COGNITO_REDIRECT_URI` | URI de redirección (callback)          | `http://localhost:8080/auth/cli/callback` |
| `COGNITO_USER_POOL_ID` | ID del User Pool de Cognito            | `us-east-1_XXXXX`                         |
| `COGNITO_REGION`       | Región de AWS de Cognito               | `us-east-1`                               |
| `STS_ROLE_ARN`         | ARN del Rol de IAM a asumir            | `arn:aws:iam::123456:role/CliRole`        |
| `STS_EXTERNAL_ID`      | (Opcional) External ID para AssumeRole | `mi-id-externo`                           |

## 🚀 Instalación y Ejecución

### Requisitos previos

- Rust (última versión estable)
- Redis corriendo localmente o en un contenedor.

### Pasos

1. **Clonar el repositorio:**
   ```bash
   git clone <repo-url>
   cd mega-uploader-auth
   ```

2. **Configurar el entorno:**
   Puedes crear un archivo `.env` o exportar las variables mencionadas arriba.

3. **Compilar y ejecutar:**
   ```bash
   cargo run --release
   ```

El servidor estará disponible por defecto en `http://127.0.0.1:8080`.

## 📡 Endpoints de la API

### Información

- **`GET /`**: Devuelve una página informativa con el estado del servicio y los endpoints disponibles.

### Autenticación CLI

1. **`POST /auth/cli/start`**: Inicia el proceso. El cliente envía información del dispositivo y recibe una URL de
   autorización de Cognito.
2. **`GET /auth/cli/callback`**: Endpoint donde Cognito redirige al usuario tras el login exitoso. Procesa el código y
   guarda la sesión en Redis.
3. **`GET /auth/cli/status?state=<uuid>`**: El cliente CLI hace polling aquí para verificar si el usuario completó el
   login y obtener las credenciales de AWS STS.
4. **`POST /auth/cli/renew`**: Permite al cliente renovar sus credenciales de AWS usando el `refresh_token` almacenado.

---

Desarrollado por **DPAAS**.
