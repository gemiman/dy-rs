use dy_rs::prelude::*;
use utoipa::OpenApi;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
struct User {
    id: Uuid,
    email: String,
    name: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate, ToSchema)]
struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,

    #[validate(length(
        min = 2,
        max = 100,
        message = "Name must be between 2 and 100 characters"
    ))]
    name: String,
}

#[derive(Deserialize, Validate, ToSchema)]
struct UpdateUserRequest {
    #[validate(length(
        min = 2,
        max = 100,
        message = "Name must be between 2 and 100 characters"
    ))]
    name: Option<String>,
}

// In-memory "database" for demo purposes
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Database = Arc<Mutex<HashMap<Uuid, User>>>;

/// Create a new user
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses((status = 200, description = "User created", body = User))
)]
async fn create_user(
    State(db): State<Database>,
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> ApiResult<User> {
    let user = User {
        id: Uuid::new_v4(),
        email: payload.email,
        name: payload.name,
        created_at: Utc::now(),
    };

    db.lock().unwrap().insert(user.id, user.clone());

    Ok(Json(user))
}

/// Get all users
#[utoipa::path(
    get,
    path = "/users",
    responses((status = 200, description = "List users", body = [User]))
)]
async fn list_users(State(db): State<Database>) -> ApiResult<Vec<User>> {
    let users: Vec<User> = db.lock().unwrap().values().cloned().collect();
    Ok(Json(users))
}

/// Get a user by ID
#[utoipa::path(
    get,
    path = "/users/{id}",
    params(("id" = String, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found")
    )
)]
async fn get_user(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    let db = db.lock().unwrap();
    let user = db
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", id)))?;

    Ok(Json(user.clone()))
}

/// Update a user
#[utoipa::path(
    patch,
    path = "/users/{id}",
    params(("id" = String, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 404, description = "User not found")
    )
)]
async fn update_user(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> Result<Json<User>, ApiError> {
    let mut db = db.lock().unwrap();
    let user = db
        .get_mut(&id)
        .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", id)))?;

    if let Some(name) = payload.name {
        user.name = name;
    }

    Ok(Json(user.clone()))
}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(("id" = String, Path, description = "User ID")),
    responses(
        (status = 200, description = "User deleted", body = User),
        (status = 404, description = "User not found")
    )
)]
async fn delete_user(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    let mut db = db.lock().unwrap();
    let user = db
        .remove(&id)
        .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", id)))?;

    Ok(Json(user))
}

fn user_routes() -> Router<Database> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_user))
        .route("/users/{id}", patch(update_user))
        .route("/users/{id}", delete(delete_user))
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "dy-rs REST API",
        version = "0.1.0",
        description = "Example REST API built with dy-rs"
    ),
    paths(create_user, list_users, get_user, update_user, delete_user),
    components(schemas(User, CreateUserRequest, UpdateUserRequest))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // Create shared database
    let db: Database = Arc::new(Mutex::new(HashMap::new()));

    // Build and run the app
    App::new()
        .with_openapi(ApiDoc::openapi())
        .auto_configure()
        .mount(user_routes().with_state(db))
        .run()
        .await
        .unwrap();
}
