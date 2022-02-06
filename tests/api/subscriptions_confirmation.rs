use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[actix_rt::test]
async fn confirmation_wo_token_rejected() {
    let app = spawn_app().await;

    let endpoint = format!("{}/subscriptions/confirm", app.address);

    let res = reqwest::get(&endpoint).await.unwrap();

    assert_eq!(res.status(), 400, "request w/o a token shoud have failed");
}

#[actix_rt::test]
async fn the_link_returned_by_subscribe_returns_200_if_called() {
    let app = spawn_app().await;

    let body = "name=test&email=test@gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_link = app.get_confirmation_links(email_request);

    assert_eq!(confirmation_link.plain_text.host_str().unwrap(),"127.0.0.1");

    let response = reqwest::get(confirmation_link.plain_text)
    .await
    .unwrap();

    assert_eq!(response.status().as_u16(),200);

}

#[actix_rt::test]
async fn clicking_the_link_confirms_subscription() {
    let app = spawn_app().await;

    let body = "name=test&email=test@gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_link = app.get_confirmation_links(email_request);

    assert_eq!(confirmation_link.plain_text.host_str().unwrap(),"127.0.0.1");

     reqwest::get(confirmation_link.plain_text)
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch subscription status");

    assert_eq!(saved.name, "test");
    assert_eq!(saved.email, "test@gmail.com");
    assert_eq!(saved.status, "confirmed");

}
