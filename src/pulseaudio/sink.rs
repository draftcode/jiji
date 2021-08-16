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
use pulse::context::introspect::SinkInfo;
use pulse::context::Context;
use pulse::volume::{ChannelVolumes, Volume};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct SinkState {
    pa_context: Rc<RefCell<Option<Context>>>,
    pub name: String,
    pub description: String,
    pub mute: bool,
    pub volume: ChannelVolumes,
}

impl SinkState {
    /// Creates a new SinkState.
    pub fn new(pa_context: Rc<RefCell<Option<Context>>>, si: &SinkInfo) -> SinkState {
        SinkState {
            pa_context,
            name: si.name.as_ref().map(|v| v.to_string()).unwrap_or_default(),
            description: si
                .description
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            mute: si.mute,
            volume: si.volume,
        }
    }

    /// Toggles the mute state.
    pub fn toggle_mute(&self) {
        self.pa_context
            .borrow_mut()
            .as_mut()
            .unwrap()
            .introspect()
            .set_sink_mute_by_name(&self.name, !self.mute, None);
    }

    /// Creates a connected adjustment.
    pub fn adjustment(&self) -> gtk::Adjustment {
        let obj = gtk::Adjustment::new(
            // From pa_volume_snprint_verbose.
            (self.volume.max().0 as f64) * 100.0 / (Volume::NORMAL.0 as f64) + 0.5,
            0.0,
            100.0,
            0.0,
            0.0,
            0.0,
        );
        let name = self.name.clone();
        let pa_context = self.pa_context.clone();
        let cv = self.volume.clone();
        obj.connect_value_changed(move |obj| {
            let cv = super::util::percentage_to_volume(obj.value(), cv);
            pa_context
                .borrow_mut()
                .as_mut()
                .unwrap()
                .introspect()
                .set_sink_volume_by_name(&name, &cv, None);
        });
        obj
    }
}
