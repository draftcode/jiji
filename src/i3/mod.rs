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

#[derive(Clone, Debug)]
pub struct WorkspaceState {
    pub num: i32,
    pub name: String,
    pub visible: bool,
    pub focused: bool,
    pub urgent: bool,
}

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "Workspaces")]
pub struct Workspaces(HashMap<String, Vec<WorkspaceState>>);

gtk::glib::wrapper! {
    pub struct I3State(ObjectSubclass<imp::I3State>);
}

impl I3State {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an I3State")
    }

    pub fn workspaces(&self) -> HashMap<String, Vec<WorkspaceState>> {
        self.property("workspaces")
            .unwrap()
            .get::<Workspaces>()
            .unwrap()
            .0
    }

    pub fn switch_workspace(&self, num: i32) {
        let self_ = imp::I3State::from_instance(self);
        if let Some(ref mut connection) = self_.connection.borrow_mut().as_mut() {
            connection
                .run_command(&format!("workspace number {}", num))
                .expect("Failed to switch workspaces");
        }
    }
}

mod imp {
    use super::{WorkspaceState, Workspaces};
    use glib::{ParamFlags, ParamSpec};
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::thread;

    #[derive(Debug, Default)]
    pub struct I3State {
        pub(crate) connection: RefCell<Option<i3ipc::I3Connection>>,
        pub(crate) workspaces: RefCell<Workspaces>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for I3State {
        const NAME: &'static str = "I3State";
        type Type = super::I3State;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for I3State {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![WORKSPACES.clone()]);
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "workspaces" => self.workspaces.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.connection.replace(Some(
                i3ipc::I3Connection::connect().expect("Failed to connect i3"),
            ));

            let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            receiver.attach(
                None,
                glib::clone!(@weak obj => @default-return Continue(false), move |ws| {
                    I3State::from_instance(&obj).workspaces.replace(ws);
                    obj.notify_by_pspec(&WORKSPACES);
                    Continue(true)
                }),
            );
            thread::spawn(glib::clone!(@strong sender => move || {
                let mut connection = i3ipc::I3Connection::connect().expect("Failed to connect i3");
                sender.send(get_workspaces(&mut connection)).expect("Failed to send new workspaces");

                let mut listener = i3ipc::I3EventListener::connect().expect("Failed to connect i3");
                listener.subscribe(&[i3ipc::Subscription::Workspace]).expect("Failed to subscribe to i3");
                for event in listener.listen() {
                    match event.expect("Failed to parse an i3 event") {
                        i3ipc::event::Event::WorkspaceEvent(_) => {
                            sender.send(get_workspaces(&mut connection)).expect("Failed to send new workspaces");
                        },
                        _ => unreachable!()
                    }
                }
            }));
        }
    }

    fn get_workspaces(connection: &mut i3ipc::I3Connection) -> Workspaces {
        let i3wses = connection
            .get_workspaces()
            .expect("Failed to get workspaces")
            .workspaces;
        let mut wses = HashMap::new();
        for ref i3ws in i3wses {
            if !wses.contains_key(i3ws.output.as_str()) {
                wses.insert(i3ws.output.to_owned(), vec![]);
            };
            let v = wses.get_mut(i3ws.output.as_str()).unwrap();
            v.push(WorkspaceState {
                num: i3ws.num,
                name: i3ws.name.to_owned(),
                visible: i3ws.visible,
                focused: i3ws.focused,
                urgent: i3ws.urgent,
            });
        }
        for (_, ref mut wss) in &mut wses {
            wss.sort_by_key(|ref ws| ws.num);
        }
        Workspaces(wses)
    }

    lazy_static! {
        static ref WORKSPACES: ParamSpec = ParamSpec::new_boxed(
            "workspaces",
            "workspaces",
            "workspaces",
            Workspaces::static_type(),
            ParamFlags::READABLE,
        );
    }
}
