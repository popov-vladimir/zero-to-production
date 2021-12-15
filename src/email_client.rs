use crate::domain::SubscriberEmail;

use reqwest::Client;
use std::time::Duration;

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendEmailRequest<'a> {
    from: String,
    to: String,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: String,
}


const TOKEN_HEADER_NAME: &'static str = "X-Postmark-Server-Token";

impl EmailClient {
    pub fn new(base_url: String,
               sender: SubscriberEmail,
               authorization_token: String,
               timeout: Duration,
    ) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(timeout)
                .build()
                .unwrap(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), reqwest::Error> {

        // let url = format!("{}/email",self.base_url);
        let url = reqwest::Url::parse(&self.base_url)
            .unwrap()
            .join("email")
            .unwrap();

        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_owned(),
            to: recipient.as_ref().to_owned(),
            subject,
            html_body,
            text_body,
        };

        let _builder = self.http_client
            .post(url)
            .header(TOKEN_HEADER_NAME, &self.authorization_token)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?
            ;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use fake::faker::internet::en::SafeEmail;
    use fake::{Fake, Faker};
    use crate::email_client::{EmailClient, TOKEN_HEADER_NAME};
    use wiremock::matchers::{any, header_exists, header, path, method};
    use wiremock::{Mock, MockServer, ResponseTemplate, Request};
    use fake::faker::lorem::en::{Paragraph, Sentence};


    fn subject() -> String {
        Sentence(1..3).fake()
    }

    fn content() -> String {
        Paragraph(1..2).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        let configuration = get_configuration().expect("failed to get configuration");
        EmailClient::new(base_url, email(), Faker.fake(), configuration.email_client.timeout())
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);


            result.map(|v| {
                dbg!(&v);
                v.get("From").is_some() &
                    v.get("To").is_some() &
                    v.get("Subject").is_some() &
                    v.get("HtmlBody").is_some() &
                    v.get("TextBody").is_some()
            }).unwrap_or(false)

            // match result {
            //     Err(_) => false,
            //     Ok(v) => true
            // }
            // if let Ok(body) = result {
            //     body.get("From").is_some()
            // } else {
            //     false
            // }
        }
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;

        let sender = email();

        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists(TOKEN_HEADER_NAME))
            .and(header("Content-type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;


        let subscriber_email = email();

        let outcome = email_client.send_email(
            subscriber_email,
            &subject(),
            &content(),
            &content())
            .await;

        assert!(outcome.is_ok())
    }


    use claim::*;
    use std::time::Duration;
    use crate::configuration::get_configuration;

    #[tokio::test]
    async fn send_email_succeeds_if_server_returns_200() {
        let mock_server = MockServer::start().await;
        let sender = email();
        let subscriber_email = email();
        let email_client = email_client(mock_server.uri());

        let subject: String = subject();
        let content: String = content();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        let outcome = email_client.send_email(subscriber_email, &subject, &content, &content).await;

        assert_ok!(outcome)
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = email_client(mock_server.uri());

        let subject: String = subject();
        let content: String = content();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client.send_email(subscriber_email, &subject, &content, &content).await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_too_long() {
        let mock_server = MockServer::start().await;
        let sender = email();
        let subscriber_email = email();
        let email_client = email_client(mock_server.uri());

        let subject: String = subject();
        let content: String = content();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(180)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client.send_email(subscriber_email, &subject, &content, &content).await;

        assert_err!(outcome);
    }
}
