/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,
    #[serde(skip)]
    box_plot_points: usize,
}

#[allow(non_snake_case)]
#[derive(serde::Deserialize, Debug, Default, Clone)]
pub struct Data {
    pub time: u64,
    pub high: f32,
    pub low: f32,
    pub open: f32,
    pub volumefrom: f32,
    pub volumeto: f32,
    pub close: f32,
    pub conversionType: String,
    pub conversionSymbol: Option<String>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            box_plot_points: 100,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            label,
            value,
            box_plot_points,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }
            if ui.button("Decrement").clicked() {
                *value -= 1.0;
            }
            let data = include_str!("/home/brasides/programming/data/BTC_historic_minute/master/2022-07-21_to_2022-08-17_15:13:00.csv");
            ui.add(egui::Slider::new(box_plot_points, 1..=data.len()).text("Data points"));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            ui.add(doc_link_label("Box Plot", "box plot"));
            example_boxplot(ui, box_plot_points);
            ui.end_row();
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

fn example_boxplot(ui: &mut egui::Ui, box_plot_points: &usize) -> egui::Response {
    use csv::Reader;
    use egui::plot::{BoxElem, BoxPlot, BoxSpread, Plot};
    let data = include_str!("/home/brasides/programming/data/BTC_historic_minute/master/2022-07-21_to_2022-08-17_15:13:00.csv");
    let mut rdr1 = Reader::from_reader(data.as_bytes());
    let mut box_color_vec: Vec<egui::Color32> = vec![egui::Color32::GRAY];
    let data1: Vec<Data> = rdr1.deserialize().map(|d| d.unwrap()).collect();
    for i in 1..*box_plot_points {
        match data1[i].close >= data1[i - 1].close {
            true => box_color_vec.push(egui::Color32::GREEN),
            false => box_color_vec.push(egui::Color32::RED),
        }
    }
    let mut rdr2 = Reader::from_reader(data.as_bytes());
    let box_elems: Vec<BoxElem> = rdr2
        .deserialize::<Data>()
        .zip((0..*box_plot_points).into_iter())
        .map(|(d, i)| {
            let row = d.unwrap();
            (
                i as f64,
                BoxSpread {
                    lower_whisker: row.low as f64,
                    quartile1: row.open.min(row.close) as f64,
                    median: ((row.high + row.low + row.close) / 3.0_f32) as f64,
                    quartile3: row.open.max(row.close) as f64,
                    upper_whisker: row.high as f64,
                },
            )
        })
        .zip(box_color_vec)
        .map(|((i, box_spread), color)| {
            BoxElem::new(i, box_spread)
                .fill(color)
                .stroke(egui::Stroke::new(0.2_f32, color))
        })
        .collect();
    let boxes = BoxPlot::new(box_elems);
    Plot::new("box_plot")
        .view_aspect(2.0)
        .data_aspect(0.1)
        .show(ui, |plot_ui| plot_ui.box_plot(boxes))
        .response
}

fn doc_link_label<'a>(title: &'a str, search_term: &'a str) -> impl egui::Widget + 'a {
    let label = format!("{}:", title);
    let url = format!("https://docs.rs/egui?search={}", search_term);
    move |ui: &mut egui::Ui| {
        ui.hyperlink_to(label, url).on_hover_ui(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Search egui docs for");
                ui.code(search_term);
            });
        })
    }
}
