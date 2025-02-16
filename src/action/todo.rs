use crate::action::rover::ImageResponse;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value};
use tokio::time::Duration;
use uuid::Uuid;

use super::rover::{ImageCoordinates, OperationResult, RoverData};

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub todo: String,
    pub status: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub user_id: i32,
    pub id: i32,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: String,
    pub metadata: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResults {
    pub id: String,
}

pub async fn insert_one_json(
    State(state): State<AppState>,
    Json(operation): Json<Operation>,
) -> Result<Json<OperationResults>, (StatusCode, String)> {
    // Validate input
    if operation.id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Operation ID cannot be empty".to_string(),
        ));
    }

    // Convert metadata to a JSON string
    let metadata_json = serde_json::to_string(&operation.metadata).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Failed to serialize metadata: {}", e),
        )
    })?;

    // Insert the operation into the database
    let result = state
        .db
        .client
        .query_one(
            "CALL insert_one_test($1::TEXT, $2::TEXT, NULL::TEXT)",
            &[&operation.id, &metadata_json],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database insertion failed: {}", e),
            )
        })?;

    // Optionally, you can extract the result if needed
    let result_value = result.get::<_, &str>(0);

    // Return the result wrapped in a JSON response
    Ok(Json(OperationResults {
        id: result_value.to_owned(),
    }))
}

pub async fn select(
    State(state): State<AppState>,
) -> Result<Json<Vec<Todo>>, (StatusCode, String)> {
    let todos = state
        .db
        .client
        .query(
            "SELECT id, todo, status FROM \"railway\".\"public\".\"todo\"",
            &[],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter()
        .map(|row| Todo {
            id: row.get(0),
            todo: row.get(1),
            status: row.get(2),
        })
        .collect();

    Ok(Json(todos))
}

pub async fn insert_one(
    State(state): State<AppState>,
    Json(new_todo): Json<Todo>,
) -> Result<Json<Todo>, (StatusCode, String)> {
    // Convert the UUID to a string
    let uuid_string = Uuid::new_v4().to_string();

    let row = state
        .db
        .client
        .query_one(
            "CALL insert_one_todo($1, $2, $3, NULL, NULL, NULL)",
            &[&uuid_string, &new_todo.todo, &new_todo.status],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let inserted_todo = Todo {
        id: row.get(0),
        todo: row.get(1),
        status: row.get(2),
    };

    Ok(Json(inserted_todo))
}

pub async fn update_one(
    State(state): State<AppState>,
    Json(new_todo): Json<Todo>,
) -> Result<Json<Todo>, (StatusCode, String)> {
    let row = state
        .db
        .client
        .query_one(
            "CALL update_one_todo($1, $2, $3, NULL, NULL, NULL)",
            &[&new_todo.id, &new_todo.todo, &new_todo.status],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let inserted_todo = Todo {
        id: row.get("o_id"),
        todo: row.get("o_todo"),
        status: row.get("o_status"),
    };

    Ok(Json(inserted_todo))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = state
        .db
        .client
        .query_one("CALL delete_todo($1, NULL)", &[&id])
        .await;

    match result {
        Ok(row) => {
            let deleted: bool = row.get("o_deleted");
            if deleted {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    format!("Todo with id {} not found", id),
                ))
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn add_one_redis(
    State(state): State<AppState>,
    Json(data): Json<Todo>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Use redis_state
    match state.redis.set(&data.id, &data.todo).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn get_one_redis(
    State(state): State<AppState>,
    Path(redis_id): Path<String>,
) -> Result<Json<String>, (StatusCode, String)> {
    // Use redis_state
    match state.redis.get(&redis_id).await {
        Ok(value) => Ok(Json(value)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn delete_one_redis(
    State(state): State<AppState>,
    Path(redis_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Use redis_state
    match state.redis.delete(&redis_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Handler to fetch data from the API
pub async fn get_data_external_url(
    Path(id): Path<String>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let url: String = format!("https://jsonplaceholder.typicode.com/posts/{}", id);

    tokio::time::sleep(Duration::from_secs(1)).await; // delay for testing response time

    // Make the HTTP GET request using reqwest
    let response = reqwest::get(&url).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Request error: {}", err),
        )
    })?;

    // Parse the JSON response
    let post = response.json::<Post>().await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Deserialization error: {}", err),
        )
    })?;

    Ok(Json(post))
}

pub async fn post_data_external_url(
    Json(post): Json<Post>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let url = "https://jsonplaceholder.typicode.com/posts/";

    let client = Client::new(); // Consider reusing the client for better performance

    println!("Operation : 9");

    // Make the POST request
    let response = client
        .post(url)
        .json(&post) // Attach the payload
        .send()
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Request error: {}", err),
            )
        })?;

    // Check for non-200 status codes
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Request failed with status: {}", response.status()),
        ));
    }

    // Parse the JSON response
    let post = response.json::<Post>().await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Deserialization error: {}", err),
        )
    })?;

    Ok(Json(post))
}

