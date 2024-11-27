use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value}; // For date-time handling

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub time: String,
    pub id: String,
}

pub async fn test_insert_one(
    State(state): State<AppState>,
    Json(operation): Json<Operation>,
) -> Result<Json<TestResult>, (StatusCode, String)> {
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

    // Define the payload
    let payload = TestResult {
        time: Utc::now().timestamp().to_string(),
        id: result_value.to_owned(),
    };

    // Return the result wrapped in a JSON response
    Ok(Json(payload))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: String,
    pub metadata: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageCoordinates {
    pub x: i64,
    pub y: i64,
    pub z: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResponse {
    pub status: i8,
    pub image_result: Vec<ImageCoordinates>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResult {
    pub rover_state: i8,
    pub random_id: String,
    pub image_result: Vec<ImageCoordinates>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoverData {
    pub rover_id: i32,
    pub random_id: i32,
    pub battery_status: f32,
    pub temp: f32,
    pub humidity: f32,
    pub image_data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Positions {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoverResult {
    pub random_id: i32,
    pub rover_status: i32,
    pub image_data_result: Vec<Positions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OperationState {
    pub one: bool,
    pub two: bool,
    pub three: bool,
    pub four: bool,
    pub five: bool,
    pub six: bool,
    pub time: String,
    pub error: String,
}

pub async fn insert_one_from_rover(
    State(state): State<AppState>,
    Json(operation): Json<RoverData>,
) -> Result<Json<OperationResult>, (StatusCode, String)> {
    // operation initial state from rover to server
    let mut opt_state = OperationState {
        one: true,
        two: false,
        three: false,
        four: false,
        five: false,
        six: false,
        time: Utc::now().timestamp().to_string(),
        error: "".to_string(),
    };
    // store initial rover request on redis
    let _ = match state
        .redis
        .set(
            &operation.rover_id.to_string(),
            &serde_json::to_string(&opt_state).unwrap(),
        )
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    // Insert the operation into the database
    let user_result = state
        .db
        .client
        .query_one(
            "CALL get_user_for_rover($1::TEXT, $2::TEXT, NULL::TEXT)",
            &[&operation.rover_id, &operation.random_id],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database insertion failed: {}", e),
            )
        })?;

    if let Some(result_value) = user_result.get(0) {
        match result_value {
            "1" => {
                // Do nothing
            }
            _ => opt_state.error = "Error on storing DB".to_string(),
        }
    } else {
        opt_state.error = "Failed to retrieve result value".to_string();
    }

    // Validate input
    if operation.rover_id.is_positive() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Operation ID cannot be empty".to_string(),
        ));
    }

    // store from server to image modal on redis
    opt_state.two = true;
    opt_state.time = Utc::now().timestamp().to_string();
    let _ = match state
        .redis
        .set(
            &operation.rover_id.to_string(),
            &serde_json::to_string(&opt_state).unwrap(),
        )
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    // Convert metadata to a JSON string
    let image_data_json = serde_json::to_string(&operation.image_data).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Failed to serialize metadata: {}", e),
        )
    })?;

    // Make the HTTP GET request using reqwest
    let url: String = format!(
        "https://jsonplaceholder.typicode.com/posts/{}",
        &operation.rover_id
    );

    // Create an HTTP client
    let client = Client::new();

    // Define the payload
    let payload = json!({
        "time": Utc::now().timestamp().to_string(),
        "image": image_data_json,
        "userId": &operation.rover_id.to_string()
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

    // build response body for image data
    let mut image_result_payload = OperationResult {
        rover_state: 1,
        random_id: (&operation.random_id).to_string(),
        image_result: Vec::new(),
    };

    // Check the status or process the response
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
        image_result_payload.image_result = image_data_json.image_result;
        println!("Response: {}", response_body);
    } else {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to retrieve error body".to_string());
        opt_state.error = format!("Status: {}, Body: {}", status, error_body); // Store detailed error in opt_state
        eprintln!("Failed with status: {}", status);
    }

    // store from server to DB on redis
    opt_state.three = true;
    opt_state.time = Utc::now().timestamp().to_string();
    let _ = match state
        .redis
        .set(
            &operation.rover_id.to_string(),
            &serde_json::to_string(&opt_state).unwrap(),
        )
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    // store from image modal to server on redis
    opt_state.four = true;
    opt_state.time = Utc::now().timestamp().to_string();
    let _ = match state
        .redis
        .set(
            &operation.rover_id.to_string(),
            &serde_json::to_string(&opt_state).unwrap(),
        )
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    // Insert the operation into the database
    let result = state
        .db
        .client
        .query_one(
            "CALL insert_one_operation($1::TEXT, $2::TEXT, NULL::TEXT)",
            &[
                &operation.rover_id,
                &operation.random_id,
                &operation.battery_status,
                &operation.temp,
                &operation.humidity,
            ],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database insertion failed: {}", e),
            )
        })?;

    // store from  to server on redis
    opt_state.five = true;
    opt_state.time = Utc::now().timestamp().to_string();
    let _ = match state
        .redis
        .set(
            &operation.rover_id.to_string(),
            &serde_json::to_string(&opt_state).unwrap(),
        )
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    if let Some(result_value) = result.get(0) {
        match result_value {
            "1" => {
                // Do nothing
            }
            _ => opt_state.error = "Error on storing DB".to_string(),
        }
    } else {
        opt_state.error = "Failed to retrieve result value".to_string();
    }

    // store from image modal to server on redis
    opt_state.six = true;
    opt_state.time = Utc::now().timestamp().to_string();
    let _ = match state
        .redis
        .set(
            &operation.rover_id.to_string(),
            &serde_json::to_string(&opt_state).unwrap(),
        )
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    // Return the result wrapped in a JSON response
    Ok(Json(image_result_payload))
}
