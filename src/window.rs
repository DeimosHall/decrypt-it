use std::collections::HashSet;
use std::sync::atomic::AtomicUsize;

use crate::components::about_window::DecryptItAbout;
use crate::components::drag_overlay::DragOverlay;
use crate::config::APP_ID;
use crate::file_chooser::FileChooser;
use crate::input_file::InputFile;
use crate::runtime;
use crate::services::exif::ExifService;
use adw::prelude::*;
use dlc_decoder::DlcDecoder;
use futures::future::join_all;
use gettextrs::gettext;
use glib::{MainContext, clone, idle_add_local_once};
use gtk::gdk::Texture;
use gtk::{gdk, gio, glib, subclass::prelude::*};
use itertools::Itertools;
use shared_child::SharedChild;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeFilter {
    Default,
    Point,
}

#[allow(dead_code)]
impl ResizeFilter {
    pub fn as_display_string(&self) -> Option<&str> {
        match self {
            ResizeFilter::Default => None,
            ResizeFilter::Point => Some("Point"),
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(ResizeFilter::Default),
            1 => Some(ResizeFilter::Point),
            _ => None,
        }
    }
}

mod imp {
    use std::{
        cell::{Cell, RefCell},
        sync::atomic::AtomicBool,
    };

    use crate::config::PKGDATADIR;

    use super::*;

    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use derivative::Derivative;
    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate, Derivative)]
    #[derivative(Default)]
    #[template(resource = "/dev/deimoshall/DecryptIt/ui/window.ui")]
    pub struct AppWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub drag_overlay: TemplateChild<DragOverlay>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub open_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub loading_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub url_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub url_list_box: TemplateChild<gtk::ListBox>,
        // TODO: delete
        #[template_child]
        pub view_switcher: TemplateChild<adw::ViewSwitcher>,

        #[template_child]
        pub navigation: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub help_overlay: TemplateChild<adw::ShortcutsDialog>,

        #[derivative(Default(value = "gio::ListStore::new::<InputFile>()"))]
        pub input_file_store: gio::ListStore,
        #[derivative(Default(value = "gio::Settings::new(APP_ID)"))]
        pub settings: gio::Settings,
        #[derivative(Default(value = "std::sync::Arc::new(AtomicBool::new(true))"))]
        pub is_canceled: std::sync::Arc<AtomicBool>,
        pub current_jobs: RefCell<Vec<Arc<SharedChild>>>,
        pub image_width: Cell<Option<u32>>,
        pub image_height: Cell<Option<u32>>,
        pub removed: RefCell<HashSet<u32>>,
        pub elements: Cell<usize>,
    }

    #[::glib::object_subclass]
    impl ObjectSubclass for AppWindow {
        const NAME: &'static str = "AppWindow";
        type Type = super::AppWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self::default()
        }
    }

    impl ObjectImpl for AppWindow {
        fn constructed(&self) {
            self.parent_constructed();

            // Load CSS
            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/dev/deimoshall/DecryptIt/style.css");

            if let Some(display) = gtk::gdk::Display::default() {
                gtk::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }

            let theme = gtk::IconTheme::for_display(
                &gtk::gdk::Display::default().expect("cannot find display"),
            );
            theme.add_search_path(PKGDATADIR.to_owned() + "/icons");

            let obj = self.obj();
            obj.load_window_size();
            obj.setup_gactions();
        }
    }

    impl WidgetImpl for AppWindow {}
    impl WindowImpl for AppWindow {
        fn close_request(&self) -> glib::Propagation {
            if let Err(err) = self.obj().save_window_size() {
                dbg!("Failed to save window state, {}", &err);
            }

            if !self.is_canceled.load(std::sync::atomic::Ordering::SeqCst) {
                self.obj().close_dialog();
                glib::Propagation::Stop
            } else {
                // Pass close request on to the parent
                self.parent_close_request()
            }
        }
    }

    impl ApplicationWindowImpl for AppWindow {}
    impl AdwApplicationWindowImpl for AppWindow {}
}

