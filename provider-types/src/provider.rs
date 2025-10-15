use std::{any::Any, collections::HashMap, pin::Pin, sync::Mutex};

use once_cell::sync::Lazy;
use toml::{Table, Value};

use crate::ProviderRuntime;

pub struct ProviderRegistration {
    pub id: &'static str,
    pub create_and_run: fn(&Value) -> Pin<Box<dyn Future<Output = ()> + Send>>,
}

inventory::collect!(ProviderRegistration);
