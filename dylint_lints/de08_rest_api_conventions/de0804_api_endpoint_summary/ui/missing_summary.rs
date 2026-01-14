use modkit::api::OperationBuilder;
use axum::Router;
use http::StatusCode;

struct MockOpenApi;
struct Resource;
struct Action;

#[derive(Clone, Copy)]
enum License {}

impl AsRef<str> for License {
    fn as_ref(&self) -> &str { "test" }
}

impl modkit::api::operation_builder::LicenseFeature for License {}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &str { "test_resource" }
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str { "test_action" }
}

impl modkit::api::operation_builder::AuthReqResource for Resource {}
impl modkit::api::operation_builder::AuthReqAction for Action {}

impl modkit::api::OpenApiRegistry for MockOpenApi {
    fn register_operation(&self, _spec: &modkit::api::operation_builder::OperationSpec) {}
    fn ensure_schema_raw(&self, name: &str, _schemas: Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>) -> String { 
        name.to_owned() 
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

fn main() {
    let router: Router = Router::new();
    let openapi = MockOpenApi;

    let _router = OperationBuilder::get("/users-info/v1/users")
        .operation_id("users_info.list_users")
        .handler(|| async { "ok" })
        .require_auth(&Resource, &Action)
        .require_license_features::<License>([])
        .json_response(StatusCode::OK, "Success")
        // Should trigger DE0804 - API endpoint missing required summary
        .register(router, &openapi);
}