glib::wrapper! {
    pub struct AppWindow(ObjectSubclass<imp::AppWindow>)
        @extends gtk::Widget, gtk::Window,  gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup,
                    gtk::Root, gtk::Native, gtk::ShortcutManager,
                    gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl AppWindow {
    pub fn new<P: glib::prelude::IsA<gtk::Application>>(app: &P) -> Self {
        let win = glib::Object::builder::<AppWindow>()
            .property("application", app)
            .build();

        win.setup_callbacks();
        win.setup_drop_target();

        win
    }

    /// Shows a basic toast with the given text.
    fn show_toast(&self, text: &str) {
        self.imp().toast_overlay.add_toast(adw::Toast::new(text));
    }

    fn setup_gactions(&self) {
        self.add_action_entries([
            gio::ActionEntry::builder("close")
                .activate(clone!(
                    #[weak(rename_to=window)]
                    self,
                    move |_, _, _| {
                        window.destroy();
                    }
                ))
                .build(),
            gio::ActionEntry::builder("add")
                .activate(clone!(
                    #[weak(rename_to=window)]
                    self,
                    move |_, _, _| {
                        window.add_dialog();
                    }
                ))
                .build(),
            gio::ActionEntry::builder("clear")
                .activate(clone!(
                    #[weak(rename_to=window)]
                    self,
                    move |_, _, _| {
                        window.clear();
                    }
                ))
                .build(),
            gio::ActionEntry::builder("show-help-overlay")
                .activate(clone!(
                    #[weak(rename_to=window)]
                    self,
                    move |_, _, _| {
                        window.show_help_overlay();
                    }
                ))
                .build(),
            gio::ActionEntry::builder("about")
                .activate(clone!(
                    #[weak(rename_to=window)]
                    self,
                    move |_, _, _| {
                        window.show_about();
                    }
                ))
                .build(),
        ]);
    }

    fn setup_callbacks(&self) {
        //load imp
        let imp = self.imp();

        imp.open_button.connect_clicked(clone!(
            #[weak(rename_to=this)]
            self,
            move |_| {
                this.add_dialog();
            }
        ));

        imp.add_button.connect_clicked(clone!(
            #[weak(rename_to=this)]
            self,
            move |_| {
                this.add_dialog();
            }
        ));

        imp.cancel_button.connect_clicked(clone!(
            #[weak(rename_to=this)]
            self,
            move |_| {
                this.apply_cancel();
            }
        ));
    }

    fn setup_drop_target(&self) {
        let drop_target = gtk::DropTarget::builder()
            .name("file-drop-target")
            .actions(gdk::DragAction::COPY)
            .formats(&gdk::ContentFormats::for_type(gdk::FileList::static_type()))
            .build();

        drop_target.connect_drop(clone!(
            #[weak(rename_to=win)]
            self,
            #[upgrade_or_default]
            move |_, value, _, _| {
                if let Ok(file_list) = value.get::<gdk::FileList>() {
                    if file_list.files().is_empty() {
                        win.show_toast(&gettext("Unable to access dropped files"));
                        return false;
                    }

                    let input_files = file_list.files().iter().map(InputFile::new).collect_vec();
                    win.open_files(input_files);
                    return true;
                }

                false
            }
        ));

        self.imp().drag_overlay.set_drop_target(&drop_target);
    }

    fn show_about(&self) {
        DecryptItAbout::show(self);
    }

    fn show_help_overlay(&self) {
        self.imp().help_overlay.present(Some(self));
    }

    fn close_dialog(&self) {
        let stop_converting_dialog = adw::AlertDialog::new(
            Some(&gettext("Stop converting?")),
            Some(&gettext("You will lose all progress.")),
        );

        stop_converting_dialog.add_response("cancel", &gettext("_Cancel"));
        stop_converting_dialog.add_response("stop", &gettext("_Stop"));
        stop_converting_dialog
            .set_response_appearance("stop", adw::ResponseAppearance::Destructive);
        stop_converting_dialog.connect_response(
            None,
            clone!(
                #[weak(rename_to=this)]
                self,
                move |_, response_id| {
                    if response_id == "stop" {
                        this.imp()
                            .is_canceled
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                        let mut current_jobs = this.imp().current_jobs.borrow_mut();
                        for x in current_jobs.iter() {
                            match x.kill() {
                                Ok(_) => {}
                                Err(_) => {
                                    x.wait().ok();
                                }
                            }
                        }
                        current_jobs.clear();
                        this.close();
                    }
                }
            ),
        );
        stop_converting_dialog.present(Some(self));
    }

    fn set_convert_progress(&self, done: usize, total: usize) {
        let msg = format!("{done}/{total}");
        self.imp().progress_bar.set_text(Some(&msg));
        self.imp()
            .progress_bar
            .set_fraction((done as f64) / (total as f64));
    }

    fn decrypt(&self, files: Vec<InputFile>) {
        let decoder = DlcDecoder::new();
        let dlc = decoder.from_file(files.first().unwrap().path());

        let url_group = &self.imp().url_group;
        let url_list_box = &self.imp().url_list_box;
        url_list_box.remove_all();

        match dlc {
            Ok(package) => {
                let urls: Vec<String> = package
                    .files
                    .iter()
                    .filter_map(|link| Some(link.url.clone()))
                    .collect();

                for url in urls {
                    let row = adw::ActionRow::builder()
                        .title(url.clone())
                        .selectable(true)
                        .build();

                    let copy_button = gtk::Button::builder()
                        .icon_name("edit-copy-symbolic")
                        .css_classes(vec!["flat".to_string()])
                        .build();

                    let url_clone = url.clone();
                    copy_button.connect_clicked(move |button| {
                        let clipboard = button.clipboard();
                        clipboard.set_text(&url_clone);
                    });

                    row.add_suffix(&copy_button);

                    url_list_box.append(&row);
                }
            }
            Err(error) => self.show_toast(&error.to_string()),
        };
    }

    fn open_success(&self, mut files: Vec<InputFile>) {
        let prev_files = self.active_files();
        let prev_files_paths = prev_files.iter().map(|f| f.path()).collect_vec();
        files = files
            .into_iter()
            .filter(|f| !prev_files_paths.contains(&f.path()))
            .chain(prev_files)
            .filter(|f| f.exists())
            .collect();

        // TODO: allow more than one file
        if files.len() > 1 {
            files.truncate(1);
        }

        self.imp().input_file_store.remove_all();
        self.imp().removed.replace(HashSet::new());

        self.switch_to_stack_loading_generally();

        for file in files.iter() {
            self.imp().input_file_store.append(file);
        }

        let _ = fdlimit::raise_fd_limit();

        self.switch_to_stack_apply();
        self.imp()
            .url_group
            .set_title(&files.first().unwrap().path());
        self.decrypt(files);
    }

    pub fn clear(&self) {
        self.imp().input_file_store.remove_all();

        self.switch_to_stack_welcome();
    }

    fn active_files(&self) -> Vec<InputFile> {
        let removed = self.imp().removed.borrow().clone();
        self.files()
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !removed.contains(&(*i as u32)))
            .map(|(_, f)| f)
            .collect_vec()
    }

    fn files(&self) -> Vec<InputFile> {
        self.imp()
            .input_file_store
            .iter::<InputFile>()
            .flatten()
            .collect_vec()
    }

    fn files_count(&self) -> usize {
        (self.imp().input_file_store.n_items() as usize) - self.imp().removed.borrow().len()
    }
}

