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

use std::marker::PhantomData;
use std::rc::Rc;

pub struct FnModFactory<Config> {
    name: &'static str,
    config_factory: Box<dyn ConfigFactory<T = Config>>,
    build_ui_fn: Rc<Box<dyn Fn(&Rc<Config>, &gtk::Box)>>,
}

impl<Config> FnModFactory<Config> {
    pub fn new(
        name: &'static str,
        config_factory: Box<dyn ConfigFactory<T = Config>>,
        func: Box<dyn Fn(&Rc<Config>, &gtk::Box)>,
    ) -> FnModFactory<Config> {
        FnModFactory {
            name,
            config_factory,
            build_ui_fn: Rc::new(func),
        }
    }
}

impl<Config: 'static> crate::module::ModuleFactory for FnModFactory<Config> {
    fn name(&self) -> &str {
        self.name
    }

    fn create(
        &self,
        json_config: &serde_json::Value,
        monitor: &gtk::gdk::Monitor,
    ) -> Box<dyn crate::module::Module> {
        let config = self
            .config_factory
            .from_json(json_config, monitor)
            .expect("Failed to parse the config");

        Box::new(FnMod {
            config: Rc::new(config),
            build_ui_fn: self.build_ui_fn.clone(),
        })
    }
}

struct FnMod<Config> {
    config: Rc<Config>,
    build_ui_fn: Rc<Box<dyn Fn(&Rc<Config>, &gtk::Box)>>,
}

impl<Config> crate::module::Module for FnMod<Config> {
    fn build_ui(&self, container: &gtk::Box) {
        (self.build_ui_fn)(&self.config, container);
    }
}

pub trait ConfigFactory {
    type T;

    fn from_json<'a>(
        &self,
        json_config: &serde_json::Value,
        monitor: &gtk::gdk::Monitor,
    ) -> Result<Self::T, ()>;
}

#[derive(Default)]
pub struct JSONConfigFactory<Config: serde::de::DeserializeOwned + Default> {
    _marker: PhantomData<Config>,
}

impl<Config: serde::de::DeserializeOwned + Default> ConfigFactory for JSONConfigFactory<Config> {
    type T = Config;

    fn from_json<'a>(
        &self,
        json_config: &serde_json::Value,
        _monitor: &gtk::gdk::Monitor,
    ) -> Result<Self::T, ()> {
        if json_config.is_null() {
            Ok(Config::default())
        } else {
            serde_json::from_str(&json_config.to_string()).map_err(|_| ())
        }
    }
}

