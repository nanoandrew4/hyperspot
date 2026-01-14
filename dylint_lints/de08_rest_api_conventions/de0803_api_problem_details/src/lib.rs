#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;

use rustc_hir::{Expr, ExprKind, QPath};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Enforces the use of RFC 9457 Problem Details for all 4xx/5xx error responses in REST API endpoints.
    /// This lint detects when `.json_response()` is used with error status codes instead of `.problem_response()`
    /// or convenience methods like `.error_404()`.
    ///
    /// ### Why is this bad?
    ///
    /// DNA Section 7 requires RFC 9457 Problem Details for all error responses to provide:
    /// - Standardized error format across all APIs
    /// - Machine-readable error types
    /// - Human-readable error descriptions
    /// - Additional context via extension members
    /// - Better client error handling
    ///
    /// Using plain JSON for errors leads to inconsistent error formats and makes it harder for clients
    /// to handle errors properly.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // Warning: Using plain JSON for error response
    /// OperationBuilder::get("/users/{id}")
    ///     .handler(|| async { "ok" })
    ///     .json_response(StatusCode::OK, "User found")
    ///     .json_response(StatusCode::NOT_FOUND, "User not found")  // Triggers lint
    ///     .register(router, &openapi);
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Correct: Using Problem Details for error response
    /// OperationBuilder::get("/users/{id}")
    ///     .handler(|| async { "ok" })
    ///     .json_response(StatusCode::OK, "User found")
    ///     .error_404(&openapi)  // Uses Problem Details
    ///     .register(router, &openapi);
    ///
    /// // Or use the explicit method:
    /// OperationBuilder::get("/users/{id}")
    ///     .handler(|| async { "ok" })
    ///     .json_response(StatusCode::OK, "User found")
    ///     .problem_response(&openapi, StatusCode::NOT_FOUND, "User not found")  // Uses Problem Details
    ///     .register(router, &openapi);
    /// ```
    pub DE0803_API_PROBLEM_DETAILS,
    Warn,
    "use Problem Details for 4xx/5xx error responses, not plain JSON (DE0803)"
}

impl<'tcx> LateLintPass<'tcx> for De0803ApiProblemDetails {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Look for .json_response() method calls in OperationBuilder chains
        if let ExprKind::MethodCall(method_segment, receiver, args, _span) = &expr.kind {
            if method_segment.ident.name.as_str() == "json_response" {
                if is_operation_builder_chain(receiver) && has_error_status_code(args) {
                    cx.span_lint(
                        DE0803_API_PROBLEM_DETAILS,
                        method_segment.ident.span,
                        |diag| {
                            diag.primary_message(
                                "Use Problem Details for error responses, not plain JSON (DE0803)"
                            );
                            diag.help("Use .problem_response(openapi, status, description) or convenience methods like .error_404(openapi)");
                            diag.note("DNA Section 7 requires RFC 9457 Problem Details for all 4xx/5xx responses");
                        },
                    );
                }
            }
        }
    }
}

fn is_operation_builder_chain(expr: &Expr) -> bool {
    match &expr.kind {
        // Direct OperationBuilder method call
        ExprKind::Call(func, _args) => {
            match &func.kind {
                ExprKind::Path(QPath::Resolved(_, path)) => {
                    path.segments.iter().any(|segment| {
                        let name = segment.ident.name.as_str();
                        matches!(name, "get" | "post" | "put" | "patch" | "delete" | "head" | "options")
                    }) && path.segments.iter().any(|segment| {
                        segment.ident.name.as_str() == "OperationBuilder"
                    })
                }
                ExprKind::Path(QPath::TypeRelative(ty, segment)) => {
                    let method_name = segment.ident.name.as_str();
                    let is_constructor = matches!(method_name, "get" | "post" | "put" | "patch" | "delete" | "head" | "options");
                    
                    let is_operation_builder = if let rustc_hir::TyKind::Path(QPath::Resolved(_, path)) = &ty.kind {
                         path.segments.iter().any(|segment| {
                            segment.ident.name.as_str() == "OperationBuilder"
                        })
                    } else {
                        false
                    };
                    
                    is_constructor && is_operation_builder
                }
                _ => false,
            }
        }
        // Method call on another expression (chained calls)
        ExprKind::MethodCall(_method_segment, receiver, _args, _span) => {
            is_operation_builder_chain(receiver)
        }
        _ => false,
    }
}

fn has_error_status_code(args: &[Expr]) -> bool {
    if let Some(first_arg) = args.first() {
        match &first_arg.kind {
            ExprKind::Path(QPath::Resolved(_, path)) => {
                if let Some(last_segment) = path.segments.last() {
                    is_error_status_name(last_segment.ident.name.as_str())
                } else {
                    false
                }
            }
            ExprKind::Path(QPath::TypeRelative(_ty, segment)) => {
                is_error_status_name(segment.ident.name.as_str())
            }
            _ => false,
        }
    } else {
        false
    }
}

fn is_error_status_name(name: &str) -> bool {
    matches!(name, 
        "BAD_REQUEST" | "UNAUTHORIZED" | "FORBIDDEN" | "NOT_FOUND" |
        "METHOD_NOT_ALLOWED" | "CONFLICT" | "GONE" | "UNPROCESSABLE_ENTITY" |
        "TOO_MANY_REQUESTS" | "INTERNAL_SERVER_ERROR" | "BAD_GATEWAY" |
        "SERVICE_UNAVAILABLE" | "GATEWAY_TIMEOUT"
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui_examples() {
        dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn test_comment_annotations_match_stderr() {
        let ui_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui");
        lint_utils::test_comment_annotations_match_stderr(
            &ui_dir,
            "DE0803",
            "Use Problem Details for error responses, not plain JSON"
        );
    }
}
