// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::subclass::prelude::*;
use glib::{clone, ParamFlags, ParamSpec, ParamSpecDouble, Value};
use gtk::{gio, glib, prelude::*, CompositeTemplate};
use log::debug;
use once_cell::sync::Lazy;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/bassi/Amberol/volume-control.ui")]
    pub struct VolumeControl {
        // Template widgets
        #[template_child]
        pub volume_low_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub volume_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub volume_high_image: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VolumeControl {
        const NAME: &'static str = "AmberolVolumeControl";
        type Type = super::VolumeControl;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.set_css_name("volume");
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VolumeControl {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_adjustment();
            obj.setup_controller();
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.volume_low_image.unparent();
            self.volume_scale.unparent();
            self.volume_high_image.unparent();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecDouble::new(
                    "volume",
                    "",
                    "",
                    0.0,
                    1.0,
                    1.0,
                    ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "volume" => self.volume_scale.set_value(
                    value
                        .get::<f64>()
                        .expect("Failed to get a floating point value"),
                ),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "volume" => self.volume_scale.value().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for VolumeControl {}
}

glib::wrapper! {
    pub struct VolumeControl(ObjectSubclass<imp::VolumeControl>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for VolumeControl {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create VolumeControl")
    }
}

impl VolumeControl {
    pub fn new() -> Self {
        Self::default()
    }

    fn setup_adjustment(&self) {
        let adj = gtk::Adjustment::builder()
            .lower(0.0)
            .upper(1.0)
            .step_increment(0.05)
            .value(1.0)
            .build();
        self.imp().volume_scale.set_adjustment(&adj);
        adj.connect_notify_local(
            Some("value"),
            clone!(@strong self as this => move |adj, _| {
                let value = adj.value();
                if value == adj.lower() {
                    this.imp().volume_low_image.set_icon_name(Some("audio-volume-muted-symbolic"));
                } else {
                    this.imp().volume_low_image.set_icon_name(Some("audio-volume-low-symbolic"));
                }

                this.notify("volume");
            }),
        );
    }

    fn setup_controller(&self) {
        let controller = gtk::EventControllerScroll::builder()
            .name("volume-scroll")
            .flags(gtk::EventControllerScrollFlags::VERTICAL)
            .build();
        controller.connect_scroll(clone!(@strong self as this => move |_, _, dy| {
            debug!("Scroll delta: {}", dy);
            let adj = this.imp().volume_scale.adjustment();
            let delta = dy * adj.step_increment();
            let d = (adj.value() - delta).clamp(adj.lower(), adj.upper());
            adj.set_value(d);
            gtk::Inhibit(true)
        }));
        self.imp().volume_scale.add_controller(&controller);
    }

    pub fn volume(&self) -> f64 {
        self.imp().volume_scale.value()
    }
}
