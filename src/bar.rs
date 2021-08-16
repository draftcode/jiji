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
use std::collections::HashMap;

pub(crate) struct Bar {
    left_modules: Vec<Box<dyn crate::module::Module>>,
    center_modules: Vec<Box<dyn crate::module::Module>>,
    right_modules: Vec<Box<dyn crate::module::Module>>,
    name: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Bar {
    pub(crate) fn new(
        config: &crate::config::MonitorConfig,
        module_factories: &HashMap<String, Box<dyn crate::module::ModuleFactory>>,
        monitor: &gtk::gdk::Monitor,
    ) -> Bar {
        let left_modules = Bar::init_modules(&config.left_modules, module_factories, monitor);
        let center_modules = Bar::init_modules(&config.center_modules, module_factories, monitor);
        let right_modules = Bar::init_modules(&config.right_modules, module_factories, monitor);
        let geom = monitor.geometry();
        return Bar {
            left_modules,
            center_modules,
            right_modules,
            name: monitor.model().map(|v| v.to_string()).unwrap_or("".to_string()),
            x: geom.x,
            y: geom.y,
            width: geom.width,
            height: config.height.unwrap_or(30),
        };
    }

    pub(crate) fn build_ui(&self, app: &gtk::Application) {
        let win = gtk::ApplicationWindow::builder()
            .application(app)
            .type_hint(gtk::gdk::WindowTypeHint::Dock)
            .build();
        win.move_(self.x, self.y);
        win.resize(self.width, self.height);
        win.set_widget_name(&format!("root-{}", self.name));
        win.style_context().add_class("root");

        let win_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        win_box.set_widget_name(&format!("bar-{}", self.name));
        win_box.style_context().add_class("bar");
        win_box.pack_start(&Bar::init_box("left-modules", &self.left_modules), false, false, 0);
        win_box.set_center_widget(Some(&Bar::init_box("center-modules", &self.center_modules)));
        win_box.pack_end(&Bar::init_box("right-modules", &self.right_modules), false, false, 0);
        win.add(&win_box);

        win.show_all();
    }

    fn init_box(class: &str, modules: &Vec<Box<dyn crate::module::Module>>) -> gtk::Box {
        let b = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        b.style_context().add_class(class);
        for ref module in modules {
            let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            module.build_ui(&container);
            b.pack_start(&container, false, false, 0);
        }
        b
    }

    fn init_modules(
        configs: &Vec<crate::config::ModuleConfig>,
        module_factories: &HashMap<String, Box<dyn crate::module::ModuleFactory>>,
        monitor: &gtk::gdk::Monitor,
    ) -> Vec<Box<dyn crate::module::Module>> {
        let mut modules = vec![];
        for ref config in configs {
            modules.push(
                module_factories
                    .get(config.name.as_str())
                    .expect("Failed to find a module")
                    .create(&config.config, monitor),
            );
        }
        modules
    }
}
