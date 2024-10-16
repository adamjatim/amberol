// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};

use glib::clone;
use gtk::{gdk, gio, glib, graphene, gsk, prelude::*, subclass::prelude::*};

use crate::i18n::i18n;

#[derive(Clone, Copy, Debug, glib::Enum, PartialEq, Default)]
#[enum_type(name = "AmberolCoverSize")]
pub enum CoverSize {
    #[default]
    Large = 0,
    Small = 1,
}

impl AsRef<str> for CoverSize {
    fn as_ref(&self) -> &str {
        match self {
            CoverSize::Large => "large",
            CoverSize::Small => "small",
        }
    }
}

mod imp {
    use glib::{ParamSpec, ParamSpecEnum, ParamSpecObject, Value};
    use once_cell::sync::Lazy;

    use super::*;

    const LARGE_SIZE: i32 = 192;
    const SMALL_SIZE: i32 = 48;

    #[derive(Debug, Default)]
    pub struct CoverPicture {
        pub cover: RefCell<Option<gdk::Texture>>,
        pub cover_size: Cell<CoverSize>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CoverPicture {
        const NAME: &'static str = "AmberolCoverPicture";
        type Type = super::CoverPicture;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("picture");
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for CoverPicture {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().add_css_class("cover");
            self.obj().set_overflow(gtk::Overflow::Hidden);

            self.obj().connect_notify_local(
                Some("scale-factor"),
                clone!(
                    #[weak(rename_to = _obj)]
                    self,
                    move |picture, _| {
                        picture.queue_draw();
                    }
                ),
            );

            self.obj()
                .upcast_ref::<gtk::Accessible>()
                .update_property(&[(gtk::accessible::Property::Label(&i18n("Cover image")))]);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<gdk::Texture>("cover").build(),
                    ParamSpecEnum::builder::<CoverSize>("cover-size").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "cover" => self.cover.borrow().to_value(),
                "cover-size" => self.cover_size.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "cover" => self
                    .obj()
                    .set_cover(value.get::<gdk::Texture>().ok().as_ref()),
                "cover-size" => self
                    .obj()
                    .set_cover_size(value.get::<CoverSize>().expect("Required CoverSize")),
                _ => unimplemented!(),
            };
        }
    }

    impl WidgetImpl for CoverPicture {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::ConstantSize
        }

        fn measure(&self, _orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match self.cover_size.get() {
                CoverSize::Large => (LARGE_SIZE, LARGE_SIZE, -1, -1),
                CoverSize::Small => (SMALL_SIZE, SMALL_SIZE, -1, -1),
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            if let Some(ref cover) = *self.cover.borrow() {
                let widget = self.obj();
                let scale_factor = widget.scale_factor() as f64;
                let width = widget.width() as f64 * scale_factor;
                let height = widget.height() as f64 * scale_factor;
                let ratio = cover.intrinsic_aspect_ratio();
                let w;
                let h;
                if ratio > 1.0 {
                    w = width;
                    h = width / ratio;
                } else {
                    w = height * ratio;
                    h = height;
                }

                let x = (width - w.ceil()) / 2.0;
                let y = (height - h).floor() / 2.0;

                snapshot.save();
                snapshot.scale(1.0 / scale_factor as f32, 1.0 / scale_factor as f32);
                snapshot.translate(&graphene::Point::new(x as f32, y as f32));
                snapshot.append_scaled_texture(
                    cover,
                    gsk::ScalingFilter::Trilinear,
                    &graphene::Rect::new(0.0, 0.0, w as f32, h as f32),
                );
                snapshot.restore();
            }
        }
    }
}

glib::wrapper! {
    pub struct CoverPicture(ObjectSubclass<imp::CoverPicture>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible;
}

impl Default for CoverPicture {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl CoverPicture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cover(&self) -> Option<gdk::Texture> {
        (*self.imp().cover.borrow()).as_ref().cloned()
    }

    pub fn set_cover(&self, cover: Option<&gdk::Texture>) {
        if let Some(cover) = cover {
            self.imp().cover.replace(Some(cover.clone()));
        } else {
            self.imp().cover.replace(None);
        }

        self.queue_draw();
        self.notify("cover");
    }

    pub fn set_cover_size(&self, cover_size: CoverSize) {
        self.imp().cover_size.replace(cover_size);
        self.queue_resize();
        self.notify("cover-size");
    }
}
