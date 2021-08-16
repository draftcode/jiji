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

//! # jiji library
//!
//! When reading Jiji's source code, you would probably want to see the following modules first:
//!
//! * `config`: The config.json data structures.
//! * `bar`: The implementation of Bars. A bar contains multiple modules, and a module represents
//!   one component in a bar.
//! * `module`: The interface of modules. Each module needs to implement these interfaces.
//! * `plugins`: The implementation of the modules. Each plugin provides a way to create modules.
//!
//! When the application starts, it reads the config file and initializes the plugins. During this
//! process, a plugin can provide ModuleFactories. A ModuleFactory is basically a function that
//! takes a (JSON) config and then return a module. A module is a component in a bar. You can think
//! a module as one GTK widget in the bar basically. Then, based on the config file's content, it
//! creates a module by using ModuleFactories, show the modules in the bar.
//!
//! ## Adding a new module
//!
//! What you need to do is:
//!
//! 1. Write a Module and a ModuleFactory.
//! 2. Add them through the `plugins` (Rust) module's `PLUGINS`.
//! 3. Modify your `config.json` to instantiate the module in a bar.
//!
//! A very simple example is the `text` plugin. This takes a string to show as a config then shows
//! it in the bar. As you can see, you will get a `gtk::Box` from the caller, and you can put any
//! GTK widgets in it.
//!
//! Basically, what you need to do is writing a function that takes a JSON config and a `gtk::Box`
//! and populates the GTK widget container. There is a utility for wrapping that function into a
//! Module in the `module_base` module. You can find a usage in the `button` plugin.
//!
//! You might want to interact with other systems. For example, the `i3` plugin interacts with i3wm
//! to get and modify the workspace state. For the interaction with other systems, it's better to
//! add an abstraction layer by making a GObject. See the `i3` (Rust) module. It defines a GObject
//! that represents the i3wm workspace state and provides a way to operate on them. Since changes
//! to GObject properties can trigger GTK widget changes, you can write a widget that reacts to
//! other system's state changes easily.

#[macro_use]
extern crate lazy_static;

pub(crate) mod bar;
pub(crate) mod config;
pub(crate) mod i3;
pub(crate) mod module;
pub(crate) mod module_base;
pub(crate) mod plugins;
pub(crate) mod pulseaudio;

use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

/// Jiji holds the whole application data.
///
/// In order to use glib's weak references for callbacks, we need to own the data somewhere. This
/// struct is the root of such structures.
struct Jiji {
    config: config::Config,
    module_factories: HashMap<String, Box<dyn module::ModuleFactory>>,
    bars: Vec<bar::Bar>,
}

impl Jiji {
    /// Callback for new monitors.
    fn handle_monitor_added(&mut self, app: &gtk::Application, monitor: &gtk::gdk::Monitor) {
        let bar = bar::Bar::new(
            config::find_monitor_config(&self.config, monitor),
            &self.module_factories,
            monitor,
        );
        bar.build_ui(app);
        self.bars.push(bar);
    }

    /// Sets up the CSS for the bars.
    fn setup_css(&self, screen: &gtk::gdk::Screen) {
        if !self.config.disable_default_css {
            let provider = gtk::CssProvider::new();
            provider
                .load_from_data(include_bytes!("default_style.css"))
                .expect("Failed to load CSS");
            gtk::StyleContext::add_provider_for_screen(
                screen,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        if !self.config.css_path.is_empty() {
            let mut p = PathBuf::from(&self.config.css_path);
            if p.is_relative() {
                let xdg_dirs =
                    xdg::BaseDirectories::with_prefix("jiji").expect("Failed to read the CSS");
                p = xdg_dirs.get_config_home().join(p);
            }
            let provider = gtk::CssProvider::new();
            provider
                .load_from_path(p.to_str().expect("Failed to read the CSS"))
                .expect("Failed to load CSS");
            gtk::StyleContext::add_provider_for_screen(
                screen,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_USER,
            );
        }
    }
}

/// Sets up the bars.
fn handle_activate(app: &gtk::Application) {
    let config = config::read_config();
    let module_factories = module::make_module_factories(&config.plugins);
    let mut jiji = Jiji {
        config,
        module_factories,
        bars: vec![],
    };
    let display = gtk::gdk::Display::default().expect("Failed to get the default Display");

    jiji.setup_css(&display.default_screen());

    for i in 0..display.n_monitors() {
        let monitor = display.monitor(i).expect("Failed to get a monitor");
        jiji.handle_monitor_added(app, &monitor);
    }
    let jiji = RefCell::new(jiji);
    display.connect_monitor_added(glib::clone!(@weak app => move |_, monitor| {
        jiji.borrow_mut().handle_monitor_added(&app, monitor);
    }));
}

/// Runs the application.
pub fn run() -> i32 {
    let app = gtk::Application::new(Some("org.example.HelloWorld"), Default::default());
    app.connect_activate(|app| handle_activate(app));
    app.run()
}
