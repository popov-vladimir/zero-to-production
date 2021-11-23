use crate::domain::SubscriberEmail;

use reqwest::Client;

pub struct EmailClient {
    client: Client,
    base_url: String,
    sender: SubscriberEmail
}


impl EmailClient {

    pub fn new(base_url: String, sender: SubscriberEmail)-> Self {

        Self {
            client: Client::new(),
            base_url,
            sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str
    ) -> Result <(), String> {
        todo!(

        )
    }
}


#[cfg(test)]
mod tests {
    use wiremock::MockServer;
    use crate::domain::SubscriberEmail;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url(){


        let mock_server = MockServer::start().await;

        // let sender = SubscriberEmail :: parse(SafeEmail.fake());
    }
}
