# DE0803: API Problem Details

### What it does

Enforces the use of RFC 9457 Problem Details for all 4xx/5xx error responses in REST API endpoints. This lint detects when `.json_response()` is used with error status codes instead of `.problem_response()` or convenience methods like `.error_404()`.

### Why is this bad?

DNA Section 7 requires RFC 9457 Problem Details for all error responses to provide:
- Standardized error format across all APIs
- Machine-readable error types
- Human-readable error descriptions
- Additional context via extension members
- Better client error handling

Using plain JSON for errors leads to inconsistent error formats and makes it harder for clients to handle errors properly.

### Known problems

None.

### Example

```rust
// Warning: Using plain JSON for error response
OperationBuilder::get("/users/{id}")
    .handler(|| async { "ok" })
    .json_response(StatusCode::OK, "User found")
    .json_response(StatusCode::NOT_FOUND, "User not found")  // ❌ Triggers lint
    .register(router, &openapi);
```

Use instead:

```rust
// Correct: Using Problem Details for error response
OperationBuilder::get("/users/{id}")
    .handler(|| async { "ok" })
    .json_response(StatusCode::OK, "User found")
    .error_404(&openapi)  // ✅ Uses Problem Details
    .register(router, &openapi);

// Or use the explicit method:
OperationBuilder::get("/users/{id}")
    .handler(|| async { "ok" })
    .json_response(StatusCode::OK, "User found")
    .problem_response(&openapi, StatusCode::NOT_FOUND, "User not found")  // ✅ Uses Problem Details
    .register(router, &openapi);
```
