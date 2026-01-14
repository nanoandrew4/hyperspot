#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;

use rustc_hir::{Expr, ExprKind, QPath};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Enforces that all REST API endpoints have a `.summary()` call in their `OperationBuilder` chain.
    /// This lint detects when an `OperationBuilder` chain calls `.register()` without having called
    /// `.summary()` first.
    ///
    /// ### Why is this bad?
    ///
    /// DNA Section 24 requires one-line summaries for all API endpoints to ensure:
    /// - Consistent API documentation quality
    /// - Clear, concise endpoint descriptions in generated OpenAPI specs
    /// - Better developer experience when browsing API documentation
    ///
    /// Missing summaries result in incomplete API documentation and make it harder for API consumers
    /// to understand endpoint purposes.
    ///
    /// ### Example
    ///
    /// ```rust
    /// // Error: Missing summary for API endpoint
    /// OperationBuilder::get("/users-info/v1/users")
    ///     .operation_id("users_info.list_users")
    ///     .handler(|| async { "ok" })
    ///     .require_auth(&Resource, &Action)
    ///     .require_license_features::<License>([])
    ///     .json_response(StatusCode::OK, "Success")
    ///     .register(router, &openapi);  // Triggers lint - no .summary() call
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Correct: Includes summary for API endpoint
    /// OperationBuilder::get("/users-info/v1/users")
    ///     .operation_id("users_info.list_users")
    ///     .summary("List all users")  // Summary provided
    ///     .handler(|| async { "ok" })
    ///     .require_auth(&Resource, &Action)
    ///     .require_license_features::<License>([])
    ///     .json_response(StatusCode::OK, "Success")
    ///     .register(router, &openapi);
    /// ```
    pub DE0804_API_ENDPOINT_SUMMARY,
    Deny,
    "API endpoints must have summary for documentation quality (DE0804)"
}

impl<'tcx> LateLintPass<'tcx> for De0804ApiEndpointSummary {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Look for method calls that look like OperationBuilder chains ending with .register()
        if let ExprKind::MethodCall(method_segment, receiver, _args, _span) = &expr.kind {
            if method_segment.ident.name.as_str() == "register" {
                // Check if this chain started with OperationBuilder and has summary
                if is_operation_builder_chain(receiver) && !has_summary_in_chain(receiver) {
                    cx.span_lint(
                        DE0804_API_ENDPOINT_SUMMARY,
                        method_segment.ident.span,
                        |diag| {
                            diag.primary_message(
                                "API endpoint missing required summary (DE0804)"
                            );
                            diag.help("Add .summary(\"Brief description\") to the OperationBuilder chain");
                            diag.note("DNA Section 24 requires one-line summary for all endpoints");
                        },
                    );
                }
            }
        }
    }
}

fn is_http_method(name: &str) -> bool {
    matches!(name, "get" | "post" | "put" | "patch" | "delete" | "head" | "options")
}

fn is_operation_builder_chain(expr: &Expr) -> bool {
    match &expr.kind {
        // Direct OperationBuilder method call
        ExprKind::Call(func, _args) => {
            match &func.kind {
                ExprKind::Path(QPath::Resolved(_, path)) => {
                    path.segments.iter().any(|segment| {
                        is_http_method(segment.ident.name.as_str())
                    }) && path.segments.iter().any(|segment| {
                        segment.ident.name.as_str() == "OperationBuilder"
                    })
                }
                ExprKind::Path(QPath::TypeRelative(ty, segment)) => {
                    let method_name = segment.ident.name.as_str();
                    let is_constructor = is_http_method(method_name);
                    
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

fn has_summary_in_chain(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::MethodCall(method_segment, receiver, _args, _span) => {
            let method_name = method_segment.ident.name.as_str();
            method_name == "summary" || has_summary_in_chain(receiver)
        }
        ExprKind::Call(_func, _args) => {
            // Base OperationBuilder call - no summary here
            false
        }
        _ => false,
    }
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
            "DE0804",
            "API endpoint missing required summary"
        );
    }
}
