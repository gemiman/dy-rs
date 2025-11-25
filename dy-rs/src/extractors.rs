use axum::{
    Json,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use validator::Validate;

/// Extractor that deserializes and validates JSON payloads
///
/// # Example
///
/// ```rust,ignore
/// use dy_rs::prelude::*;
/// use validator::Validate;
///
/// #[derive(Deserialize, Validate)]
/// struct CreateUser {
///     #[validate(email)]
///     email: String,
///     #[validate(length(min = 8))]
///     password: String,
/// }
///
/// async fn create_user(
///     ValidatedJson(payload): ValidatedJson<CreateUser>
/// ) -> ApiResult<User> {
///     // payload is guaranteed to be valid
///     Ok(Json(user))
/// }
/// ```
pub struct ValidatedJson<T>(pub T);

#[derive(Serialize)]
struct ValidationErrorResponse {
    code: String,
    message: String,
    errors: Vec<ValidationFieldError>,
}

#[cfg(test)]
mod tests {
    use super::ValidatedJson;
    use axum::{body::Body, extract::FromRequest, http::Request};
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    struct TestPayload {
        #[validate(length(min = 3))]
        name: String,
    }

    #[tokio::test]
    async fn validated_json_accepts_valid_payload() {
        let req = Request::builder()
            .uri("/")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"name":"abc"}"#))
            .unwrap();

        let result = ValidatedJson::<TestPayload>::from_request(req, &()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.name, "abc");
    }

    #[tokio::test]
    async fn validated_json_rejects_invalid_payload() {
        let req = Request::builder()
            .uri("/")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"name":"a"}"#))
            .unwrap();

        let result = ValidatedJson::<TestPayload>::from_request(req, &()).await;
        assert!(result.is_err(), "expected validation error for short name");
    }
}

#[derive(Serialize)]
struct ValidationFieldError {
    field: String,
    message: String,
}

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    fn from_request(
        req: Request,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // First, extract JSON
            let Json(value) = Json::<T>::from_request(req, state)
                .await
                .map_err(|rejection| {
                    tracing::error!("JSON deserialization failed: {:?}", rejection);

                    let error_response = ValidationErrorResponse {
                        code: "INVALID_JSON".to_string(),
                        message: "Invalid JSON payload".to_string(),
                        errors: vec![],
                    };

                    (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
                })?;

            // Then validate
            value.validate().map_err(|validation_errors| {
                tracing::error!("Validation failed: {:?}", validation_errors);

                let errors: Vec<ValidationFieldError> = validation_errors
                    .field_errors()
                    .into_iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |error| ValidationFieldError {
                            field: field.to_string(),
                            message: error
                                .message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| "Validation failed".to_string()),
                        })
                    })
                    .collect();

                let error_response = ValidationErrorResponse {
                    code: "VALIDATION_ERROR".to_string(),
                    message: "Request validation failed".to_string(),
                    errors,
                };

                (StatusCode::UNPROCESSABLE_ENTITY, Json(error_response)).into_response()
            })?;

            Ok(ValidatedJson(value))
        }
    }
}
