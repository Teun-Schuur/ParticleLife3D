use egui::{
    Context, 
    Modifiers, 
    ScrollArea, 
    Ui,
    plot::{
        Line, 
        Plot,
        PlotPoints,
    },
};
use crate::system::stats::StatHistory;

// ----------------------------------------------------------------------------

/// A menu bar in which you can select different demo windows to show.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct GUI {
    plot_is_open: bool,
    plot: EnergyGraph,
}

impl Default for GUI {
    fn default() -> Self {
        Self {
            plot_is_open: true,
            plot: Default::default(),
        }
    }
}

impl GUI {
    /// Show the app ui (menu bar and windows).
    pub fn ui(&mut self, ctx: &Context, data: StatHistory) {
        self.show_windows(ctx, data);
    }

    /// Show the open windows.
    fn show_windows(&mut self, ctx: &Context, data: StatHistory) {
        self.plot.show(ctx, &mut self.plot_is_open, data);
    }
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct EnergyGraph {}

impl EnergyGraph {
    fn name(&self) -> &'static str {
        "Energy Graph"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, data: StatHistory) {
        egui::Window::new(self.name())
            .default_width(320.0)
            .open(open)
            .show(ctx, |ui| {
                self.ui(ui, data);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui, data: StatHistory) {
        ui.heading("Energy graph");

        ui.label("This is a graph of the energy of the system over time.");
        ui.add_space(12.0);
        ui.heading("Some thermodynamic data");
        
        ui.label(format!("Temperature: {}", data.temperature()));
        ui.label(format!("velocity rms: {}", data.velocity_rms()));

        ui.add_space(12.0); // ui.separator();
        ui.heading("Graph");

        let mut plot = Plot::new("Energy");
        let sample_rate = 10usize;

        let mut ke_line = Line::new(PlotPoints::new(
            data.graph_KE(sample_rate),
        ));
        ke_line = ke_line.color(egui::Color32::from_rgb(255, 0, 0));
        ke_line = ke_line.name("KE");
        let mut pe_line = Line::new(PlotPoints::new(
            data.graph_PE(sample_rate),
        ));
        pe_line = pe_line.color(egui::Color32::from_rgb(0, 255, 0));
        pe_line = pe_line.name("PE");

        let mut te_line = Line::new(
            PlotPoints::new(data.graph_TE(sample_rate)),
        );
        te_line = te_line.color(egui::Color32::from_rgb(0, 0, 255));
        te_line = te_line.name("TE");


        plot.show(ui, |ui| {
            ui.line(ke_line);
            ui.line(pe_line);
            ui.line(te_line);
        });

    }
}
