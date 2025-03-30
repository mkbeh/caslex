use axum::Router;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable as ScalarServable};

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    servers(
            (url = "http://localhost:9000", description = "local")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

pub fn get_openapi_router(router: OpenApiRouter, docs_url: String) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(router)
        .split_for_parts();

    router.merge(Scalar::with_url(docs_url, api))
}
