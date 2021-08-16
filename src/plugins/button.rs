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

use crate::module_base::{FnModFactory, JSONConfigFactory};
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::rc::Rc;

#[derive(Serialize, Deserialize, Default)]
struct ButtonConfig {
    text: String,
    command: Vec<String>,
}

fn button_module() -> FnModFactory<ButtonConfig> {
    FnModFactory::new(
        "button",
        Box::new(JSONConfigFactory::default()),
        Box::new(move |config: &Rc<ButtonConfig>, container: &gtk::Box| {
            let button = gtk::Button::with_label(&config.text);
            button.set_relief(gtk::ReliefStyle::None);
            container.add(&button);

            let command = config.command.clone();
            button.connect_button_release_event(move |_, e| {
                if e.button() == gtk::gdk::BUTTON_PRIMARY {
                    Command::new(&command[0])
                        .args(&command[1..])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .unwrap();
                    return Inhibit(true);
                }
                Inhibit(false)
            });
        }),
    )
}

pub(crate) fn make_module_factories(
    _config: &serde_json::Value,
) -> Vec<Box<dyn crate::module::ModuleFactory>> {
    vec![Box::new(button_module())]
}
