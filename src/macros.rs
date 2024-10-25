#[macro_export]
macro_rules! define_handle {
    ( $name:ident ) => {
        struct $name;

        impl poise::serenity_prelude::prelude::TypeMapKey for $name {
            type Value = tokio::task::JoinHandle<()>;
        }
    };
}
