use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};
use tower::{Layer, Service};
use uuid::Uuid;

/// Layer that adds request IDs to all requests
#[derive(Clone)]
pub struct RequestIdLayer;

impl RequestIdLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RequestIdLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<S> Service<Request> for RequestIdService<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        // Generate or extract request ID
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Store in extensions for handlers to access
        req.extensions_mut().insert(request_id.clone());

        let future = self.inner.call(req);

        Box::pin(async move {
            let mut response = future.await?;
            
            // Add request ID to response headers
            if let Ok(header_value) = HeaderValue::from_str(&request_id) {
                response
                    .headers_mut()
                    .insert("x-request-id", header_value);
            }

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RequestIdLayer;
    use axum::{body::Body, http::Request, response::Response};
    use tower::{service_fn, ServiceBuilder, ServiceExt};

    #[tokio::test]
    async fn generates_request_id_when_missing() {
        let svc = ServiceBuilder::new()
            .layer(RequestIdLayer::new())
            .service(service_fn(|req: Request| async move {
                // Request extensions should contain request id
                let id = req.extensions().get::<String>().cloned();
                assert!(id.is_some());
                Ok::<_, std::convert::Infallible>(Response::new(Body::empty()))
            }));

        let resp = svc
            .oneshot(Request::new(Body::empty()))
            .await
            .expect("service should succeed");

        let header = resp.headers().get("x-request-id");
        assert!(header.is_some(), "response should carry generated request id");
    }

    #[tokio::test]
    async fn preserves_existing_request_id_header() {
        let svc = ServiceBuilder::new()
            .layer(RequestIdLayer::new())
            .service(service_fn(|req: Request| async move {
                let id = req.extensions().get::<String>().cloned();
                Ok::<_, std::convert::Infallible>(Response::new(Body::from(
                    id.unwrap_or_default(),
                )))
            }));

        let req = Request::builder()
            .header("x-request-id", "abc-123")
            .body(Body::empty())
            .unwrap();

        let resp = svc.oneshot(req).await.expect("service should succeed");
        assert_eq!(
            resp.headers().get("x-request-id").unwrap(),
            "abc-123",
            "existing header should be retained"
        );
    }
}
