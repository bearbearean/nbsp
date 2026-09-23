use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::{AssertSqlSafe, PgPool};
use tower::{Service, ServiceExt};

use crate::{
    create_nbsp_router,
    database::{Invite, NbspConfig, UserInviteSettings},
};

async fn setup_test_fixture() -> (Router, PgPool) {
    let db_name_override = std::thread::current()
        .name()
        .unwrap()
        .replace("tests::", "");
    let pool = crate::database::initialize(None).await.unwrap();

    let query = AssertSqlSafe(format!("DROP DATABASE IF EXISTS {db_name_override};"));
    sqlx::query(query).execute(&pool).await.unwrap();

    let query = AssertSqlSafe(format!("CREATE DATABASE {db_name_override};"));
    sqlx::query(query).execute(&pool).await.unwrap();

    let pool = crate::database::initialize(Some(db_name_override))
        .await
        .unwrap();

    let config = NbspConfig::load(&pool).await.unwrap();
    (create_nbsp_router(&pool, &config).await, pool.clone())
}

#[tokio::test]
async fn test_axum_get_routes_as_anonymous() {
    let (router, _pool) = setup_test_fixture().await;
    let mut router = router.into_service();
    let routes = [
        // Public routes
        ("/", StatusCode::OK),
        ("/account/register", StatusCode::OK),
        ("/account/login", StatusCode::OK),
        ("/account/logout", StatusCode::SEE_OTHER),
        ("/assets/robots.txt", StatusCode::OK),
        // Redirects
        ("/robots.txt", StatusCode::PERMANENT_REDIRECT),
        // Auth-only routes (will redirect to /account/login)
        ("/account/invites", StatusCode::SEE_OTHER),
        ("/account/profile", StatusCode::SEE_OTHER),
        ("/user/nbsp", StatusCode::SEE_OTHER),
        // Non-existent routes
        ("/test-404", StatusCode::NOT_FOUND),
    ];

    for (route, expected_status_code) in routes {
        let request = Request::get(route).body(Body::empty()).unwrap();
        let response = router.ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(
            response.status(),
            expected_status_code,
            "{route} expected {expected_status_code}"
        );

        if [StatusCode::SEE_OTHER, StatusCode::PERMANENT_REDIRECT].contains(&expected_status_code) {
            assert!(response.headers().get("location").is_some());
        }
    }
}

#[tokio::test]
async fn test_axum_routes_as_authed() {
    let (router, pool) = setup_test_fixture().await;
    let mut router = router.into_service();

    let invite = {
        let mut txn = pool.begin().await.unwrap();
        let invite = Invite::create_new(-1, &mut txn).await.unwrap();
        txn.commit().await.unwrap();
        invite.invite_code.to_string()
    };

    let username = "bear";
    let password = "Password@123";

    // POST /account/register
    {
        let body = format!(
            "username={username}&\
            password={password}&\
            confirm_password={password}&\
            invite={invite}"
        );
        let request = Request::post("/account/register")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap();
        let response = router.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert!(response.headers().get("set-cookie").is_some());
        let cookies = response
            .headers()
            .get_all("set-cookie")
            .into_iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect::<Vec<String>>();
        assert_eq!(cookies.len(), 2);
    }

    // POST /account/login
    let cookies = {
        let request = Request::post("/account/login")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from("username=bear&password=Password@123"))
            .unwrap();
        let response = router.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert!(response.headers().get("set-cookie").is_some());
        let cookies = response
            .headers()
            .get_all("set-cookie")
            .into_iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect::<Vec<String>>();
        assert_eq!(cookies.len(), 2);
        cookies
    };

    // GET /user/{username}
    {
        // User exists
        let request = Request::get(format!("/user/{username}"))
            .header("Cookie", cookies.first().unwrap())
            .header("Cookie", cookies.get(1).unwrap())
            .body(Body::empty())
            .unwrap();
        let response = router.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // User does not exist
        let request = Request::get("/user/not%20exist")
            .header("Cookie", cookies.first().unwrap())
            .header("Cookie", cookies.get(1).unwrap())
            .body(Body::empty())
            .unwrap();
        let response = router.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // GET /account/invites
    {
        let request = Request::get("/account/invites")
            .header("Cookie", cookies.first().unwrap())
            .header("Cookie", cookies.get(1).unwrap())
            .body(Body::empty())
            .unwrap();
        let response = router.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // POST /account/invites
    {
        // Set available invite count to 1
        let mut txn = pool.begin().await.unwrap();
        let settings = UserInviteSettings::get_by_user_id(1, &pool).await.unwrap();
        UserInviteSettings::save_invite_count(settings.setting_id, 1, &mut txn)
            .await
            .unwrap();
        txn.commit().await.unwrap();

        // Then the first POST will work, as we have 1 code available to generate
        let request = Request::post("/account/invites")
            .header("Cookie", cookies.first().unwrap())
            .header("Cookie", cookies.get(1).unwrap())
            .body(Body::empty())
            .unwrap();
        let response = router.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // And the second POST will fail, as we then do not have codes left
        let request = Request::post("/account/invites")
            .header("Cookie", cookies.first().unwrap())
            .header("Cookie", cookies.get(1).unwrap())
            .body(Body::empty())
            .unwrap();
        let response = router.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
