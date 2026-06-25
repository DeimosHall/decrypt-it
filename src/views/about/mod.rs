use adw::prelude::*;
use gettextrs::gettext;
use glib::object::IsA;

// Code 'inspired' by https://gitlab.com/news-flash/news_flash_gtk/-/blob/master/src/about_dialog.rs

// This is non-translatable information, so it can be const
pub const DEVELOPERS: &[&str] = &["Deimos Hall <deimoshall@proton.me>"];

#[derive(Clone, Debug)]
pub struct AboutDialog;

impl AboutDialog {
    pub fn show<W: IsA<gtk::Widget>>(window: &W) {
        let about = adw::AboutDialog::from_appdata(
            "/dev/deimoshall/DecryptIt/dev.deimoshall.DecryptIt.metainfo.xml",
            Some(crate::config::VERSION),
        );

        about.set_developers(DEVELOPERS);
        about.set_translator_credits(&gettext("translator-credits"));

        about.add_other_app(
            "dev.deimoshall.Metamorphosis",
            &gettext("Metamorphosis"),
            &gettext("Edit Metadata"),
        );

        about.add_acknowledgement_section(
            Some(&gettext("Code and Design Borrowed from")),
            &[
                "Switcheroo https://gitlab.com/adhami3310/Switcheroo",
                "Fractal https://gitlab.gnome.org/World/fractal",
            ],
        );

        about.set_copyright("Copyright © 2026 Deimos Hall");
        about.set_comments("Decrypt DLC files");
        about.present(Some(window));
    }
}
