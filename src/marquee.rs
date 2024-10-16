// SPDX-FileCopyrightText: 2023  Yuri Izmer
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::clone;
use gtk::gdk;
use gtk::glib;
use gtk::graphene;
use gtk::gsk;
use once_cell::sync::OnceCell;
use std::cell::Cell;

const SPACING: f32 = 32.0;

mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Marquee)]
    pub struct Marquee {
        child: gtk::Label,

        pub(super) rotation_progress: Cell<f32>,
        pub(super) label_fits: Cell<bool>,

        pub(super) animation: OnceCell<adw::TimedAnimation>,

        #[property(
            get = |imp: &Self| imp.child.label(),
            set = Self::set_label,
        )]
        pub(super) label: std::marker::PhantomData<glib::GString>,

        #[property(get, set = Self::set_width_chars)]
        pub(super) width_chars: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Marquee {
        const NAME: &'static str = "AmberolMarquee";
        type Type = super::Marquee;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("label");
            klass.set_accessible_role(gtk::AccessibleRole::Label);
        }
    }

    impl ObjectImpl for Marquee {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let widget = &*self.obj();

            self.child.set_parent(widget);
            self.child.set_xalign(0.5);

            let target = adw::CallbackAnimationTarget::new(clone!(
                #[weak]
                widget,
                move |value| {
                    widget.imp().set_rotation_progress(value as f32);
                }
            ));

            let animation = adw::TimedAnimation::builder()
                .widget(widget)
                .value_from(0.0)
                .value_to(1.0)
                .target(&target)
                .easing(adw::Easing::EaseInOutCubic)
                .build();

            // TODO: I think animation rest duration property can be useful in libadwaita
            animation.connect_done(clone!(
                #[weak]
                widget,
                move |_| {
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(1500),
                        move || {
                            let imp = widget.imp();
                            if !imp.label_fits.get() {
                                let animation = imp.animation.get().unwrap();
                                if animation.state() != adw::AnimationState::Playing {
                                    animation.play();
                                }
                            }
                        },
                    );
                }
            ));

            self.animation.set(animation).unwrap();
        }

        fn dispose(&self) {
            self.child.unparent();
        }
    }

    impl WidgetImpl for Marquee {
        fn realize(&self) {
            self.parent_realize();
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            if orientation == gtk::Orientation::Horizontal {
                (0, self.width_pixels(), -1, -1)
            } else {
                self.child.measure(orientation, for_size)
            }
        }

        fn size_allocate(&self, width: i32, _height: i32, _baseline: i32) {
            let (_min, natural) = self.child.preferred_size();

            let child_width = natural.width().max(width);

            self.child.allocate(child_width, natural.height(), -1, None);

            let animation = self.animation.get().unwrap();

            animation.set_duration(child_width.max(20) as u32 * 30);

            if self.child.width() > width {
                self.label_fits.set(false);
                self.start_animation();
            } else {
                self.label_fits.set(true);
                self.stop_animation();
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            if self.label_fits.get() {
                self.parent_snapshot(snapshot);
                return;
            }

            let widget = self.obj();

            let width = widget.width() as f32;
            let node = {
                let snapshot = gtk::Snapshot::new();
                self.parent_snapshot(&snapshot);
                snapshot.to_node()
            };

            let Some(node) = node else {
                return;
            };

            let label_bounds = node.bounds();

            let label_width = label_bounds.width();
            let label_height = label_bounds.height();

            let gradient_width = SPACING * 0.5;

            let bounds = graphene::Rect::new(
                -gradient_width,
                label_bounds.y(),
                width + gradient_width,
                label_height,
            );

            snapshot.push_mask(gsk::MaskMode::InvertedAlpha);
            {
                let l_start = bounds.top_left();
                let mut l_end = bounds.top_left();
                l_end.set_x(l_end.x() + gradient_width);

                snapshot.append_linear_gradient(
                    &bounds,
                    &l_start,
                    &l_end,
                    &[
                        gsk::ColorStop::new(0.0, gdk::RGBA::BLACK),
                        gsk::ColorStop::new(1.0, gdk::RGBA::TRANSPARENT),
                    ],
                );

                let mut r_start = bounds.top_right();
                r_start.set_x(r_start.x() - gradient_width);
                let r_end = bounds.top_right();

                snapshot.append_linear_gradient(
                    &bounds,
                    &r_start,
                    &r_end,
                    &[
                        gsk::ColorStop::new(0.0, gdk::RGBA::TRANSPARENT),
                        gsk::ColorStop::new(1.0, gdk::RGBA::BLACK),
                    ],
                );
            }

            snapshot.pop(); // mask node

            snapshot.push_clip(&bounds);

            snapshot.translate(&graphene::Point::new(
                -(label_width + SPACING) * self.rotation_progress.get(),
                0.0,
            ));

            snapshot.append_node(&node);
            snapshot.translate(&graphene::Point::new(label_width + SPACING, 0.0));
            snapshot.append_node(&node);

            snapshot.pop(); // clip

            snapshot.pop(); // mask child
        }
    }

    impl Marquee {
        fn set_label(&self, value: Option<glib::GString>) {
            if let Some(value) = value {
                self.child.set_label(&value);
            } else {
                self.child.set_label("");
            }

            // restart animation if label was changed
            let animation = self.animation.get().unwrap();
            if animation.state() == adw::AnimationState::Playing {
                animation.skip();
            }
        }

        fn set_rotation_progress(&self, value: f32) {
            self.rotation_progress.replace(value.rem_euclid(1.0));
            self.obj().queue_draw();
        }

        fn set_width_chars(&self, value: i32) {
            if self.width_chars.replace(value) != value {
                self.obj().queue_resize();
            }
        }

        fn width_pixels(&self) -> i32 {
            let metrics = self.obj().pango_context().metrics(None, None);

            let char_width = metrics
                .approximate_char_width()
                .max(metrics.approximate_digit_width());

            let width = char_width * self.width_chars.get();

            width / gtk::pango::SCALE
        }

        fn start_animation(&self) {
            let animation = self.animation.get().unwrap();
            if animation.state() != adw::AnimationState::Playing {
                glib::timeout_add_local_once(
                    std::time::Duration::from_millis(1500),
                    clone!(
                        #[weak]
                        animation,
                        move || {
                            if animation.state() != adw::AnimationState::Playing {
                                animation.play();
                            }
                        },
                    ),
                );
            }
        }

        fn stop_animation(&self) {
            let animation = self.animation.get().unwrap();
            animation.pause();
        }
    }
}

glib::wrapper! {
    pub struct Marquee(ObjectSubclass<imp::Marquee>)
        @extends gtk::Widget;
}

impl Default for Marquee {
    fn default() -> Self {
        glib::Object::new()
    }
}
