CREATE TABLE server_subscriptions (
    server_id BIGINT NOT NULL,
    subscription_type INTEGER NOT NULL,
    notification_channel_id BIGINT NOT NULL,
    
    role_id_to_mention BIGINT,

    created_at INTEGER DEFAULT CURRENT_TIMESTAMP NOT NULL,
    modified_at INTEGER,

    PRIMARY KEY(server_id, subscription_type)
);