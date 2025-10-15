use std::{any::Any, collections::HashMap, sync::Mutex};

use once_cell::sync::Lazy;
use provider_types::ProviderRuntime;
use toml::{Table, Value};

pub type ConfigFactory = Box<dyn Fn(&Value) -> Box<dyn Any> + Send + Sync>;

pub static REGISTRY: Lazy<Mutex<HashMap<String, ConfigFactory>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn register_provider<F>(id: &'static str, factory: F)
where
    F: Fn(&Value) -> Box<dyn Any> + 'static + Send + Sync,
{
    REGISTRY
        .lock()
        .unwrap()
        .insert(id.to_string(), Box::new(factory));
}

pub struct ProviderRegistration {
    pub id: &'static str,
    pub runtime: &'static dyn ProviderRuntime,
}

inventory::collect!(ProviderRegistration);

pub fn initialize_providers(raw_config: &Table) {
    if let Some(enabled) = raw_config.get("providers").and_then(|v| v.as_array()) {
        let enabled_ids: Vec<_> = enabled.iter().filter_map(|v| v.as_str()).collect();

        for reg in inventory::iter::<ProviderRegistration> {
            if enabled_ids.contains(&reg.id) {
                let provider_cfg = raw_config.get(reg.id).expect("config should exist");

                reg.runtime.maybe_register(provider_cfg);
            }
        }
    }
}

// example final parse entry point, optional praceholder
pub fn parse_all_configs(raw_config: &Table) -> HashMap<String, Box<dyn Any>> {
    let mut out = HashMap::new();

    for (id, factory) in REGISTRY.lock().unwrap().iter() {
        if let Some(value) = raw_config.get(id) {
            out.insert(id.clone(), factory(value));
        }
    }

    out
}
