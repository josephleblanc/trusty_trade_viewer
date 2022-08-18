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
    #[serde(skip)]
    change_box_points_by: usize,
    #[serde(skip)]
    show_bollinger: bool,
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

impl Data {
    fn tp(&self) -> f64 {
        (self.high + self.low + self.close) as f64 / 3.0_f64
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            box_plot_points: 100,
            change_box_points_by: 5,
            show_bollinger: false,
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
            change_box_points_by,
            show_bollinger,
        } = self;

        let data = include_bytes!("/home/brasides/programming/data/BTC_historic_minute/master/2022-07-21_to_2022-08-17_15:13:00.csv");
        let data: Vec<Data> = read_data(data, *box_plot_points);

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
            use egui::{FontId, RichText};
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });
            ui.label(
                RichText::new("Increase/Decrease data points by").font(FontId::proportional(16.0)),
            );
            ui.add(egui::Slider::new(change_box_points_by, 1..=50));
            ui.horizontal(|ui| {
                if ui.button("Add data points").clicked() {
                    match (*box_plot_points).checked_add(*change_box_points_by) {
                        Some(number) => *box_plot_points = number,
                        None => *box_plot_points = data.len(),
                    }
                }
                if ui.button("Subtract data points").clicked() {
                    match (*box_plot_points).checked_sub(*change_box_points_by) {
                        Some(number) => *box_plot_points = number,
                        None => *box_plot_points = 0,
                    }
                }
            });

            ui.label(RichText::new("Show Indicators").font(FontId::proportional(16.0)));
            ui.checkbox(show_bollinger, "Bollinger Bands");

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
            draw_multiplot(ui, boxplot_from_data(data));
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

fn read_data(data: &[u8], box_plot_points: usize) -> Vec<Data> {
    use csv::Reader;
    let mut rdr = Reader::from_reader(data);
    let iter = rdr.deserialize();
    iter.zip(0..box_plot_points).map(|(d, i)| d.unwrap()).collect()
}

fn boxplot_from_data(data: Vec<Data>) -> egui::plot::BoxPlot {
    //use csv::Reader;
    use egui::plot::{BoxElem, BoxPlot, BoxSpread, Line, Plot, Value, Values};

    //let mut tp_vec: Vec<[f64; 2]> = vec![[0_f64, data[0].tp()]];
    //for i in 1..data.len() {
    //    tp_vec.push([i as f64, data[i].tp()]);
    //}
    //let tp_line = Line::new(Values::from_values_iter(
    //    tp_vec.iter().map(|[x, y]| Value::new(*x, *y)),
    //));
    let box_elems: Vec<BoxElem> = //rdr2
        data.iter().zip(data.iter().skip(1))
        .map(|(d_last, d)| 
            (
                BoxSpread {
                    lower_whisker: d.low as f64,
                    quartile1: d.open.min(d.close) as f64,
                    median: ((d.high + d.low + d.close) / 3.0_f32) as f64,
                    quartile3: d.open.max(d.close) as f64,
                    upper_whisker: d.high as f64,
                },
                match d.close >= d_last.close {
                    true => egui::Color32::GREEN,
                    false => egui::Color32::RED,
                }
            )
        )
        .enumerate()
        .map(|(i, (box_spread, color))| {
            BoxElem::new(i as f64, box_spread)
                .fill(color)
                .stroke(egui::Stroke::new(0.2_f32, color))
        })
        .collect();
    BoxPlot::new(box_elems)
    //Plot::new("box_plot")
    //    .view_aspect(2.0)
    //    .data_aspect(0.1)
    //    .show(ui, |plot_ui| {
    //        plot_ui.box_plot(boxes);
    //        // plot_ui.line(tp_line);
    //    })
    //    .response
}

fn draw_multiplot(ui: &mut egui::Ui, boxplot: egui::plot::BoxPlot) -> egui::Response {
    use egui::plot::Plot;
    Plot::new("box_plot")
        .view_aspect(2.0)
        .data_aspect(0.1)
        .show(ui, |plot_ui| {
            plot_ui.box_plot(boxplot);
            // plot_ui.line(tp_line);
        })
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
