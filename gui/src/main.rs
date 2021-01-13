use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Fancy");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    let button = gtk::Button::with_label("Click me!");

    window.add(&button);

    window.show_all();
}

fn main() {
    let app = gtk::Application::new(Some("com.musikid.fancy.gui"), Default::default())
        .expect("Initialization failed");

    app.connect_activate(|a| build_ui(a));

    app.run(&args().collect::<Vec<_>>());
}
