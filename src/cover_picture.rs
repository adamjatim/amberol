// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};

use gtk::{gdk, gio, glib, graphene, prelude::*, subclass::prelude::*};

#[derive(Clone, Copy, Debug, glib::Enum, PartialEq)]
#[enum_type(name = "AmberolCoverSize")]
pub enum CoverSize {
    Large = 0,
    Small = 1,
}

impl Default for CoverSize {
    fn default() -> Self {
        CoverSize::Large
    }
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
    use glib::{ParamFlags, ParamSpec, ParamSpecEnum, ParamSpecObject, Value};
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.add_css_class("cover");
            obj.set_overflow(gtk::Overflow::Hidden);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "cover",
                        "",
                        "",
                        gdk::Texture::static_type(),
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecEnum::new(
                        "cover-size",
                        "",
                        "",
                        CoverSize::static_type(),
                        CoverSize::default() as i32,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "cover" => self.cover.borrow().to_value(),
                "cover-size" => self.cover_size.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "cover" => obj.set_cover(value.get::<gdk::Texture>().ok().as_ref()),
                "cover-size" => {
                    obj.set_cover_size(value.get::<CoverSize>().expect("Required CoverSize"))
                }
                _ => unimplemented!(),
            };
        }
    }

    impl WidgetImpl for CoverPicture {
        fn request_mode(&self, _widget: &Self::Type) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::ConstantSize
        }

        fn measure(
            &self,
            _widget: &Self::Type,
            _orientation: gtk::Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            match self.cover_size.get() {
                CoverSize::Large => (LARGE_SIZE, LARGE_SIZE, -1, -1),
                CoverSize::Small => (SMALL_SIZE, SMALL_SIZE, -1, -1),
            }
        }

        fn snapshot(&self, widget: &Self::Type, snapshot: &gtk::Snapshot) {
            if let Some(ref cover) = *self.cover.borrow() {
                let width = widget.width() as f64;
                let height = widget.height() as f64;
                let ratio = cover.intrinsic_aspect_ratio();
                let w;
                let h;
                if ratio > 1.0 {
                    w = width;
                    h = height / ratio;
                } else {
                    w = width / ratio;
                    h = height;
                }

                let x = (width - w.ceil()) as i32 / 2;
                let y = (height - h.ceil()) as i32 / 2;

                snapshot.save();
                snapshot.translate(&graphene::Point::new(x as f32, y as f32));
                cover.snapshot(snapshot.upcast_ref::<gdk::Snapshot>(), w, h);
                snapshot.restore();
            }
        }
    }
}

glib::wrapper! {
    pub struct CoverPicture(ObjectSubclass<imp::CoverPicture>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for CoverPicture {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
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
