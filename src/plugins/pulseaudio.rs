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
use crate::pulseaudio::PulseAudioState;
use gtk::glib;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::rc::Rc;

fn default_source_volume_toggle_module(
    state: Rc<PulseAudioState>,
) -> FnModFactory<serde_json::Value> {
    FnModFactory::new(
        "pulseaudio-default-source-volume-toggle",
        Box::new(JSONConfigFactory::default()),
        Box::new(move |_, container: &gtk::Box| {
            let button = gtk::Button::new();
            button.set_relief(gtk::ReliefStyle::None);
            button
                .style_context()
                .add_class("default-source-volume-toggle");

            let state = state.clone();
            button.connect_button_release_event(
                    glib::clone!(@weak state => @default-return Inhibit(false), move |_, e| {
                        if e.button() == gtk::gdk::BUTTON_PRIMARY {
                            state.default_source().map(|s| s.toggle_mute());
                        } else if e.button() == gtk::gdk::BUTTON_SECONDARY {
                            Command::new("pavucontrol").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
                        }
                        return Inhibit(true);
                    }),
                );

            state.connect_notify_local(
                None,
                glib::clone!(@weak button => move |state, _| {
                    if let Some(source) = state.default_source() {
                        button.set_sensitive(true);
                        let mut s = source.volume.max().print();
                        if source.mute {
                            s += " (muted)"
                        }
                        button.set_label(&s);
                    } else {
                        button.set_sensitive(false);
                    }
                }),
            );

            container.add(&button);
        }),
    )
}

fn default_source_volume_module(state: Rc<PulseAudioState>) -> FnModFactory<serde_json::Value> {
    FnModFactory::new(
        "pulseaudio-default-source-volume",
        Box::new(JSONConfigFactory::default()),
        Box::new(move |_, container: &gtk::Box| {
            let adjustment = state.default_source().map(|s| s.adjustment());
            let scale = gtk::Scale::new(gtk::Orientation::Horizontal, adjustment.as_ref());
            scale.set_width_request(100);
            scale.set_draw_value(false);
            scale
                .style_context()
                .add_class("pulseaudio-default-source-volume");
            container.add(&scale);

            state.connect_notify_local(
                None,
                glib::clone!(@weak scale => move |state, _| {
                    if let Some(adjustment) = state.default_source().map(|s| s.adjustment()) {
                        scale.set_adjustment(&adjustment);
                    }
                }),
            );
            scale.connect_scroll_event(move |_, _| gtk::Inhibit(true));
        }),
    )
}

#[derive(Serialize, Deserialize, Default)]
struct DefaultSourceSelectorConfig {
    /// Nicknames for sources.
    ///
    /// By default, the widget shows the source descriptions. However, this might be too long for
    /// the menu bar. This allows mapping from a name (e.g. "alsa_input.usb-foo-bar.analog-stereo")
    /// to a name of your choice.
    #[serde(default)]
    nicknames: HashMap<String, String>,
}

fn default_source_selector_module(
    state: Rc<PulseAudioState>,
) -> FnModFactory<DefaultSourceSelectorConfig> {
    FnModFactory::new(
        "pulseaudio-default-source-selector",
        Box::new(JSONConfigFactory::default()),
        Box::new(
            move |config: &Rc<DefaultSourceSelectorConfig>, container: &gtk::Box| {
                let button = gtk::Button::new();
                button.set_relief(gtk::ReliefStyle::None);
                container.add(&button);

                {
                    let state = state.clone();
                    let config = config.clone();
                    button.connect_button_release_event(glib::clone!(@weak button, @weak state => @default-return Inhibit(false), move |_, e| {
                        if e.button() == gtk::gdk::BUTTON_PRIMARY {
                            let menu = gtk::Menu::new();
                            for (_, ref source) in state.sources() {
                                if source.is_monitor {
                                    continue
                                }
                                let name = source.name.to_string();
                                let mut shown_name = &source.description;
                                for (ref k, ref nickname) in &config.nicknames {
                                    if k.as_str() == name.as_str() {
                                        shown_name = nickname;
                                    }
                                }

                                let item = gtk::MenuItem::with_label(shown_name);
                                item.connect_activate(glib::clone!(@weak state => move |_| {
                                    state.set_default_source(&name);
                                }));
                                menu.append(&item);
                            }
                            menu.show_all();
                            menu.popup_at_widget(&button, gtk::gdk::Gravity::South, gtk::gdk::Gravity::North, None);
                            return Inhibit(true);
                        }
                        Inhibit(false)
                    }));
                }

                {
                    let config = config.clone();
                    state.connect_notify_local(
                        None,
                        glib::clone!(@weak button => move |state, _| {
                            if let Some(source) = state.default_source() {
                                if let Some(ref nickname) = config.nicknames.get(&source.name) {
                                    button.set_label(nickname);
                                    return;
                                }
                                button.set_label(&source.description);
                            }
                        }),
                    );
                }
            },
        ),
    )
}

