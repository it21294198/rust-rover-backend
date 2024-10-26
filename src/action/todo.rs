use crate::DbState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub todo: String,
    pub status: i32,
}

pub async fn select(State(state): State<DbState>) -> Result<Json<Vec<Todo>>, (StatusCode, String)> {
    let todos = state
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
            status: row.get(2)
        })
        .collect();

    Ok(Json(todos))
}

pub async fn insert_one(
    State(state): State<DbState>,
    Json(new_todo): Json<Todo>,
) -> Result<Json<Todo>, (StatusCode, String)> {
    // Convert the UUID to a string
    let uuid_string = Uuid::new_v4().to_string();

    let row = state.client.query_one(
        "CALL insert_one_todo($1, $2, $3, NULL, NULL, NULL)",
        &[&uuid_string, &new_todo.todo,&new_todo.status],
    )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let inserted_todo = Todo {
        id: row.get(0),
        todo: row.get(1),
        status: row.get(2)
    };

    Ok(Json(inserted_todo))
}

pub async fn update_one(
    State(state): State<DbState>,
    Json(new_todo): Json<Todo>,
) -> Result<Json<Todo>, (StatusCode, String)> {
    let row = state
        .client
        .query_one(
            "CALL update_one_todo($1, $2, $3, NULL, NULL, NULL)",
            &[
                &new_todo.id,
                &new_todo.todo,
                &new_todo.status
            ],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let inserted_todo = Todo {
        id: row.get("o_id"),
        todo: row.get("o_todo"),
        status: row.get("o_status")
    };

    Ok(Json(inserted_todo))
}

pub async fn delete_one(
    State(state): State<DbState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = state
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
