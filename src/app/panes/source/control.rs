use super::settings::Settings;
use serde::{Deserialize, Serialize};

/// Source control
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Control {
    pub(crate) settings: Settings,
    pub(crate) open: bool,
}

impl Control {
    pub(crate) const fn new() -> Self {
        Self {
            settings: Settings::new(),
            open: false,
        }
    }
}
