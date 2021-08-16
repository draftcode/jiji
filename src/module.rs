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

use std::collections::HashMap;

/// Module is one component in a bar.
pub(crate) trait Module {
    fn build_ui(&self, container: &gtk::Box);
}

pub(crate) trait ModuleFactory {
    fn name(&self) -> &str;
    fn create(&self, config: &serde_json::Value, monitor: &gtk::gdk::Monitor) -> Box<dyn Module>;
}

pub(crate) type Plugin = fn(&serde_json::Value) -> Vec<Box<dyn ModuleFactory>>;

pub(crate) fn make_module_factories(
    configs: &Vec<crate::config::PluginConfig>,
) -> HashMap<String, Box<dyn ModuleFactory>> {
    let mut ret = HashMap::new();
    for config in configs {
        let plugin = PLUGINS
            .get(config.name.as_str())
            .expect("Failed to find a plugin");
        for mf in plugin(&config.config) {
            ret.insert(mf.name().to_owned(), mf);
        }
    }
    ret
}

lazy_static! {
    static ref PLUGINS: HashMap<&'static str, Plugin> = {
        let mut m: HashMap<&'static str, Plugin> = HashMap::new();
        m.insert("button", crate::plugins::button::make_module_factories);
        m.insert("i3", crate::plugins::i3::make_module_factories);
        m.insert("pulseaudio", crate::plugins::pulseaudio::make_module_factories);
        m.insert("text", crate::plugins::text::make_module_factories);
        m
    };
}
