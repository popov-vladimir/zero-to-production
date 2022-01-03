use wiremock::{Mock, ResponseTemplate, matchers::{method, path}};

use crate::helpers::spawn_app;

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

    let saved = sqlx::query!("select email, name from subscriptions;")
        .fetch_one(&app.db_pool)
        .await
        .expect("query failed");

    assert_eq!("test", saved.name);
    assert_eq!("test@gmail.com", saved.email);
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

    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s:&str| {
        let links:  Vec<_> = linkify::LinkFinder::new()
        .links(s)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
    assert_eq!(links.len(),1);
    links[0].as_str().to_owned()
    };

    let html_link = get_link (&body["HtmlBody"].as_str().unwrap());
    let text_link = get_link (&body["TextBody"].as_str().unwrap());

    assert_eq!(html_link,text_link);
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
