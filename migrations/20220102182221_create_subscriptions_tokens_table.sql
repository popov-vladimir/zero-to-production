CREATE TABLE IF NOT EXISTS subscription_tokens (
    subscription_token text NOT NULL,
    subscriber_id uuid NOT NULL  REFERENCES subscriptions (id),
    PRIMARY KEY (subscription_token)
);