fn default_sink_volume_toggle_module(
    state: Rc<PulseAudioState>,
) -> FnModFactory<serde_json::Value> {
    FnModFactory::new(
        "pulseaudio-default-sink-volume-toggle",
        Box::new(JSONConfigFactory::default()),
        Box::new(move |_, container: &gtk::Box| {
            let button = gtk::Button::new();
            button.set_relief(gtk::ReliefStyle::None);
            container.add(&button);

            let state = state.clone();
            button.connect_button_release_event(
                glib::clone!(@weak state => @default-return Inhibit(false), move |_, e| {
                    if e.button() == gtk::gdk::BUTTON_PRIMARY {
                        state.default_sink().map(|s| s.toggle_mute());
                    } else if e.button() == gtk::gdk::BUTTON_SECONDARY {
                        Command::new("pavucontrol").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
                    }
                    return Inhibit(true);
                }),
            );

            state.connect_notify_local(
                None,
                glib::clone!(@weak button => move |state, _| {
                    if let Some(sink) = state.default_sink() {
                        button.set_sensitive(true);
                        let mut s = sink.volume.max().print();
                        if sink.mute {
                            s += " (muted)"
                        }
                        button.set_label(&s);
                    } else {
                        button.set_sensitive(false);
                    }
                }),
            );
        }),
    )
}

fn default_sink_volume_module(state: Rc<PulseAudioState>) -> FnModFactory<serde_json::Value> {
    FnModFactory::new(
        "pulseaudio-default-sink-volume",
        Box::new(JSONConfigFactory::default()),
        Box::new(move |_, container: &gtk::Box| {
            let adjustment = state.default_sink().map(|s| s.adjustment());
            let scale = gtk::Scale::new(gtk::Orientation::Horizontal, adjustment.as_ref());
            scale.set_width_request(100);
            scale.set_draw_value(false);
            container.add(&scale);

            state.connect_notify_local(
                None,
                glib::clone!(@weak scale => move |state, _| {
                    if let Some(adjustment) = state.default_sink().map(|s| s.adjustment()) {
                        scale.set_adjustment(&adjustment);
                    }
                }),
            );
            scale.connect_scroll_event(move |_, _| gtk::Inhibit(true));
        }),
    )
}

#[derive(Serialize, Deserialize, Default)]
struct DefaultSinkSelectorConfig {
    /// Nicknames for sinks.
    ///
    /// By default, the widget shows the sink descriptions. However, this might be too long for
    /// the menu bar. This allows mapping from a name (e.g.
    /// "alsa_output.usb-foo-bar.analog-stereo") to a name of your choice.
    #[serde(default)]
    nicknames: HashMap<String, String>,
}

fn default_sink_selector_module(
    state: Rc<PulseAudioState>,
) -> FnModFactory<DefaultSinkSelectorConfig> {
    FnModFactory::new(
        "pulseaudio-default-sink-selector",
        Box::new(JSONConfigFactory::default()),
        Box::new(
            move |config: &Rc<DefaultSinkSelectorConfig>, container: &gtk::Box| {
                let button = gtk::Button::new();
                button.set_relief(gtk::ReliefStyle::None);
                container.add(&button);

                {
                    let state = state.clone();
                    let config = config.clone();
                    button.connect_button_release_event(glib::clone!(@weak button => @default-return Inhibit(false), move |_, e| {
                        if e.button() == gtk::gdk::BUTTON_PRIMARY {
                            let menu = gtk::Menu::new();
                            for (_, ref sink) in state.sinks() {
                                let name = sink.name.to_string();
                                let mut shown_name = &sink.description;
                                for (ref k, ref nickname) in &config.nicknames {
                                    if k.as_str() == name.as_str() {
                                        shown_name = nickname;
                                    }
                                }

                                let item = gtk::MenuItem::with_label(shown_name);
                                item.connect_activate(glib::clone!(@weak state => move |_| {
                                    state.set_default_sink(&name);
                                }));
                                menu.append(&item);
                            }
                            menu.show_all();
                            menu.popup_at_widget(&button, gtk::gdk::Gravity::South, gtk::gdk::Gravity::North, None);
                            return Inhibit(true);
                        }
                        Inhibit(false)
                    }));
                }

                {
                    let config = config.clone();
                    state.connect_notify_local(
                        None,
                        glib::clone!(@weak button => move |state, _| {
                            if let Some(sink) = state.default_sink() {
                                if let Some(ref nickname) = config.nicknames.get(&sink.name) {
                                    button.set_label(nickname);
                                    return;
                                }
                                button.set_label(&sink.description);
                            }
                        }),
                    );
                }
            },
        ),
    )
}

pub(crate) fn make_module_factories(
    _config: &serde_json::Value,
) -> Vec<Box<dyn crate::module::ModuleFactory>> {
    let state = Rc::new(PulseAudioState::new());

    vec![
        Box::new(default_source_volume_toggle_module(state.clone())),
        Box::new(default_source_volume_module(state.clone())),
        Box::new(default_source_selector_module(state.clone())),
        Box::new(default_sink_volume_toggle_module(state.clone())),
        Box::new(default_sink_volume_module(state.clone())),
        Box::new(default_sink_selector_module(state.clone())),
    ]
}
