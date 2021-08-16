// Copyright 2021 Masaya Suzuki
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a plugin.
///
/// A plugin provides modules. For example, "i3" plugin may provide a workspace switcher module and
/// a window title module.
#[derive(Serialize, Deserialize)]
pub(crate) struct PluginConfig {
    /// Name of the plugin.
    pub(crate) name: String,

    /// Configuration of the plugin. The schema depends on the plugin.
    #[serde(default)]
    pub(crate) config: serde_json::Value,
}

/// Configuration for a module.
///
/// Module is one component shown in a bar.
#[derive(Serialize, Deserialize)]
pub(crate) struct ModuleConfig {
    /// Name of the module.
    pub(crate) name: String,

    /// Configuration of the module. The schema depends on the module.
    #[serde(default)]
    pub(crate) config: serde_json::Value,
}

/// Configuration for a monitor.
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct MonitorConfig {
    /// Hide the bar for this monitor.
    #[serde(default)]
    pub(crate) hidden: bool,

    /// Hight of the bar.
    #[serde(default)]
    pub(crate) height: Option<i32>,

    /// Modules on the left side.
    #[serde(default)]
    pub(crate) left_modules: Vec<ModuleConfig>,

    /// Modules on the center.
    #[serde(default)]
    pub(crate) center_modules: Vec<ModuleConfig>,

    /// Modules on the right side.
    #[serde(default)]
    pub(crate) right_modules: Vec<ModuleConfig>,
}

/// Configuration for the application.
#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    /// Disable loading the default CSS.
    #[serde(default)]
    pub(crate) disable_default_css: bool,

    /// The path to the GTK CSS file.
    ///
    /// If the path is relative, it'll be relative from the XDG_CONFIG_DIR.
    #[serde(default)]
    pub(crate) css_path: String,

    /// Plugin configurations. Only plugins configured here are activated.
    #[serde(default)]
    pub(crate) plugins: Vec<PluginConfig>,

    /// Monitor configurations.
    ///
    /// Each config is keyed by a monitor's model (e.g. "HDMI-1").
    #[serde(default)]
    pub(crate) monitors: HashMap<String, MonitorConfig>,

    /// Default monitor configuration. If a monitor's model (e.g. "HDMI-1") doesn't match in the
    /// monitors config, this config is used.
    #[serde(default)]
    pub(crate) default_monitor: MonitorConfig,
}

/// Reads the config file.
///
/// The config file is based on the XDG Base Directory Specification. See [`crate::config::Config`]
/// for the config schema.
pub(crate) fn read_config() -> Config {
    let xdg_dirs =
        xdg::BaseDirectories::with_prefix("jiji").expect("Failed to read the config dir");
    let config_path = xdg_dirs.find_config_file("config.json");
    if let Some(pth) = config_path {
        let config_str = std::fs::read_to_string(pth).expect("Failed to read the config.json");
        serde_json::from_str(&config_str).expect("Failed to parse the config.json")
    } else {
        serde_json::from_str("{}").expect("Failed to create the default config")
    }
}

/// Finds the MonitorConfig for the monitor.
pub(crate) fn find_monitor_config<'a>(
    config: &'a Config,
    monitor: &gtk::gdk::Monitor,
) -> &'a MonitorConfig {
    if let Some(model) = monitor.model() {
        for (ref name, ref mc) in &config.monitors {
            if name.as_str() == model.as_str() {
                return mc;
            }
        }
    }
    &config.default_monitor
}
