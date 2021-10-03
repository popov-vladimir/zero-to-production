use std::net::TcpListener;
use zero2prod::run;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    // We return the application address to the caller!
    format!("http://127.0.0.1:{}", port)
}

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        // Use the returned application address
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_200_if_valid_data() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let body = "name=test&email=test%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_returns_400_if_missing_data() {
    let app_address = spawn_app();

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("email=test$%40gmail.com", "missing a name"),
        ("name=name", "missing and email"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("failed to send request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail when payload was {}",
            error_message
        )
    }
}
