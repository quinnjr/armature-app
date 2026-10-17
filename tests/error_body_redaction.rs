//! Script errors raised by handlers and guards are logged server-side, not
//! echoed to the client: a Rhai error message can carry source snippets and
//! whatever data the script was handling.

#[path = "support/mod.rs"]
mod support;

use armature_core::HttpRequest;

const SCRIPT: &str = r#"
    let broken = guard("Broken");
    broken.can_activate(|req| { throw "guard saw api key sk_live_123"; });

    let ctrl = controller("/x");
    ctrl.get("/handler", |req, ctx| { throw "db password is hunter2"; });

    let guarded = controller("/g");
    guarded.use_guard(broken);
    guarded.get("/", |req, ctx| { ok() });

    let app_module = create_module("AppModule");
    app_module.controllers([ctrl, guarded]);

    let app = create_app(app_module);
    app.listen(3000);
"#;

#[tokio::test]
async fn handler_error_details_are_not_returned_to_client() {
    let router = support::build_router_from_script(SCRIPT);
    let response = router
        .route(HttpRequest::new("GET", "/x/handler".to_string()))
        .await
        .expect("route should dispatch");

    assert_eq!(response.status, 500);
    let body = String::from_utf8_lossy(response.body_ref());
    assert_eq!(body, "Internal Server Error");
    assert!(!body.contains("hunter2"));
}

#[tokio::test]
async fn guard_error_details_are_not_returned_to_client() {
    let router = support::build_router_from_script(SCRIPT);
    let response = router
        .route(HttpRequest::new("GET", "/g".to_string()))
        .await
        .expect("route should dispatch");

    assert_eq!(response.status, 403);
    let body = String::from_utf8_lossy(response.body_ref());
    assert_eq!(body, "Forbidden");
    assert!(!body.contains("sk_live_123"));
}