pub trait FileOperations {
    fn add_dialog(&self);
    fn open_files(&self, files: Vec<Option<InputFile>>);
    fn open_load(&self);
    fn open_error(&self, error: Option<&str>);
    fn add_success_wrapper(&self, files: Vec<InputFile>);
}

trait StackNavigation {
    fn switch_to_stack_apply(&self);
    fn switch_to_stack_applying(&self);
    fn switch_to_stack_welcome(&self);
    fn switch_to_stack_invalid_image(&self);
    fn switch_to_stack_loading(&self);
    fn switch_back_from_loading(&self);
    fn switch_to_stack_loading_generally(&self);
}

pub trait WindowUI {
    fn apply_cancel(&self);
}

trait SettingsStore {
    fn save_window_size(&self) -> Result<(), glib::BoolError>;
    fn load_window_size(&self);
}

impl WindowUI for AppWindow {
    fn apply_cancel(&self) {}
}

impl StackNavigation for AppWindow {
    fn switch_to_stack_apply(&self) {
        self.imp().add_button.set_visible(true);
        self.imp().view_switcher.set_visible(true);
        self.imp().stack.set_visible_child_name("stack_apply");
    }

    fn switch_to_stack_applying(&self) {
        self.imp().add_button.set_visible(false);
        self.imp().view_switcher.set_visible(false);
        self.imp().stack.set_visible_child_name("stack_applying");
    }

    fn switch_to_stack_welcome(&self) {
        self.imp().add_button.set_visible(false);
        self.imp().view_switcher.set_visible(false);
        self.imp()
            .stack
            .set_visible_child_name("stack_welcome_page");
    }

    fn switch_to_stack_invalid_image(&self) {
        self.imp().add_button.set_visible(false);
        self.imp().view_switcher.set_visible(false);
        self.imp()
            .stack
            .set_visible_child_name("stack_invalid_image");
    }

    fn switch_to_stack_loading(&self) {
        self.imp().add_button.set_visible(false);
        self.imp().view_switcher.set_visible(false);
        self.imp().stack.set_visible_child_name("stack_loading");
        self.imp().loading_spinner.start();
    }

    fn switch_back_from_loading(&self) {
        self.imp().loading_spinner.stop();
    }

    fn switch_to_stack_loading_generally(&self) {
        self.switch_to_stack_loading();
    }
}

impl SettingsStore for AppWindow {
    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        let is_maximized = imp.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }
}

impl FileOperations for AppWindow {
    fn open_files(&self, files: Vec<Option<InputFile>>) {
        let files = files.into_iter().flatten().collect_vec();
        if files.is_empty() {
            self.show_toast(&gettext("Unsupported filetype"));
            return;
        }
        self.add_success_wrapper(files);
    }

    fn add_dialog(&self) {
        FileChooser::open_files_wrapper(
            self,
            vec![],
            AppWindow::open_load,
            AppWindow::add_success_wrapper,
            AppWindow::open_error,
        );
    }

    fn open_error(&self, error: Option<&str>) {
        if error.is_some() {
            self.switch_to_stack_invalid_image();
        }
    }

    fn open_load(&self) {
        self.switch_to_stack_loading_generally();
        self.imp().loading_spinner.start();
    }

    fn add_success_wrapper(&self, files: Vec<InputFile>) {
        self.open_success(files);
    }
}
