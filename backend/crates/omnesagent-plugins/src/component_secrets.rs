//! Host implementation for plugin worlds that import `secrets`.

use crate::component::PluginState;
use crate::component::bindings;
use crate::services::SecretLookupError;

macro_rules! impl_secrets_host {
    ($world:ident) => {
        impl bindings::$world::omnesagent::plugin::secrets::Host for PluginState {
            async fn get(
                &mut self,
                name: String,
            ) -> Result<String, bindings::$world::omnesagent::plugin::secrets::SecretError> {
                use bindings::$world::omnesagent::plugin::secrets::SecretError;

                self.secret(&name).map_err(|error| match error {
                    SecretLookupError::AccessDenied => SecretError::AccessDenied,
                    SecretLookupError::NotFound => SecretError::NotFound,
                    SecretLookupError::Unavailable => SecretError::Unavailable,
                })
            }
        }
    };
}

impl_secrets_host!(tool);
impl_secrets_host!(channel);
