// SPDX-FileCopyrightText: 2022  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::RefCell, rc::Rc};

use adw::subclass::prelude::*;
use glib::{clone, Receiver};
use gtk::{gio, glib, prelude::*};
use gtk_macros::action;
use log::debug;

use crate::{
    audio::AudioPlayer,
    config::{APPLICATION_ID, VERSION},
    i18n::i18n,
    window::Window,
};

pub enum ApplicationAction {
    Present,
}

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct Application {
        pub player: Rc<AudioPlayer>,
        pub receiver: RefCell<Option<Receiver<ApplicationAction>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "AmberolApplication";
        type Type = super::Application;
        type ParentType = adw::Application;

        fn new() -> Self {
            let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let receiver = RefCell::new(Some(r));

            Self {
                player: AudioPlayer::new(sender),
                receiver,
            }
        }
    }

    impl ObjectImpl for Application {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_channel();
            obj.setup_gactions();

            obj.set_accels_for_action("app.quit", &["<primary>q"]);

            obj.set_accels_for_action("queue.add-song", &["<primary>s"]);
            obj.set_accels_for_action("queue.add-folder", &["<primary>a"]);
            obj.set_accels_for_action("queue.clear", &["<primary>L"]);
            obj.set_accels_for_action("queue.toggle", &["F9"]);
            obj.set_accels_for_action("queue.shuffle", &["<primary>r"]);

            obj.set_accels_for_action("win.seek-backwards", &["<primary>Left"]);
            obj.set_accels_for_action("win.seek-forward", &["<primary>Right"]);
            obj.set_accels_for_action("win.previous", &["<primary>b"]);
            obj.set_accels_for_action("win.next", &["<primary>n"]);
            obj.set_accels_for_action("win.play", &["<primary>p"]);
            obj.set_accels_for_action("win.copy", &["<primary>c"]);
        }
    }

    impl ApplicationImpl for Application {
        fn startup(&self, application: &Self::Type) {
            self.parent_startup(application);

            gtk::Window::set_default_icon_name(APPLICATION_ID);
        }

        fn activate(&self, application: &Self::Type) {
            debug!("Application::activate");

            application.present_main_window();
        }

        fn open(&self, application: &Self::Type, files: &[gio::File], _hint: &str) {
            debug!("Application::open");

            application.present_main_window();
            if let Some(window) = application.active_window() {
                window.downcast_ref::<Window>().unwrap().open_files(files);
            }
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Application {
    fn default() -> Self {
        glib::Object::new(&[
            ("application-id", &APPLICATION_ID),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
            // We don't change the resource path depending on the
            // profile, so we need to specify the base path ourselves
            ("resource-base-path", &"/io/bassi/Amberol"),
        ])
        .expect("Failed to create Application")
    }
}

impl Application {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player(&self) -> Rc<AudioPlayer> {
        self.imp().player.clone()
    }

    fn setup_channel(&self) {
        let receiver = self.imp().receiver.borrow_mut().take().unwrap();
        receiver.attach(
            None,
            clone!(@strong self as this => move |action| this.process_action(action)),
        );
    }

    fn process_action(&self, action: ApplicationAction) -> glib::Continue {
        match action {
            ApplicationAction::Present => self.present_main_window(),
            // _ => debug!("Received action {:?}", action),
        }

        glib::Continue(true)
    }

    fn present_main_window(&self) {
        let window = if let Some(window) = self.active_window() {
            window
        } else {
            let window = Window::new(self);
            window.upcast()
        };

        window.present();
    }

    fn setup_gactions(&self) {
        action!(
            self,
            "quit",
            clone!(@weak self as app => move |_, _| {
                app.quit();
            })
        );

        action!(
            self,
            "about",
            clone!(@weak self as app => move |_, _| {
                app.show_about();
            })
        );
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_icon(APPLICATION_ID)
            .application_name("Amberol")
            .developer_name("Emmanuele Bassi")
            .version(VERSION)
            .developers(vec!["Emmanuele Bassi".into()])
            .copyright("© 2022 Emmanuele Bassi")
            .website("https://apps.gnome.org/app/io.bassi.Amberol/")
            .issue_url("https://gitlab.gnome.org/World/amberol/-/issues/new")
            .license_type(gtk::License::Gpl30)
            // Translators: Replace "translator-credits" with your names, one name per line
            .translator_credits(&i18n("translator-credits"))
            .build();

        dialog.present();
    }
}