// test user before insert the operation into the database
pub async fn get_user(
    State(state): State<AppState>, // Assuming `AppState` is wrapped in `Arc` for thread safety
    Path(id): Path<i32>,
) -> Result<Json<String>, (StatusCode, String)> {
    // Query the database to get rover status
    println!("{:?}", id);
    let rover_status_result = state
        .db
        .client
        .query_one("CALL get_rover($1, NULL)", &[&id])
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database query failed: {}", e),
            )
        })?;

    // Extract the rover_status value with explicit type annotation
    let rover_status: Option<&str> = Some(rover_status_result.get::<_, &str>("rover_status"));

    match rover_status {
        Some("1") => {
            println!("Status is 1");
        }
        Some("2") => {
            println!("Status is 2");
        }
        Some(other) => {
            eprintln!("Unexpected status: {}", other);
        }
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "rover_status not found",
                    "details": "The database query did not return a rover_status field."
                }))
                .to_string(),
            ));
        }
    }

    // Convert `rover_status` to a `String` and wrap in `Json`
    Ok(Json(
        rover_status
            .unwrap_or("Unknown") // Provide a fallback value if needed
            .to_string(),
    ))
}

pub async fn add_operation(
    State(state): State<AppState>,
    Path(redis_id): Path<String>,
) -> Result<Json<OperationResults>, (StatusCode, String)> {
    let user_result = state
        .db
        .client
        .query_one(
            "CALL add_one_rover_operation($1::TEXT, NULL::TEXT)",
            &[&redis_id],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database query failed: {}", e),
            )
        })?;

    let result = OperationResults {
        id: user_result.get("id_result"),
    };

    Ok(Json(result))
}

pub async fn api_external() -> Result<Json<OperationResult>, (StatusCode, String)> {
    let url: String =
        format!("https://test-railway-fastapi-backend-production.up.railway.app/data");

    // Create an HTTP client
    let client = Client::new();

    // Define the payload
    let payload = json!({
        "image": "image_data_json",
        "randomId": "test",
    });

    // Make the POST request
    let response = client
        .post(&url)
        .json(&payload) // Attach the payload
        .send()
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Request error: {}", err),
            )
        })?;

    // Prepare response payload
    let mut image_result_payload = OperationResult {
        rover_state: 0,
        random_id: "".to_string(),
        base64_image: "".to_string(),
        image_result: Vec::new(),
    };

    // Check the response status
    if response.status().is_success() {
        let response_body = response.text().await.map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Response read error: {}", err),
            )
        })?;

        // Parse JSON response into `ImageResponse`
        let image_data_json: ImageResponse = from_str(&response_body).map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JSON parse error: {}", err),
            )
        })?;

        // Assign parsed data to `image_result_payload`
        image_result_payload.rover_state = image_data_json.status;
        image_result_payload.image_result = image_data_json.image_result;
        println!("Response: {}", response_body);
    } else {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to retrieve error body".to_string());
        eprintln!("Failed with status: {}, Body: {}", status, error_body);

        return Err((
            StatusCode::BAD_REQUEST,
            format!("Request failed: {}", error_body),
        ));
    }

    Ok(Json(image_result_payload))
}

pub async fn test_ping(
    State(state): State<AppState>,
) -> Result<Json<String>, (StatusCode, String)> {
    let ping = "ping".to_string();

    // Call the stored procedure and fetch the output
    let result = state
        .db
        .client
        .query_one("CALL get_ping($1, null)", &[&ping])
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Extract the output from the query result
    let pong: String = result.get(0); // Assuming 'pong' is the first (and only) returned field

    Ok(Json(pong))
}

pub async fn test_insert_one_from_rover(
    State(state): State<AppState>,
    Json(operation): Json<RoverData>,
) -> Result<Json<OperationResult>, (StatusCode, String)> {
    println!("{:?}", operation);
    let rover_status_result = state
        .db
        .client
        .query_one("CALL get_rover($1, NULL)", &[&operation.rover_id])
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database query failed: {}", e),
            )
        })?;
    let db_rover_status: Option<&str> = Some(rover_status_result.get::<_, &str>("rover_status"));
    // response payload
    let mut image_result_payload = OperationResult {
        rover_state: 0,
        random_id: (&operation.random_id).to_string(),
        base64_image: "".to_string(),
        image_result: Vec::new(),
    };
    match db_rover_status {
        Some("0") => {
            image_result_payload.rover_state = 0;
            image_result_payload.image_result = vec![];
        }
        Some("1") => {
            image_result_payload.rover_state = 1;
            image_result_payload.image_result = vec![
                ImageCoordinates {
                    x: 100.0,
                    y: 500.0,
                    confidence: 0.5,
                },
                ImageCoordinates {
                    x: 200.0,
                    y: 1000.0,
                    confidence: 0.5,
                },
                ImageCoordinates {
                    x: 300.0,
                    y: 1500.0,
                    confidence: 0.5,
                },
                ImageCoordinates {
                    x: 400.0,
                    y: 2000.0,
                    confidence: 0.5,
                },
            ];
        }
        Some("4") => {
            image_result_payload.rover_state = 4;
            image_result_payload.image_result = vec![];
        }
        Some(_) => {
            image_result_payload.rover_state = db_rover_status
                .unwrap()
                .parse::<i32>()
                .expect("Failed to parse string to i32");
            image_result_payload.image_result = vec![];
        }
        None => {
            image_result_payload.rover_state = 0;
        }
    }
    println!("{:?}", image_result_payload);
    Ok(Json(image_result_payload))
}
