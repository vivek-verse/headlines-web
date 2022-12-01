use eframe::egui::{self, Button, TopBottomPanel, Window};
use eframe::egui::{
    Align, Color32, FontData, FontDefinitions, FontFamily, Hyperlink, Label, Layout, RichText,
    Separator,
};
use newslib::{NewsAPI, NewsAPIResponse};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::{sync::mpsc::channel, thread};

pub const PADDING: f32 = 5.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const RED: Color32 = Color32::from_rgb(255, 0, 0);

#[derive(Serialize, Deserialize, Debug)]
pub struct HeadlinesConfig {
    pub dark_mode: bool,
    pub api_key: String,
}

impl Default for HeadlinesConfig {
    fn default() -> Self {
        Self {
            dark_mode: Default::default(),
            api_key: String::new(),
        }
    }
}

pub struct Headlines {
    pub articles: Vec<NewsCardData>,
    pub config: HeadlinesConfig,
    pub api_key_initialized: bool,
    pub data_is_set: bool,
    pub initial_is_set: bool,
    pub news_rx: Option<Receiver<NewsCardData>>,
}

#[derive(Debug)]
pub struct NewsCardData {
    pub title: String,
    pub desc: String,
    pub url: String,
}

impl Headlines {
    pub fn new() -> Headlines {
        Headlines {
            api_key_initialized: Default::default(),
            articles: vec![],
            config: Default::default(),
            news_rx: None,
            data_is_set: false,
            initial_is_set: false,
        }
    }

    pub fn configure_fonts(&mut self, ctx: &egui::Context) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS".to_owned(),
            FontData::from_static(include_bytes!("../../MesloLGS_NF_Regular.ttf")),
        );
        font_def
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "MesloLGS".to_string());
        ctx.set_fonts(font_def);
    }

    pub fn render_news_cards(&self, ui: &mut eframe::egui::Ui) {
        for a in &self.articles {
            ui.add_space(PADDING);
            let title = format!("▶ {}", a.title);

            if self.config.dark_mode {
                ui.colored_label(WHITE, title);
            } else {
                ui.colored_label(BLACK, title);
            }

            ui.add_space(PADDING);
            let desc =
                Label::new(RichText::new(&a.desc).text_style(eframe::egui::TextStyle::Button));
            ui.add(desc);

            if self.config.dark_mode {
                ui.style_mut().visuals.hyperlink_color = CYAN;
            } else {
                ui.style_mut().visuals.hyperlink_color = RED;
            }

            ui.add_space(PADDING);
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.add(Hyperlink::from_label_and_url("read more ⤴", &a.url));
            });
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }

    pub fn render_top_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.add(Label::new(
                        RichText::new("📓").text_style(eframe::egui::TextStyle::Heading),
                    ));
                });

                ui.with_layout(Layout::right_to_left(Align::Min), move |ui| {
                    let refresh_btn = ui.add(Button::new("🔄"));

                    if refresh_btn.clicked() {
                        self.refresh_data();
                    }

                    let theme_btn = ui.add(Button::new({
                        if self.config.dark_mode {
                            "🌞"
                        } else {
                            "🌙"
                        }
                    }));
                    if theme_btn.clicked() {
                        self.config.dark_mode = !self.config.dark_mode;
                    }
                });

                ui.add_space(10.);
            });

            ui.add_space(10.);
        });
    }

    pub fn render_config(&mut self, ctx: &egui::Context) {
        Window::new("Configuration").show(ctx, |ui| {
            ui.label("Enter your API KEY for newsapi.org");
            let text_input = ui.text_edit_singleline(&mut self.config.api_key);
            if self.config.api_key.len() == 32
                && text_input.lost_focus()
                && ui.input().key_pressed(egui::Key::Enter)
            {
                self.api_key_initialized = true;
            }
            ui.label("If you havn't registered forr the API_KEY, head over to");
            ui.hyperlink("https://newsapi.org");
        });
    }

    pub fn load_data(&mut self) {
        if !self.data_is_set && !self.config.api_key.is_empty() && self.config.api_key.len() == 32 {
            let api_key = &self.config.api_key;

            let api_key = api_key.to_string();

            let (news_tx, news_rx) = channel();

            self.news_rx = Some(news_rx);

            #[cfg(not(target_arch = "wasm32"))]
            let response = NewsAPI::new(&api_key)
                .fetch()
                .expect("Failed to load articles");

            #[cfg(not(target_arch = "wasm32"))]
            thread::spawn(move || {
                let resp_articles = response.articles();
                for a in resp_articles.iter() {
                    let news = NewsCardData {
                        title: a.title().to_string(),
                        url: a.url().to_string(),
                        desc: a.desc().map(|s| s.to_string()).unwrap_or("...".to_string()),
                    };

                    if let Err(e) = news_tx.send(news) {
                        tracing::error!("Error sending news data: {}", e);
                    }
                }
            });

            #[cfg(target_arch = "wasm32")]
            gloo_timers::callback::Timeout::new(10, move || {
                wasm_bindgen_futures::spawn_local(async move {
                    Self::fetch_web(api_key, news_tx).await;
                });
            })
            .forget();

            self.data_is_set = true;
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn fetch_web(api_key: String, news_tx: std::sync::mpsc::Sender<NewsCardData>) {
        let response: NewsAPIResponse = NewsAPI::new(&api_key)
            .fetch_web()
            .await
            .expect("Failed to load articles");
        let resp_articles = response.articles();
        for a in resp_articles.iter() {
            let news = NewsCardData {
                title: a.title().to_string(),
                url: a.url().to_string(),
                desc: a.desc().map(|s| s.to_string()).unwrap_or("...".to_string()),
            };

            if let Err(e) = news_tx.send(news) {
                tracing::error!("Error sending news data: {}", e);
            }
        }
    }

    pub fn refresh_data(&mut self) {
        self.data_is_set = false;
        self.articles = vec![];
        self.load_data();
    }

    pub fn preload_articles(&mut self) {
        if let Some(rx) = &self.news_rx {
            match rx.try_recv() {
                Ok(news_data) => {
                    self.articles.push(news_data);
                }
                Err(e) => match e {
                    TryRecvError::Empty => {}
                    TryRecvError::Disconnected => {
                        self.news_rx = None;
                    }
                },
            }
        }
    }
}
