use crate::domain::subscriber_name::SubscriberName;

pub struct NewSubscriber
{
    pub name: SubscriberName,
    pub email: String,
}
