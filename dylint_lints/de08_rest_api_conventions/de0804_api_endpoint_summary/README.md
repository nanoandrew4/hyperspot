# DE0804: API Endpoint Summary

### What it does

Enforces that all REST API endpoints have a `.summary()` call in their `OperationBuilder` chain. This lint detects when an `OperationBuilder` chain calls `.register()` without having called `.summary()` first.

### Why is this bad?

DNA Section 24 requires one-line summaries for all API endpoints to ensure:
- Consistent API documentation quality
- Clear, concise endpoint descriptions in generated OpenAPI specs
- Better developer experience when browsing API documentation
- Improved discoverability of API functionality

Missing summaries result in incomplete API documentation and make it harder for API consumers to understand endpoint purposes.

### Known problems

None.

### Example

```rust
// Error: Missing summary for API endpoint
OperationBuilder::get("/users-info/v1/users")
    .operation_id("users_info.list_users")
    .handler(|| async { "ok" })
    .require_auth(&Resource, &Action)
    .require_license_features::<License>([])
    .json_response(StatusCode::OK, "Success")
    .register(router, &openapi);  // ❌ Triggers lint - no .summary() call
```

Use instead:

```rust
// Correct: Includes summary for API endpoint
OperationBuilder::get("/users-info/v1/users")
    .operation_id("users_info.list_users")
    .summary("List all users")  // ✅ Summary provided
    .handler(|| async { "ok" })
    .require_auth(&Resource, &Action)
    .require_license_features::<License>([])
    .json_response(StatusCode::OK, "Success")
    .register(router, &openapi);
```
