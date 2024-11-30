use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value}; // For date-time handling

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub time: String,
    pub id: String,
}

pub async fn get_rover_status_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<OperationState>, (StatusCode, String)> {
    match state.redis.get(&id).await {
        Ok(value) => {
            // Assuming value is a JSON string that you can deserialize into OperationState
            match serde_json::from_str::<OperationState>(&value) {
                Ok(state_data) => Ok(Json(state_data)),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
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
    pub x: f64,
    pub y: f64,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageProcessingAPICall {
    pub image: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResponse {
    pub status: i32,
    pub image: String,
    pub image_result: Vec<ImageCoordinates>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub rover_state: i32,
    pub random_id: String,
    pub base64_image: String,
    pub image_result: Vec<ImageCoordinates>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoverData {
    pub rover_id: i32,
    pub random_id: i32,
    pub battery_status: f64,
    pub temp: f64,
    pub humidity: f64,
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
pub struct OperationState {
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
    println!("{:?}", operation);
    // operation initial state from rover to server
    println!("Operation : 1");
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
    println!("Operation : 2");
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

    println!("Operation : 3");
    // check user status from database
    let rover_status_result = state
        .db
        .client
        .query_one("CALL get_rover($1, NULL)", &[&operation.rover_id])
        .await
        .map_err(|e| {
            opt_state.error = e.to_string();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database query failed: {}", e),
            )
        })?;

    println!("Operation : 4");
    // Extract the rover_status value with explicit type annotation
    let rover_status: Option<&str> = Some(rover_status_result.get::<_, &str>("rover_status"));
    match rover_status {
        Some("1") => {
            // println!("Status is 1 : request can continue");
            opt_state.error = "".to_string();
        }
        Some("2") => {
            println!("Status is 2");
            opt_state.error = "status is 2".to_string();
            return Ok(Json(OperationResult {
                rover_state: 2,
                random_id: (&operation.random_id).to_string(),
                base64_image: "".to_string(),
                image_result: Vec::new(),
            }));
        }
        Some("3") => {
            println!("Status is 3");
            opt_state.error = "status is 3".to_string();
            return Ok(Json(OperationResult {
                rover_state: 3,
                random_id: (&operation.random_id).to_string(),
                base64_image: "".to_string(),
                image_result: Vec::new(),
            }));
        }
        Some(other) => {
            opt_state.error = other.to_string();
            eprintln!("Unexpected status: {}", other);
        }
        None => {
            opt_state.error = "None error".to_string();
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

    println!("Operation : 5");
    // Validate input
    if operation.image_data.is_null() {
        opt_state.error = "Image data is null".to_string();
        return Ok(Json(OperationResult {
            rover_state: 4,
            random_id: (&operation.random_id).to_string(),
            base64_image: "".to_string(),
            image_result: Vec::new(),
        }));
    }

    println!("Operation : 6");
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

    println!("Operation : 7");
    // Convert metadata to a JSON string
    // let url: String = format!("http://127.0.0.1:8080/data");
    let url: String = state.url;

    // Define the payload
    println!("Operation : 8");
    let payload = ImageProcessingAPICall {
        image: operation.image_data.to_string(),
    };

    println!("Operation : 9");
    // Create an HTTP client
    let client = Client::new();

    println!("Operation : 10");
    // Make the POST request
    let response = client
        .post(url)
        .json(&payload) // Attach the payload
        .send()
        .await
        .map_err(|err| {
            opt_state.error = "response from image processing API error".to_string();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Request error: {}", err),
            )
        })?;

    println!("Operation : 11");
    // build response body for image data
    let mut image_result_payload = OperationResult {
        rover_state: 1,
        random_id: (&operation.random_id).to_string(),
        base64_image: "empty".to_string(),
        image_result: Vec::new(),
    };

    println!("Operation : 12");
    // Check the status or process the response
    if response.status().is_success() {
        println!("Operation : 12.1");
        let response_body = response.text().await.map_err(|err| {
            opt_state.error = err.to_string();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Response read error: {}", err),
            )
        })?;

        println!("Operation : 12.2");
        // Parse JSON response into `ImageResponse`
        let image_data_json: ImageResponse = from_str(&response_body).map_err(|err| {
            opt_state.error = err.to_string();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JSON parse error: {}", err),
            )
        })?;

        // Assign parsed data to `image_result_payload`
        image_result_payload.image_result = image_data_json.image_result;
        image_result_payload.base64_image = image_data_json.image;
        println!("Response: {}", response_body);
    } else {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to retrieve error body".to_string());
        opt_state.error = format!("Status: {}, Body: {}", status, error_body); // Store detailed error in opt_state
        eprintln!("Failed with status: {}", status);
        opt_state.error = error_body.to_string();
        return Ok(Json(OperationResult {
            rover_state: 4,
            random_id: (&operation.random_id).to_string(),
            base64_image: "Image Processing is not working".to_string(),
            image_result: Vec::new(),
        }));
    }

    println!("Operation : 13");
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

    println!("Operation : 14");
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

    println!("Operation : 15");
    // Convert image_coordinates to a string
    let image_data_json_to_string: String =
        serde_json::to_string(&image_result_payload.image_result).map_err(|e| {
            opt_state.error = e.to_string();
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Failed to serialize metadata: {}", e),
            )
        })?;

    println!("Operation : 16");
    // Insert the operation into the database
    let result = state
        .db
        .client
        .query_one(
            "CALL insert_one_operation($1,$2,$3::FLOAT,$4::FLOAT,$5::FLOAT,$6,$7,null)",
            &[
                &operation.rover_id,
                &operation.random_id,
                &operation.battery_status,
                &operation.temp,
                &operation.humidity,
                &image_result_payload.base64_image,
                &image_data_json_to_string,
            ],
        )
        .await
        .map_err(|e| {
            opt_state.error = e.to_string();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database insertion failed: {}", e),
            )
        })?;

    println!("Operation : 17");
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

    println!("Operation : 18");
    if let Some(result_value) = result.get("result") {
        match result_value {
            "1" => {
                // Do nothing
            }
            _ => opt_state.error = "Error on storing DB".to_string(),
        }
    } else {
        opt_state.error = "Failed to retrieve result value".to_string();
    }

    println!("Operation : 19");
    // store from image modal to server on redis
    opt_state.six = true;
    opt_state.time = Utc::now().timestamp().to_string();
    // opt_state.error = "".to_string();
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

    println!("Operation : 20");
    // Return the result wrapped in a JSON response
    image_result_payload.base64_image = "".to_string();
    Ok(Json(image_result_payload))
}
