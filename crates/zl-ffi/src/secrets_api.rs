//! The secret store half of the [`Core`] object: Kotlin keeps the API keys (in the Android
//! Keystore) and the core asks for them when a call needs one.

use std::sync::Arc;

use zl_core as core;

use crate::core_object::Core;
use crate::error::CoreError;

/// Result of a core call.
type Res<T> = Result<T, CoreError>;

/// Where the API keys are kept, implemented in Kotlin.
#[uniffi::export(with_foreign)]
pub trait SecretStore: Send + Sync {
    /// The value of `name`, if one is stored.
    fn get(&self, name: String) -> Option<String>;
    /// Stores `value` under `name` (`None` removes it); `false` when it could not be stored.
    fn set(&self, name: String, value: Option<String>) -> bool;
}

/// Adapts the foreign store to the core's trait.
struct Bridge(Arc<dyn SecretStore>);

impl core::SecretStore for Bridge {
    /// Forwards to Kotlin.
    fn get(&self, name: &str) -> Option<String> {
        self.0.get(name.to_string())
    }
    /// Forwards to Kotlin.
    fn set(&self, name: &str, value: Option<&str>) -> bool {
        self.0.set(name.to_string(), value.map(str::to_string))
    }
}

#[uniffi::export]
impl Core {
    /// Uses `store` for the API keys from now on and moves the keys an older version kept in
    /// the database into it; returns how many were moved. Call it once, right after opening.
    pub fn set_secret_store(&self, store: Arc<dyn SecretStore>) -> Res<u32> {
        Ok(self.service().set_secret_store(Arc::new(Bridge(store)))?)
    }
}
