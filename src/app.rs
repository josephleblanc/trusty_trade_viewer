/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state

pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    _value: f32,
    #[serde(skip)]
    box_plot_points: usize,
    #[serde(skip)]
    change_box_points_by: usize,
    #[serde(skip)]
    show_bollinger: bool,
    #[serde(skip)]
    show_tp_line: bool,
    #[serde(skip)]
    show_moving_average: bool,
    #[serde(skip)]
    moving_average_size: usize,
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
            _value: 2.7,
            box_plot_points: 100,
            change_box_points_by: 5,
            show_bollinger: false,
            show_tp_line: false,
            show_moving_average: false,
            moving_average_size: 20,
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            label,
            _value,
            box_plot_points,
            change_box_points_by,
            show_bollinger,
            show_tp_line,
            show_moving_average,
            moving_average_size,
        } = self;
        // Examples of how to create different panels and windows.

        // This is where to put things which are needed for different
        // calculations, making it a bad idea to toggle them. The variables in
        // this area should be kept as limited as possible, to limit memory
        // bloat. It should also only use datapoints in the range of
        // box_plot_points.
        let data = include_bytes!(
            r#"/home/brasides/programming/data/BTC_historic_minute/master/2022-08-15_to_2022-08-22_21:55:00.csv"#
        );
        let data: Vec<Data> = read_data(data, *box_plot_points);
        let tp_vec: Vec<f64> = data.iter().map(|d| d.tp()).collect();

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
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

            // Checkboxes
            // These toggle whether to show the indicator on the plot. Ideally
            // this means that they will not be calculated if the box is not
            // ticked.
            ui.label(RichText::new("Show Indicators").font(FontId::proportional(16.0)));
            ui.checkbox(show_bollinger, "Bollinger Bands");
            ui.checkbox(show_tp_line, "Typical Price Line");
            ui.checkbox(show_moving_average, "Simple Moving Average");
            egui::ComboBox::from_label("Simple Moving Average Size")
                .selected_text(format!("{:?}", moving_average_size))
                .show_ui(ui, |ui| {
                    ui.selectable_value(moving_average_size, 20_usize, "20");
                    ui.selectable_value(moving_average_size, 50, "50");
                    ui.selectable_value(moving_average_size, 200, "200");
                });

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

            let simple_lines: Vec<Option<egui::plot::Line>> = vec![
                tp_line(&tp_vec, show_tp_line),
                sma_line(&tp_vec, *moving_average_size, *show_moving_average),
            ];

            draw_multiplot(ui, boxplot_from_data(data), simple_lines);
            ui.end_row();
            ui.label(format!("size of dataset used: {}", box_plot_points));
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
    iter.zip(0..box_plot_points)
        .map(|(d, _)| d.unwrap())
        .collect()
}

// Make a boxplot to be used in the draw_multiplot function.
// This function takes a Vec of the Data struct as input and creates a BoxPlot
// that looks like the standard candlestick charts. In order for the candle to
// be colored, it must be compared to the previous candle to know if it is green
// or red. Currently this results in a boxplot of size n-1 from a Data input of
// n size, where the first data point is discarded.
fn boxplot_from_data(data: Vec<Data>) -> egui::plot::BoxPlot {
    use egui::plot::{BoxElem, BoxPlot, BoxSpread};

    let first_box: BoxElem = BoxElem::new(
        0.0_f64,
        BoxSpread {
            lower_whisker: data[0].low as f64,
            quartile1: data[0].open.min(data[0].close) as f64,
            median: ((data[0].high + data[0].low + data[0].close) / 3.0_f32) as f64,
            quartile3: data[0].open.max(data[0].close) as f64,
            upper_whisker: data[0].high as f64,
        },
    )
    .fill(egui::Color32::GRAY)
    .stroke(egui::Stroke::new(0.2_f32, egui::Color32::GRAY));

    let mut box_elems: Vec<BoxElem> = //rdr2
        data.iter().zip(data.iter().skip(1))
        .enumerate()
        .map(|(i, (d_last, d))|
            (
                i + 1,
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
        .map(|(i, box_spread, color)| {
            BoxElem::new(i as f64, box_spread)
                .fill(color)
                .stroke(egui::Stroke::new(0.2_f32, color))
        })
        .collect();

    box_elems.insert(0, first_box);
    BoxPlot::new(box_elems)
}

//
fn tp_line(tp_vec: &[f64], show_tp_line: &bool) -> Option<egui::plot::Line> {
    use egui::plot::{Line, PlotPoints};
    match show_tp_line {
        true => Some(Line::new(PlotPoints::from_iter(
            tp_vec.iter().enumerate().map(|(x, y)| [x as f64, *y]),
        ))),
        false => None,
    }
}

fn sma_line(
    tp_vec: &[f64],
    moving_average_size: usize,
    show_moving_average: bool,
) -> Option<egui::plot::Line> {
    use egui::plot::{Line, PlotPoints};
    if show_moving_average {
        let sma_vec = rustatistics::rolling_mean(tp_vec, moving_average_size);
        let sma_values = sma_vec
            .iter()
            .enumerate()
            .filter(|(_, sma)| sma.is_some())
            .map(|(i, sma)| [i as f64, sma.unwrap()]);
        Some(Line::new(PlotPoints::from_iter(sma_values)))
    } else {
        None
    }
}

fn draw_multiplot(
    ui: &mut egui::Ui,
    boxplot: egui::plot::BoxPlot,
    simple_lines: Vec<Option<egui::plot::Line>>,
) -> egui::Response {
    use egui::plot::Plot;
    Plot::new("box_plot")
        .view_aspect(2.0)
        .data_aspect(0.1)
        .show(ui, |plot_ui| {
            plot_ui.box_plot(boxplot);
            for line in simple_lines.into_iter().flatten() {
                plot_ui.line(line);
            }
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
