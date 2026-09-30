use crate::{SmnRequest, SmnResponse};
use crate::structures::PluginBase;

pub struct PluginManager {
    plugins: Vec<Box<dyn PluginBase>>,
}


impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn PluginBase>) {
        self.plugins.push(plugin);
    }

    pub fn init_all(&mut self) {
        for plugin in self.plugins.iter_mut() {
            println!("Initializing plugin: {}", plugin.name());
            plugin.init();
        }
    }

    pub fn route(&self, request: &SmnRequest) -> SmnResponse {
        for plugin in &self.plugins {
            if plugin.can_serve(request) {
                return plugin.serve(request);
            }
        }

        // Default fallback if no plugin handles the route
        SmnResponse::new(
            404,
            "Not Found",
            b"Not Found".to_vec(),
        )
        .with_header("Content-Type", "text/plain")
    }
}
