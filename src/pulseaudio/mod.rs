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

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::collections::HashMap;

pub mod sink;
pub mod source;
pub mod util;

#[derive(Clone, Default, glib::GBoxed)]
#[gboxed(type_name = "Sinks")]
pub struct Sinks(HashMap<u32, sink::SinkState>);

#[derive(Clone, Default, glib::GBoxed)]
#[gboxed(type_name = "Sources")]
pub struct Sources(HashMap<u32, source::SourceState>);

gtk::glib::wrapper! {
    pub struct PulseAudioState(ObjectSubclass<imp::PulseAudioState>);
}

impl PulseAudioState {
    /// Makes a new PulseAudioState.
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an PulseAudioState")
    }

    /// Returns the default sink name.
    pub fn default_sink_name(&self) -> String {
        self.property("defaultSink")
            .unwrap()
            .get::<String>()
            .unwrap()
    }

    /// Returns the default source name.
    pub fn default_source_name(&self) -> String {
        self.property("defaultSource")
            .unwrap()
            .get::<String>()
            .unwrap()
    }

    /// Returns the default sink state.
    pub fn default_sink(&self) -> Option<sink::SinkState> {
        let name = self.default_sink_name();
        for (_, ref state) in self.sinks() {
            if state.name == name {
                return Some(state.clone());
            }
        }
        None
    }

    /// Returns the default source state.
    pub fn default_source(&self) -> Option<source::SourceState> {
        let name = self.default_source_name();
        for (_, ref state) in self.sources() {
            if state.name == name {
                return Some(state.clone());
            }
        }
        None
    }

    /// Sets the default sink.
    pub fn set_default_sink(&self, name: &str) {
        let self_ = imp::PulseAudioState::from_instance(self);
        self_.pa_context
            .borrow_mut()
            .as_mut()
            .unwrap()
            .set_default_sink(name, move |_| {});
    }

    /// Sets the default source.
    pub fn set_default_source(&self, name: &str) {
        let self_ = imp::PulseAudioState::from_instance(self);
        self_.pa_context
            .borrow_mut()
            .as_mut()
            .unwrap()
            .set_default_source(name, move |_| {});
    }

    /// Returns all sink states.
    pub fn sinks(&self) -> HashMap<u32, sink::SinkState> {
        self.property("sinks").unwrap().get::<Sinks>().unwrap().0
    }

    /// Returns all sink sources.
    pub fn sources(&self) -> HashMap<u32, source::SourceState> {
        self.property("sources")
            .unwrap()
            .get::<Sources>()
            .unwrap()
            .0
    }
}

