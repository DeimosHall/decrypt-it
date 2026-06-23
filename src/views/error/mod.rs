use crate::impl_view_host;
use crate::traits::ViewHost;

use adw::subclass::bin::BinImpl;
use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/deimoshall/DecryptIt/ui/views/error/mod.ui")]
    pub struct ErrorView {
        #[template_child]
        pub error_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub copy_error_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub back_button: TemplateChild<gtk::Button>,

        pub view_host: RefCell<Option<Box<dyn ViewHost>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ErrorView {
        const NAME: &'static str = "ErrorView";
        type Type = super::ErrorView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ErrorView {}
    impl WidgetImpl for ErrorView {}
    impl BinImpl for ErrorView {}
}

glib::wrapper! {
    pub struct ErrorView(ObjectSubclass<imp::ErrorView>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ErrorView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_error(&self, error: String) {
        self.imp().error_page.set_description(Some(&error));
    }

    /// Copies the error message to the clipboard
    pub fn set_copy_error_listener(&self) {
        self.imp().copy_error_button.connect_clicked(glib::clone!(
            #[weak(rename_to=this)]
            self,
            move |button| {
                let clipboard = button.clipboard();
                let text = this.imp().error_page.description();
                if let Some(text) = text {
                    clipboard.set_text(&text);
                    this.show_toast(&gettext("Copied to clipboard"));
                }
            }
        ));
    }

    /// Sets a callback for the `Go Back` button
    pub fn set_on_back<F>(&self, on_back: F)
    where
        F: Fn(&ErrorView) + 'static,
    {
        let view = self.clone();
        self.imp().back_button.connect_clicked(move |_| {
            on_back(&view);
        });
    }
}

impl_view_host!(ErrorView);
