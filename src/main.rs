mod action;

use axum::http::Method;
use axum::Router;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shuttle_runtime::SecretStore;
use std::fmt::Display;
use std::sync::Arc;
use std::time::SystemTime;
use axum::routing::{delete, get, post, put};
use shuttle_runtime::__internals::Context;
use tokio_postgres::{Client, NoTls};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct DbState {
    pub client: Arc<Client>,
}

static KEYS: Lazy<Keys> = Lazy::new(|| {
    // note that in production, you will probably want to use a random SHA-256 hash or similar
    let secret = "JWT_SECRET".to_string();
    Keys::new(secret.as_bytes())
});

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let secret = secrets
        .get("MY_SECRET_KEY")
        .context("secret was not found")?;

    let db_connection = secrets
        .get("DB_CONNECTION")
        .context("secret was not found")?;

    let (client, connection) = tokio_postgres::connect(&db_connection, NoTls)
        .await
        .expect("Failed to connect to the database");

    // Spawn the connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let state = DbState {
        client: Arc::new(client),
    };

    println!("{}", secret);

    let app = Router::new()
        .nest_service("/", ServeDir::new("assets")) // http://127.0.0.1:8000/
        .nest_service("/auth", ServeDir::new("assets/auth")) // http://127.0.0.1:8000/auth/index.html
        .route("/public", get(public))
        .route("/private", get(private))
        .route("/login", post(login))
        .route("/api/todo", get(crate::action::todo::select)) // http://127.0.0.1:8000/api/todo
        .route("/api/todo", post(crate::action::todo::insert_one))
        .route("/api/todo", put(crate::action::todo::update_one))
        .route("/api/todo/:id", delete(crate::action::todo::delete_one))
        .with_state(state)
        .layer(cors);

    Ok(app.into())
}

async fn public() -> &'static str {
    // A public endpoint that anyone can access
    "Welcome to the public area :)"
}

async fn private(claims: Claims) -> Result<String, AuthError> {
    // Send the protected data to the user
    Ok(format!(
        "Welcome to the protected area :)\nYour data:\n{claims}",
    ))
}

async fn login(Json(payload): Json<AuthPayload>) -> Result<Json<AuthBody>, AuthError> {
    // Check if the user sent the credentials
    // println!("Received payload: {:?}", payload);

    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    // Here you can check the user credentials from a database
    // use email or password
    if payload.client_id != "foo" || payload.client_secret != "bar" {
        return Err(AuthError::WrongCredentials);
    }

    // add 5 minutes to current unix epoch time as expiry date/time
    let exp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 300;

    let claims = Claims {
        sub: "b@b.com".to_owned(),
        company: "ACME".to_owned(),
        // Mandatory expiry time as UTC timestamp - takes unix epoch
        exp: usize::try_from(exp).unwrap(),
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    // Send the authorized token
    Ok(Json(AuthBody::new(token)))
}

// allow us to print the claim details for the private route
impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Email: {}\nCompany: {}", self.sub, self.company)
    }
}

// implement a method to create a response type containing the JWT
impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

// implement FromRequestParts for Claims (the JWT struct)
// FromRequestParts allows us to use Claims without consuming the request
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

// implement IntoResponse for AuthError so we can use it as an Axum response type
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

// encoding/decoding keys - set in the static `once_cell` above
struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

// the JWT claim
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

// the response that we pass back to HTTP client once successfully authorised
#[derive(Debug, Serialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
}

// the request type - "client_id" is analogous to a username, client_secret can also be interpreted as a password
#[derive(Debug, Deserialize)]
struct AuthPayload {
    client_id: String,
    client_secret: String,
}

// error types for auth errors
#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
