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

use gtk::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TextModuleConfig {
    text: String,
}

struct TextModule {
    config: TextModuleConfig,
}

impl crate::module::Module for TextModule {
    fn build_ui(&self, container: &gtk::Box) {
        let label = gtk::Label::builder().label(&self.config.text).build();
        container.pack_start(&label, false, false, 0);
    }
}

struct TextModuleFactory {}

impl crate::module::ModuleFactory for TextModuleFactory {
    fn name(&self) -> &str {
        "text"
    }

    fn create(
        &self,
        config: &serde_json::Value,
        _monitor: &gtk::gdk::Monitor,
    ) -> Box<dyn crate::module::Module> {
        let config = serde_json::from_str(&config.to_string()).expect("Failed to parse the config");
        Box::new(TextModule { config })
    }
}

pub(crate) fn make_module_factories(
    _config: &serde_json::Value,
) -> Vec<Box<dyn crate::module::ModuleFactory>> {
    vec![Box::new(TextModuleFactory {})]
}
