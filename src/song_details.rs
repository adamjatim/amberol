// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};

use crate::cover_picture::CoverPicture;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/bassi/Amberol/song-details.ui")]
    pub struct SongDetails {
        // Template widgets
        #[template_child]
        pub song_title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub song_artist_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub song_album_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub cover_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub album_image: TemplateChild<CoverPicture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SongDetails {
        const NAME: &'static str = "AmberolSongDetails";
        type Type = super::SongDetails;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("songdetails");
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            CoverPicture::static_type();
            obj.init_template();
        }
    }

    impl ObjectImpl for SongDetails {
        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for SongDetails {}
}

glib::wrapper! {
    pub struct SongDetails(ObjectSubclass<imp::SongDetails>)
        @extends gtk::Widget;
}

impl Default for SongDetails {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create SongDetails")
    }
}

impl SongDetails {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn artist_label(&self) -> gtk::Label {
        self.imp().song_artist_label.get()
    }

    pub fn title_label(&self) -> gtk::Label {
        self.imp().song_title_label.get()
    }

    pub fn album_label(&self) -> gtk::Label {
        self.imp().song_album_label.get()
    }

    pub fn album_image(&self) -> CoverPicture {
        self.imp().album_image.get()
    }

    pub fn show_cover_image(&self, has_image: bool) {
        let cover_stack = self.imp().cover_stack.get();
        if has_image {
            cover_stack.set_visible_child_name("cover-image");
        } else {
            cover_stack.set_visible_child_name("no-image");
        }
    }
}
