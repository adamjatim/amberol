// SPDX-FileCopyrightText: 2022  John Toohey <john_t@mailo.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {

    use std::cell::RefCell;

    use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
    use gtk::{
        glib::{self, ParamFlags, ParamSpec, ParamSpecString, Value},
        prelude::*,
        subclass::prelude::*,
    };
    use once_cell::sync::Lazy;

    use crate::audio::Song;

    #[derive(Default)]
    pub struct FuzzyFilter {
        pub search: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FuzzyFilter {
        const NAME: &'static str = "AmberolFuzzyFilter";
        type Type = super::FuzzyFilter;
        type ParentType = gtk::Filter;
    }

    impl ObjectImpl for FuzzyFilter {
        fn constructed(&self, object: &Self::Type) {
            self.parent_constructed(object);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecString::new(
                    "search",
                    "search",
                    "a search term",
                    None,
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "search" => {
                    let p = value
                        .get::<Option<String>>()
                        .expect("Value must be a string");
                    obj.set_search(p);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl FilterImpl for FuzzyFilter {
        fn strictness(&self, _filter: &Self::Type) -> gtk::FilterMatch {
            gtk::FilterMatch::Some
        }

        fn match_(&self, _filter: &Self::Type, song: &glib::Object) -> bool {
            let song = song.downcast_ref::<Song>().unwrap();

            if let Some(search) = self.search.borrow().as_ref() {
                let key = song.search_key();
                let matcher = SkimMatcherV2::default();
                matcher.fuzzy_match(&key, search).is_some() || search.is_empty()
            } else {
                true
            }
        }
    }
}

glib::wrapper! {
    pub struct FuzzyFilter(ObjectSubclass<imp::FuzzyFilter>)
        @extends gtk::Filter;

}

impl FuzzyFilter {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create `AmberolFuzzyFilter`")
    }

    pub fn search(&self) -> Option<String> {
        self.imp().search.borrow().as_ref().map(ToString::to_string)
    }

    pub fn set_search(&self, search: Option<String>) {
        if &*self.imp().search.borrow() != &search {
            *self.imp().search.borrow_mut() = search.map(|x| x.to_lowercase());
            self.changed(gtk::FilterChange::Different);
        }
    }
}
