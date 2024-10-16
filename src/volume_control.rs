// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::Cell;

use adw::subclass::prelude::*;
use glib::clone;
use gtk::{gio, glib, prelude::*, CompositeTemplate};
use log::debug;

mod imp {
    use glib::{subclass::Signal, ParamSpec, ParamSpecBoolean, ParamSpecDouble, Value};
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/bassi/Amberol/volume-control.ui")]
    pub struct VolumeControl {
        // Template widgets
        #[template_child]
        pub volume_low_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub volume_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub volume_high_image: TemplateChild<gtk::Image>,

        pub toggle_mute: Cell<bool>,
        pub prev_volume: Cell<f64>,
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

            klass.install_property_action("volume.toggle-mute", "toggle-mute");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VolumeControl {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().setup_adjustment();
            self.obj().setup_controller();
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecDouble::builder("volume")
                        .minimum(0.0)
                        .maximum(1.0)
                        .default_value(1.0)
                        .build(),
                    ParamSpecBoolean::builder("toggle-mute").build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "volume" => {
                    let v = value.get::<f64>().expect("Failed to get f64 value");
                    self.volume_scale.set_value(v);
                }
                "toggle-mute" => {
                    let v = value.get::<bool>().expect("Failed to get a boolean value");
                    self.obj().toggle_mute(v);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "volume" => self.volume_scale.value().to_value(),
                "toggle-mute" => self.toggle_mute.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("volume-changed")
                    .param_types([f64::static_type()])
                    .build()]
            });

            SIGNALS.as_ref()
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
        glib::Object::new()
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
            clone!(
                #[strong(rename_to = this)]
                self,
                move |adj, _| {
                    let value = adj.value();
                    if value == adj.lower() {
                        this.imp()
                            .volume_low_button
                            .set_icon_name("audio-volume-muted-symbolic");
                    } else {
                        this.imp()
                            .volume_low_button
                            .set_icon_name("audio-volume-low-symbolic");
                    }
                    this.notify("volume");
                    this.emit_by_name::<()>("volume-changed", &[&value]);
                }
            ),
        );
    }

    fn setup_controller(&self) {
        let controller = gtk::EventControllerScroll::builder()
            .name("volume-scroll")
            .flags(gtk::EventControllerScrollFlags::VERTICAL)
            .build();
        controller.connect_scroll(clone!(
            #[strong(rename_to = this)]
            self,
            move |_, _, dy| {
                debug!("Scroll delta: {}", dy);
                let adj = this.imp().volume_scale.adjustment();
                let delta = dy * adj.step_increment();
                let d = (adj.value() - delta).clamp(adj.lower(), adj.upper());
                adj.set_value(d);
                glib::Propagation::Stop
            }
        ));
        self.imp().volume_scale.add_controller(controller);
    }

    fn toggle_mute(&self, muted: bool) {
        if muted != self.imp().toggle_mute.replace(muted) {
            if muted {
                let prev_value = self.imp().volume_scale.value();
                self.imp().prev_volume.replace(prev_value);
                self.imp().volume_scale.set_value(0.0);
            } else {
                let prev_value = self.imp().prev_volume.get();
                self.imp().volume_scale.set_value(prev_value);
            }
            self.notify("toggle-mute");
        }
    }

    pub fn volume(&self) -> f64 {
        self.imp().volume_scale.value()
    }
}
