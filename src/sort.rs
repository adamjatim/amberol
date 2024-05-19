// SPDX-FileCopyrightText: 2022  John Toohey <john_t@mailo.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {

    use std::cell::RefCell;

    use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
    use gtk::{
        glib::{self, ParamSpec, ParamSpecString, Value},
        prelude::*,
        subclass::prelude::*,
    };
    use once_cell::sync::Lazy;

    use crate::{audio::Song, utils::cmp_two_files};

    #[derive(Default)]
    pub struct FuzzySorter {
        pub search: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FuzzySorter {
        const NAME: &'static str = "AmberolFuzzySorter";
        type Type = super::FuzzySorter;
        type ParentType = gtk::Sorter;
    }

    impl ObjectImpl for FuzzySorter {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecString::builder("search").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "search" => {
                    let p = value
                        .get::<Option<String>>()
                        .expect("Value must be a string");
                    self.obj().set_search(p);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl SorterImpl for FuzzySorter {
        fn compare(&self, item1: &glib::Object, item2: &glib::Object) -> gtk::Ordering {
            let item1 = item1.downcast_ref::<Song>().unwrap();
            let item2 = item2.downcast_ref::<Song>().unwrap();

            if let Some(search) = self.search.borrow().as_ref() {
                let matcher = SkimMatcherV2::default();
                let item1_key = item1.search_key();
                let item2_key = item2.search_key();
                let item1_score = matcher.fuzzy_match(&item1_key, search);
                let item2_score = matcher.fuzzy_match(&item2_key, search);
                item1_score.cmp(&item2_score).reverse().into()
            } else {
                cmp_two_files(None, &item1.file(), &item2.file()).into()
            }
        }

        fn order(&self) -> gtk::SorterOrder {
            gtk::SorterOrder::Partial
        }
    }
}

glib::wrapper! {
    pub struct FuzzySorter(ObjectSubclass<imp::FuzzySorter>)
        @extends gtk::Sorter;

}

impl Default for FuzzySorter {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzySorter {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn search(&self) -> Option<String> {
        self.imp().search.borrow().as_ref().map(ToString::to_string)
    }

    pub fn set_search(&self, search: Option<String>) {
        if *self.imp().search.borrow() != search {
            *self.imp().search.borrow_mut() = search;
            self.changed(gtk::SorterChange::Different);
        }
    }
}
