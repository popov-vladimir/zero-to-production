use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(name: String) -> Result<SubscriberName, String> {
        let is_empty = name.trim().is_empty();

        let too_long = name.graphemes(true).count() > 256;

        let forbidden_characters = ['\\', '/', ')', '(', ';', '"'];

        let contains_forbidden_characters = name
            .chars()
            .any(|c| { forbidden_characters.contains(&c) });
        // println!("name {} is_empty = {}, too_long = {}, contains_forbidden = {}",
        //          name,
        //          is_empty,
        //          too_long,
        //          contains_forbidden_characters);

        if is_empty || too_long || contains_forbidden_characters {
            Err(format!("'{}' is not a valid name", name))
        } else {
            Ok(Self(name))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    // use claim::{assert_err, assert_ok};

    #[test]
    fn long_graphemes_got_rejected() {
        claim::assert_err!(SubscriberName::parse("ы".repeat(257)));
    }
}
