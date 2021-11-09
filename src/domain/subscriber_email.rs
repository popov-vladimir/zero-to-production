use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail (String);

impl SubscriberEmail{

    fn parse(s: String) -> Result<SubscriberEmail,String> {
        //TODO: implement

        if validate_email(&s) {
            Ok(Self(s))
        }
        else {
            Err(format!("{} is not a valid email", s))
        }


    }

}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[cfg(test)]
mod tests {


    use super::SubscriberEmail;

    #[test]
    fn missing_email_symbol_is_rejected() {

        let email = "123domain.com".to_string();

        claim::assert_err!(SubscriberEmail::parse(email));

    }
    #[test]
    fn missing_email_subject_is_rejected() {

        let email = "@†domain.com".to_string();

        claim::assert_err!(SubscriberEmail::parse(email));

    }



}
