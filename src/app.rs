use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation, Label, WindowPosition, Justification, Button, CssProvider, StyleContext, Settings};

use crate::core::Core;

pub struct MainApp {
    app: Application
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            app: Application::builder()
                .application_id("com.noinsts.usb-ejector")
                .build()
        }
    }
    
    pub fn run(&self) {
        self.app.connect_activate(|app| {
            MainApp::setup_ui(app);
        });

        self.app.run();
    }

    fn setup_ui(app: &Application) {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("USB Ejector")
            .default_width(350)
            .default_height(200)
            .resizable(false)
            .build();

        window.set_position(WindowPosition::Center);

        let vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(20)
            .margin(30)
            .build();

        let css_provider = Rc::new(RefCell::new(CssProvider::new()));
        
        let is_dark = Rc::new(RefCell::new(true));
        Self::load_css(&css_provider.borrow(), true);

        if let Some(settings) = Settings::default() {
            settings.set_property("gtk-application-prefer-dark-theme", true);
        }

        // Theme switch button
        let theme_button = Button::with_label("🌙");
        theme_button.set_size_request(40, 40);
        theme_button.set_tooltip_text(Some("Перейти до світлої теми"));
        theme_button.style_context().add_class("theme-switcher");

        let is_dark_clone = is_dark.clone();
        let css_provider_clone = css_provider.clone();

        theme_button.connect_clicked(move |button| {
            let mut dark = is_dark_clone.borrow_mut();
            *dark = !*dark;
            if let Some(settings) = Settings::default() {
                settings.set_property("gtk-application-prefer-dark-theme", *dark);
            }
            
            Self::load_css(&css_provider_clone.borrow(), *dark);

            if *dark {
                button.set_label("🌙");
                button.set_tooltip_text(Some("Перейти до світлої теми"));
            }
            else {
                button.set_label("☀️");
                button.set_tooltip_text(Some("Перейти до темної теми"));
            }
        });

        let header_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();

        header_box.pack_end(&theme_button, false, false, 0);

        vbox.pack_start(&header_box, false, false, 0);

        vbox.style_context().add_class("main-container");

        // Title label
        let title = Label::new(Some("USB Ejector"));
        title.set_markup("<span size='xx-large' weight='bold'>USB Ejector</span>");
        title.style_context().add_class("app-title");
        vbox.pack_start(&title, false, false, 0);

        // Description label
        let desc = Label::new(Some("Натисни кнопку, щоб витягнути всі зовнішні накопичувачі"));
        desc.set_line_wrap(true);
        desc.set_justify(Justification::Center);
        desc.style_context().add_class("app-description");
        vbox.pack_start(&desc, false, false, 0);

        // Eject button
        let eject_button = Button::with_label("⏏️  ВИТЯГНУТИ ВСІ");
        eject_button.set_size_request(200, 50);
        eject_button.style_context().add_class("eject-button");
        
        eject_button.connect_clicked(|_| {
            let state = Core::eject();
        });

        vbox.pack_start(&eject_button, false, false, 0);

        window.add(&vbox);
        window.show_all();
    }

    fn load_css(css_provider: &CssProvider, is_dark: bool) {
        let theme_file = if is_dark { "themes/dark.css" } else { "themes/light.css" };

        if Path::new(theme_file).exists() {
            match css_provider.load_from_path(theme_file) {
                Ok(_) => {
                    println!("CSS завантажено з файлу {}", theme_file);
                }
                Err(e) => {
                    println!("Помилка завантаження CSS з файлу {}: {}", theme_file, e);
                }
            }
        } 
        else {
            println!("Файл {} не знайдено.", theme_file);
        }

        StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Error initializing gtk css provider."), 
            css_provider, 
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