mod imp {
    use super::{sink::SinkState, source::SourceState, Sinks, Sources};
    use glib::{ParamFlags, ParamSpec};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;
    use pulse::callbacks::ListResult;
    use pulse::context::introspect::{ServerInfo, SinkInfo, SourceInfo};
    use pulse::context::subscribe::{Facility, InterestMaskSet, Operation};
    use pulse::context::{Context, FlagSet};
    use pulse_glib::Mainloop;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    pub struct PulseAudioState {
        pub(crate) pa_mainloop: RefCell<Option<Mainloop>>,
        pub(crate) pa_context: Rc<RefCell<Option<Context>>>,

        pub(crate) default_sink: RefCell<String>,
        pub(crate) default_source: RefCell<String>,
        pub(crate) sinks: RefCell<Sinks>,
        pub(crate) sources: RefCell<Sources>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PulseAudioState {
        const NAME: &'static str = "PulseAudioState";
        type Type = super::PulseAudioState;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for PulseAudioState {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    DEFAULT_SINK.clone(),
                    DEFAULT_SOURCE.clone(),
                    SINKS.clone(),
                    SOURCES.clone(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "defaultSink" => self.default_sink.borrow().to_value(),
                "defaultSource" => self.default_source.borrow().to_value(),
                "sinks" => self.sinks.borrow().to_value(),
                "sources" => self.sources.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.pa_mainloop.replace(Some(
                Mainloop::new(None).expect("Failed to create a PulseAudio main loop"),
            ));
            let mut pa_context = Context::new(self.pa_mainloop.borrow().as_ref().unwrap(), "jiji")
                .expect("Failed to create PulseAudio context");

            pa_context.set_state_callback(Some(Box::new(glib::clone!(@weak obj => move || {
                PulseAudioState::from_instance(&obj).on_state_change(&obj);
            }))));
            pa_context.set_subscribe_callback(Some(Box::new(
                glib::clone!(@weak obj => move |facility, operation, index| {
                    PulseAudioState::from_instance(&obj).on_event(&obj, facility, operation, index);
                }),
            )));
            pa_context
                .connect(None, FlagSet::NOFLAGS, None)
                .expect("Failed to connect to the PulseAudio server");
            self.pa_context.replace(Some(pa_context));
        }
    }

    impl PulseAudioState {
        fn on_state_change(&self, obj: &super::PulseAudioState) {
            if self.pa_context.borrow().is_none() {
                // Not initialized yet. This happens because pa_context.connect call above calls
                // this callback inline.
                return;
            }
            if self.pa_context.borrow().as_ref().unwrap().get_state()
                != pulse::context::State::Ready
            {
                return;
            }
            if let Some(ref mut pa_context) = self.pa_context.borrow_mut().as_mut() {
                pa_context.subscribe(
                    InterestMaskSet::SINK | InterestMaskSet::SOURCE | InterestMaskSet::SERVER,
                    move |e| {
                        assert!(e, "Failed to subscribe to PulseAudio events");
                    },
                );
                pa_context
                    .introspect()
                    .get_server_info(glib::clone!(@weak obj => move |si| {
                        PulseAudioState::from_instance(&obj).on_server_info(&obj, si);
                    }));
                pa_context
                    .introspect()
                    .get_sink_info_list(glib::clone!(@weak obj => move |res| {
                        match res {
                            ListResult::Item(si) => {PulseAudioState::from_instance(&obj).on_sink_info(si)}
                            ListResult::End => {obj.notify_by_pspec(&SINKS)}
                            _ => (),
                        }
                    }));
                pa_context.introspect().get_source_info_list(
                    glib::clone!(@weak obj => move |res| {
                        match res {
                            ListResult::Item(si) => {PulseAudioState::from_instance(&obj).on_source_info(si)}
                            ListResult::End => {obj.notify_by_pspec(&SOURCES)}
                            _ => (),
                        }
                    }),
                );
            }
        }
        fn on_event(
            &self,
            obj: &super::PulseAudioState,
            facility: Option<Facility>,
            operation: Option<Operation>,
            index: u32,
        ) {
            match facility {
                Some(Facility::Sink) => {
                    match operation {
                        Some(Operation::Removed) => {
                            self.sinks.borrow_mut().0.remove(&index);
                            obj.notify_by_pspec(&SINKS);
                        }
                        Some(Operation::Changed) | Some(Operation::New) => {
                            self.pa_context
                                .borrow_mut()
                                .as_mut()
                                .unwrap()
                                .introspect()
                                .get_sink_info_by_index(index, glib::clone!(@weak obj => move |res| {
                                    match res {
                                        ListResult::Item(si) => {
                                            PulseAudioState::from_instance(&obj).on_sink_info(si)
                                        }
                                        ListResult::End => {obj.notify_by_pspec(&SINKS)}
                                        _ => (),
                                    }
                                }));
                        }
                        _ => (),
                    }
                }
                Some(Facility::Source) => {
                    match operation {
                        Some(Operation::Removed) => {
                            self.sources.borrow_mut().0.remove(&index);
                            obj.notify_by_pspec(&SOURCES);
                        }
                        Some(Operation::Changed) | Some(Operation::New) => {
                            self.pa_context
                                .borrow_mut()
                                .as_mut()
                                .unwrap()
                                .introspect()
                                .get_source_info_by_index(index, glib::clone!(@weak obj => move |res| {
                                    match res {
                                        ListResult::Item(si) => {
                                            PulseAudioState::from_instance(&obj).on_source_info(si)
                                        }
                                        ListResult::End => {obj.notify_by_pspec(&SOURCES)}
                                        _ => (),
                                    }
                                }));
                        }
                        _ => (),
                    }
                }
                Some(Facility::Server) => match operation {
                    Some(Operation::Changed) => {
                        self.pa_context
                            .borrow_mut()
                            .as_mut()
                            .unwrap()
                            .introspect()
                            .get_server_info(glib::clone!(@weak obj => move |si| {
                                PulseAudioState::from_instance(&obj).on_server_info(&obj, si);
                            }));
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        fn on_server_info(&self, obj: &super::PulseAudioState, si: &ServerInfo) {
            let sink = si
                .default_sink_name
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default();
            self.default_sink.replace(sink);
            obj.notify_by_pspec(&DEFAULT_SINK);

            let source = si
                .default_source_name
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default();
            self.default_source.replace(source);
            obj.notify_by_pspec(&DEFAULT_SOURCE);
        }

        fn on_sink_info(&self, si: &SinkInfo) {
            self.sinks
                .borrow_mut()
                .0
                .insert(si.index, SinkState::new(self.pa_context.clone(), si));
        }

        fn on_source_info(&self, si: &SourceInfo) {
            self.sources
                .borrow_mut()
                .0
                .insert(si.index, SourceState::new(self.pa_context.clone(), si));
        }
    }

    lazy_static! {
        static ref DEFAULT_SINK: ParamSpec = ParamSpec::new_string(
            "defaultSink",
            "defaultSink",
            "defaultSink",
            Some(""),
            ParamFlags::READABLE,
        );
        static ref DEFAULT_SOURCE: ParamSpec = ParamSpec::new_string(
            "defaultSource",
            "defaultSource",
            "defaultSource",
            Some(""),
            ParamFlags::READABLE,
        );
        static ref SINKS: ParamSpec = ParamSpec::new_boxed(
            "sinks",
            "sinks",
            "sinks",
            Sinks::static_type(),
            ParamFlags::READABLE,
        );
        static ref SOURCES: ParamSpec = ParamSpec::new_boxed(
            "sources",
            "sources",
            "sources",
            Sources::static_type(),
            ParamFlags::READABLE,
        );
    }
}
