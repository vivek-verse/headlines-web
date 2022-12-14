pub mod headlines;

use eframe::egui::{self, Hyperlink, Label, RichText, TopBottomPanel, Ui, Visuals};
use eframe::Storage;
use eframe::{
    egui::{CentralPanel, ScrollArea, Separator},
    App,
};
use headlines::HeadlinesConfig;
pub use headlines::{Headlines, PADDING};

impl App for Headlines {
    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "headlines", &self.config);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        if let Some(storage) = frame.storage() {
            let values: HeadlinesConfig =
                eframe::get_value(storage, "headlines").unwrap_or_default();
            if !values.api_key.is_empty() {
                self.config.api_key = values.api_key;
                self.api_key_initialized = true;
            }
        }

        if self.config.dark_mode {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }

        if !self.initial_is_set && self.api_key_initialized {
            self.load_data();
            self.initial_is_set = true;
        }

        if self.news_rx.is_some() {
            self.preload_articles();
        }

        if !self.api_key_initialized {
            self.render_config(ctx);
        } else {
            self.render_top_panel(ctx, frame);
            CentralPanel::default().show(ctx, |ui| {
                render_header(ui);
                ScrollArea::both().show(ui, |ui| {
                    self.render_news_cards(ui);
                });
                render_footer(ctx);
            });
            self.configure_fonts(ctx);
        }
    }
}

fn render_footer(ctx: &egui::Context) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);

            ui.add(Label::new(
                RichText::new("API source: newsapi.org").monospace(),
            ));

            ui.add(Hyperlink::from_label_and_url(
                RichText::new("Made with egui").text_style(eframe::egui::TextStyle::Monospace),
                "https://github.com/emilk/egui",
            ));

            ui.add(Hyperlink::from_label_and_url(
                RichText::new("vivek-verse/headlines-app")
                    .text_style(eframe::egui::TextStyle::Monospace),
                "https://github.com/vivek-verse/headlines-app",
            ));

            ui.add_space(10.);
        });
    });
}

fn render_header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("headlines");
    });
    ui.add_space(PADDING);
    let sep = Separator::default().spacing(20.);
    ui.add(sep);
}

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_web(canvas_id: &str) {
    let headlines = Headlines::new();
    tracing_wasm::set_as_global_default();
    let web_options = eframe::WebOptions::default();
    eframe::start_web(canvas_id, web_options, Box::new(|_| Box::new(headlines)));
}
