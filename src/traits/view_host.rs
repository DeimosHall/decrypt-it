use std::fmt::Debug;

pub trait ViewHost: Debug {
    /// Shows a basic toast with the given message.
    fn show_toast(&self, message: &str);
}

#[macro_export]
macro_rules! impl_view_host {
    ($view_type: ty) => {
        impl $view_type {
            pub fn set_view_host(&self, host: Box<dyn $crate::traits::ViewHost>) {
                self.imp().view_host.replace(Some(host));
            }

            pub fn show_toast(&self, message: &str) {
                if let Some(host) = self.imp().view_host.borrow().as_ref() {
                    host.show_toast(message);
                }
            }
        }
    };
}
