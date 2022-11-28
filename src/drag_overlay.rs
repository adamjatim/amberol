// SPDX-FileCopyrightText: 2022  Maximiliano Sandoval R <msandova@gnome.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use std::cell::RefCell;

    use adw::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, Default)]
    pub struct DragOverlay {
        pub overlay: gtk::Overlay,
        pub revealer: gtk::Revealer,
        pub status: adw::StatusPage,
        pub drop_target: RefCell<Option<gtk::DropTarget>>,
        pub handler_id: RefCell<Option<glib::SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DragOverlay {
        const NAME: &'static str = "DragOverlay";
        type Type = super::DragOverlay;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("dragoverlay");
        }
    }

    impl ObjectImpl for DragOverlay {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "title",
                        "title",
                        "title",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "child",
                        "child",
                        "child",
                        gtk::Widget::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "drop-target",
                        "drop-target",
                        "drop-target",
                        gtk::DropTarget::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.status.title().to_value(),
                "child" => self.overlay.child().to_value(),
                "drop-target" => self.drop_target.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "title" => self.status.set_title(value.get().unwrap()),
                "child" => self
                    .overlay
                    .set_child(value.get::<gtk::Widget>().ok().as_ref()),
                "drop-target" => obj.set_drop_target(&value.get::<gtk::DropTarget>().unwrap()),
                _ => unimplemented!(),
            };
        }

        fn constructed(&self, obj: &Self::Type) {
            self.overlay.set_parent(obj);
            self.overlay.add_overlay(&self.revealer);

            self.revealer.set_can_target(false);
            self.revealer
                .set_transition_type(gtk::RevealerTransitionType::Crossfade);
            self.revealer.set_reveal_child(false);

            self.status.set_icon_name(Some("document-send-symbolic"));
            self.status.add_css_class("drag-overlay-status-page");

            self.revealer.set_child(Some(&self.status));
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.overlay.unparent();
        }
    }
    impl WidgetImpl for DragOverlay {}
    impl BinImpl for DragOverlay {}
}

glib::wrapper! {
    pub struct DragOverlay(ObjectSubclass<imp::DragOverlay>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for DragOverlay {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

impl DragOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_drop_target(&self, drop_target: &gtk::DropTarget) {
        let priv_ = self.imp();

        if let Some(target) = priv_.drop_target.borrow_mut().take() {
            self.remove_controller(&target);

            if let Some(handler_id) = priv_.handler_id.borrow_mut().take() {
                target.disconnect(handler_id);
            }
        }

        let handler_id = drop_target.connect_current_drop_notify(
            glib::clone!(@weak priv_.revealer as revealer => move |target| {
                revealer.set_reveal_child(target.current_drop().is_some());
            }),
        );
        priv_.handler_id.replace(Some(handler_id));

        self.add_controller(drop_target);
        priv_.drop_target.replace(Some(drop_target.clone()));
        self.notify("drop-target");
    }
}
