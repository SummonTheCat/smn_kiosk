use crate::{SmnRequest, SmnResponse};

pub trait PluginBase {
    /// Human-readable identifier for the plugin
    fn name(&self) -> &str;

    /// Called once when the server starts
    fn init(&mut self);

    /// Determines whether this plugin should handle the request
    fn can_serve(&self, request: &SmnRequest) -> bool;

    /// Handles the request and returns a response
    fn serve(&self, request: &SmnRequest) -> SmnResponse;
}
