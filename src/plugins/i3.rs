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

use crate::i3::I3State;
use gtk::glib;
use gtk::prelude::*;
use std::rc::Rc;

struct WorkspacesModule {
    model: String,
    state: Rc<I3State>,
}

impl crate::module::Module for WorkspacesModule {
    fn build_ui(&self, container: &gtk::Box) {
        let model = self.model.clone();
        self.state.connect_notify_local(
            Some("workspaces"),
            glib::clone!(@weak container => move |state, _| {
                for ref child in container.children() {
                    container.remove(child);
                }
                if let Some(wss) = state.workspaces().get(model.as_str()) {
                    for ref ws in wss {
                        let button = gtk::Button::with_label(&ws.name);
                        button.set_relief(gtk::ReliefStyle::None);
                        let sc = button.style_context();
                        sc.add_class("workspace");
                        sc.add_class(&format!("workspace-name-{}", ws.name));
                        sc.add_class(&format!("workspace-num-{}", ws.num));
                        if ws.urgent {
                            sc.add_class("workspace-urgent");
                        }
                        if ws.focused {
                            sc.add_class("workspace-focused");
                        }
                        let ws_num = ws.num;
                        button.connect_clicked(glib::clone!(@weak state => move |_| {
                            state.switch_workspace(ws_num);
                        }));
                        container.add(&button);
                    }
                }
                container.show_all();
            }),
        );
    }
}

struct WorkspacesModuleFactory {
    state: Rc<I3State>,
}

impl crate::module::ModuleFactory for WorkspacesModuleFactory {
    fn name(&self) -> &str {
        "i3-workspaces"
    }

    fn create(
        &self,
        _config: &serde_json::Value,
        monitor: &gtk::gdk::Monitor,
    ) -> Box<dyn crate::module::Module> {
        Box::new(WorkspacesModule {
            model: monitor.model().map(|v| v.to_string()).unwrap_or_default(),
            state: self.state.clone(),
        })
    }
}

pub(crate) fn make_module_factories(
    _config: &serde_json::Value,
) -> Vec<Box<dyn crate::module::ModuleFactory>> {
    let state = Rc::new(I3State::new());
    vec![Box::new(WorkspacesModuleFactory { state })]
}
