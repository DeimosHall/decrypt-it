use std::collections::HashSet;

use crate::config::APP_ID;
use crate::file_chooser::FileChooser;
use crate::input_file::InputFile;
use crate::traits::ViewHost;
use crate::{components::drag_overlay::DragOverlay, views::about::About};
use adw::prelude::*;
use gettextrs::gettext;
use glib::clone;
use gtk::{gdk, gio, glib, subclass::prelude::*};
use itertools::Itertools;

mod imp {
    use std::cell::{Cell, RefCell};

    use crate::{
        config::PKGDATADIR,
        views::{decrypt::DecryptView, error::ErrorView},
    };

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
        pub loading_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub decrypt_view: TemplateChild<DecryptView>,
        #[template_child]
        pub error_view: TemplateChild<ErrorView>,

        #[template_child]
        pub navigation: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub help_overlay: TemplateChild<adw::ShortcutsDialog>,

        #[derivative(Default(value = "gio::ListStore::new::<InputFile>()"))]
        pub input_file_store: gio::ListStore,
        #[derivative(Default(value = "gio::Settings::new(APP_ID)"))]
        pub settings: gio::Settings,
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

            self.parent_close_request()
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

        imp.error_view.set_copy_error_listener();

        imp.error_view.set_on_back(clone!(
            #[weak(rename_to=window)]
            self,
            move |_| {
                window.switch_to_stack_welcome();
            }
        ));

        imp.decrypt_view.set_view_host(Box::new(self.clone()));

        imp.error_view.set_view_host(Box::new(self.clone()));
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
        About::show(self);
    }

    fn show_help_overlay(&self) {
        self.imp().help_overlay.present(Some(self));
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

        for file in files.iter() {
            self.imp().input_file_store.append(file);
        }

        let _ = fdlimit::raise_fd_limit();

        match self.imp().decrypt_view.display_urls(files) {
            Ok(()) => {
                self.switch_to_stack_decrypt();
            }
            Err(err) => {
                self.imp().error_view.set_error(err.to_string());
                self.show_toast(&gettext("Something went wrong"));
                self.switch_to_stack_invalid_file();
            }
        }
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
}

pub trait FileOperations {
    fn add_dialog(&self);
    fn open_files(&self, files: Vec<Option<InputFile>>);
    fn open_load(&self);
    fn open_error(&self, error: Option<&str>);
    fn add_success_wrapper(&self, files: Vec<InputFile>);
}

trait StackNavigation {
    fn switch_to_stack_welcome(&self);
    fn switch_to_stack_decrypt(&self);
    fn switch_to_stack_invalid_file(&self);
    fn switch_to_stack_loading(&self);
}

trait SettingsStore {
    fn save_window_size(&self) -> Result<(), glib::BoolError>;
    fn load_window_size(&self);
}

impl ViewHost for AppWindow {
    fn show_toast(&self, text: &str) {
        self.imp().toast_overlay.add_toast(adw::Toast::new(text));
    }
}

impl StackNavigation for AppWindow {
    fn switch_to_stack_welcome(&self) {
        self.imp().add_button.set_visible(false);
        // Set a different transition type
        self.imp()
            .stack
            .set_transition_type(gtk::StackTransitionType::SlideLeftRight);
        self.imp().stack.set_visible_child_name("stack_welcome");
        // Return to default transition type used in the rest of the views
        self.imp()
            .stack
            .set_transition_type(gtk::StackTransitionType::Crossfade);
    }

    fn switch_to_stack_decrypt(&self) {
        self.imp().add_button.set_visible(true);
        self.imp().stack.set_visible_child_name("stack_decrypt");
    }

    fn switch_to_stack_invalid_file(&self) {
        self.imp().add_button.set_visible(false);
        self.imp()
            .stack
            .set_visible_child_name("stack_invalid_file");
    }

    fn switch_to_stack_loading(&self) {
        self.imp().add_button.set_visible(false);
        self.imp().stack.set_visible_child_name("stack_loading");
        self.imp().loading_spinner.start();
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
            self.switch_to_stack_invalid_file();
        }
    }

    fn open_load(&self) {
        self.switch_to_stack_loading();
    }

    fn add_success_wrapper(&self, files: Vec<InputFile>) {
        self.open_success(files);
    }
}
