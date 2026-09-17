//! The HTTP surface: the routes, and the two rules they all follow.
//!
//! **Every refusal has the same shape.** `{error, detail, context}`, including
//! the ones serde raises — a body with an unknown field or a bad enum arrives
//! through [`ApiJson`], which turns axum's rejection into the same three fields
//! a compiler refusal uses. A client that can render one refusal can render
//! all of them.
//!
//! **CORS is open, and that is a decision rather than an oversight.** A console
//! is served from somewhere else entirely and embeds these panels; the rig
//! network is the security boundary, as it is for vstimd and statemachined.

pub mod calibration_routes;
pub mod config_routes;
pub mod device_routes;
pub mod elements;
pub mod openapi;
pub mod state_routes;
pub mod zone_routes;

use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequest;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::daemon_state::Daemon;
use crate::model::ApiError;

/// A JSON body that refuses in this API's own words.
///
/// `deny_unknown_fields` is on every input type, so a typo is an error rather
/// than a default — but axum's own rejection is a plain string with a status,
/// which a panel would have to render as mystery text. This keeps serde's
/// message (it names the field and the line) and puts it in `detail`.
pub struct ApiJson<T>(pub T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ApiError;

    async fn from_request(request: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(request, state).await {
            Ok(axum::Json(value)) => Ok(ApiJson(value)),
            Err(rejection) => Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "bad_body",
                rejection.body_text(),
            )),
        }
    }
}

pub fn router(daemon: Arc<Daemon>) -> Router {
    Router::new()
        .route("/api/device", get(device_routes::read_device))
        .route("/api/device/connect", post(device_routes::connect))
        .route("/api/device/firmware", get(device_routes::read_firmware))
        .route("/api/device/monitor", get(device_routes::read_wire_log))
        .route("/api/device/monitor/stream", get(device_routes::wire_stream))
        .route("/api/state", get(state_routes::read_state))
        .route("/api/stream", get(state_routes::state_stream))
        .route("/api/position/zero", post(state_routes::zero_position))
        .route(
            "/api/calibration",
            get(calibration_routes::read_calibration).put(calibration_routes::replace_calibration),
        )
        .route(
            "/api/calibration/ball",
            get(calibration_routes::read_ball).put(calibration_routes::replace_ball),
        )
        .route("/api/calibration/measure/start", post(calibration_routes::start_measurement))
        .route("/api/calibration/measure/finish", post(calibration_routes::finish_measurement))
        .route("/api/calibration/measure/apply", post(calibration_routes::apply_measurement))
        .route("/api/zone-sets", get(zone_routes::list_zone_sets))
        .route(
            "/api/zone-sets/{name}",
            get(zone_routes::read_zone_set).put(zone_routes::replace_zone_set),
        )
        .route("/api/zone-sets/{name}/validate", post(zone_routes::validate_zone_set))
        .route("/api/zones", get(zone_routes::read_armed))
        .route("/api/zones/arm", post(zone_routes::arm))
        .route("/api/zones/disarm", post(zone_routes::disarm))
        .route("/api/zones/save", post(zone_routes::save_to_flash))
        .route("/api/config", get(config_routes::read_config).patch(config_routes::patch_config))
        .route("/api/lines", get(config_routes::read_lines))
        .route("/api/openapi.json", get(openapi::openapi_document))
        .merge(elements::routes())
        .layer(CorsLayer::permissive())
        .with_state(daemon)
}
