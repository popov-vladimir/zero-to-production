use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{spawn_app, ConfirmationLink};

#[actix_rt::test]
async fn subscribe_returns_200_if_valid_data() {
    let app = spawn_app().await;
    let body = "name=test&email=test%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_persist_valid_data() {
    let app = spawn_app().await;
    let body = "name=test&email=valid@email.com";

    Mock::given(path("email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let _ = app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("select email, name, status from subscriptions;")
        .fetch_one(&app.db_pool)
        .await
        .expect("query failed");

    assert_eq!("test", saved.name);
    assert_eq!("valid@email.com", saved.email);
    assert_eq!("pending", saved.status);
}

#[actix_rt::test]
async fn subscribe_sends_confirmation_email() {
    let app = spawn_app().await;
    

    let body = "name=test&email=test%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    match app.get_confirmation_links(email_request) {
        ConfirmationLink {html,plain_text} => assert_eq!(html, plain_text),
    }

    
}

#[actix_rt::test]
async fn subscribe_returns_400_if_missing_data() {
    let app = spawn_app().await;

    let test_cases = vec![
        // ("email=test$%40gmail.com", "missing a name"),
        // ("name=name", "missing and email"),
        // ("", "missing both name and email"),
        // ("name=\\\\&email=test@test.com", "missing name"),
        ("name=valid_name&email=543", "bad email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail when payload was {}",
            error_message
        )
    }
}
