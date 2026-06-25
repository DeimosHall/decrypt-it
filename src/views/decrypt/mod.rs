use adw::prelude::ActionRowExt;
use dlc_decoder::DlcDecoder;
use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::{impl_view_host, input_file::InputFile, traits::ViewHost};
use adw::subclass::bin::BinImpl;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/deimoshall/DecryptIt/ui/views/decrypt/mod.ui")]
    pub struct Decrypt {
        #[template_child]
        pub password_container: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub url_list_box: TemplateChild<gtk::ListBox>,

        pub view_host: RefCell<Option<Box<dyn ViewHost>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Decrypt {
        const NAME: &'static str = "Decrypt";
        type Type = super::DecryptView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Decrypt {}
    impl WidgetImpl for Decrypt {}
    impl BinImpl for Decrypt {}
}

glib::wrapper! {
    pub struct DecryptView(ObjectSubclass<imp::Decrypt>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DecryptView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn decrypt(&self, file: InputFile) -> Result<(String, Vec<String>), dlc_decoder::Error> {
        let decoder = DlcDecoder::new();
        let package = decoder.from_file(file.path())?;

        let urls: Vec<String> = package
            .files
            .iter()
            .filter_map(|link| Some(link.url.clone()))
            .collect();

        let password = package.password;

        Ok((password, urls))
    }

    fn build_row(&self, text: String) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(&text)
            .selectable(true)
            .build();

        let copy_button = gtk::Button::builder()
            .icon_name("edit-copy-symbolic")
            .css_classes(vec!["flat".to_string()])
            .build();

        copy_button.connect_clicked(glib::clone!(
            #[weak(rename_to=this)]
            self,
            move |button| {
                let clipboard = button.clipboard();
                clipboard.set_text(&text);
                this.show_toast(&gettext("Copied to clipboard"));
            }
        ));

        row.add_suffix(&copy_button);
        row
    }

    fn display_url(&self, url: String) {
        let url_field = self.build_row(url);
        self.imp().url_list_box.append(&url_field);
    }

    fn display_password(&self, password: String) {
        let password_field = self.build_row(password);
        self.imp().password_container.append(&password_field);
    }

    pub fn display_urls(&self, files: Vec<InputFile>) -> Result<(), dlc_decoder::Error> {
        // Clean previous password and urls
        self.imp().url_list_box.remove_all();
        self.imp().password_container.remove_all();

        for file in files {
            match self.decrypt(file) {
                Ok((password, urls)) => {
                    if password.is_empty() {
                        self.display_password(gettext("No password found"));
                    } else {
                        self.display_password(password);
                    }

                    for url in urls {
                        self.display_url(url.clone());
                    }
                }
                // This works well for one file
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}

// This expands to
// impl Decrypt { show_toast() }
impl_view_host!(DecryptView);
