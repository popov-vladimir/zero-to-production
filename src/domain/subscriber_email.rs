use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        //TODO: implement

        if validate_email(&s) {
            Ok(Self(s))
        } else {
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

    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::Gen;
    use rand_core::Error;

    #[derive(Clone, Debug)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);

            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email:ValidEmailFixture) -> bool
    {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    // #[test]
    // fn valid_emails_are_parsed_successfully() {
    //     let email = SafeEmail().fake();
    //     claim::assert_ok!(SubscriberEmail::parse(email));
    // }
}
