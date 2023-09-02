use serde::{Serialize, Deserialize};
use std::{fs, time::Instant};
use std::time::Duration;
use clokwerk::{Scheduler, TimeUnits};
use betweenworlds_api::LeaderboardsFlags;
extern crate chrono;
use chrono::{DateTime, TimeZone, LocalResult, Local};
use num_format::{Locale, ToFormattedString};


const DATA_DIR: &str = "appdata";
const STATE_PATH: &str = "./appdata/state.json";
const TRACKERS_DIR: &str = "./appdata/trackers";

fn main() {
    let native_options = eframe::NativeOptions::default();

    let mut scheduler = Scheduler::new();
    scheduler.every(30.minutes()).run(update_records);
    update_records();
    let thread_handle = scheduler.watch_thread(Duration::from_secs(60));

    eframe::run_native(
        "Player Tracking",
        native_options,
        Box::new(|cc| Box::new(PlayerTracker::new(cc))),
    ).unwrap();
    thread_handle.stop();
}

struct PlayerTracker {
    state: TrackerState,
    current_name: String,
    selected: String,
    selected_graph: Graph,
}

impl PlayerTracker {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let state = if let Ok(json) = fs::read_to_string(STATE_PATH) {
            serde_json::from_str(&json).unwrap()
        }
        else {
            TrackerState::default()
        };
        Self { state, current_name: String::new(), selected: String::new(), selected_graph: Graph::Level }
    }

    fn update_credentials(&mut self, ui: &mut egui::Ui) {
        ui.heading("Credentials");
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Auth id").on_hover_text("Your username");
            changed = ui.text_edit_singleline(&mut self.state.auth_id).changed();
        });
        ui.horizontal(|ui| {
            ui.label("Api key")
                .on_hover_text("You can get it in the account settings.");
            let text_edit = egui::TextEdit::singleline(&mut self.state.api_key).password(true);
            if ui.add(text_edit).changed() {
                changed = true;
            }
        });

        if changed {
            self.save_state();
        }
    }

    fn update_graph(&self, ui: &mut egui::Ui, reset_graph: bool) {
        if !self.selected.is_empty() {
            let record_path = format!("{TRACKERS_DIR}/{}.json", self.selected);
            if let Ok(json) = fs::read_to_string(record_path) {
                
                
                let record = serde_json::from_str::<PlayerRecord>(&json).unwrap();
                let first = record.records[0].time.timestamp();
                let iter = record.records.iter();
                let line: egui::plot::PlotPoints = match self.selected_graph {
                    Graph::Level => iter.map(|record| [(record.time.timestamp() - first) as f64 / 60.0, record.level as f64]).collect(),
                    Graph::Credits => iter.map(|record| [(record.time.timestamp() - first) as f64 / 60.0, record.credits as f64]).collect(),
                    Graph::MissionsCompleted => iter.map(|record| [(record.time.timestamp() - first) as f64 / 60.0, record.missions_completed as f64]).collect(),
                    Graph::Overdoses => iter.map(|record| [(record.time.timestamp() - first) as f64 / 60.0, record.overdoses as f64]).collect(),
                    Graph::CombatsWon => iter.map(|record| [(record.time.timestamp() - first) as f64 / 60.0, record.combats_won as f64]).collect(),
                    Graph::ItemsCrafted => iter.map(|record| [(record.time.timestamp() - first) as f64 / 60.0, record.items_crafted as f64]).collect(),
                    Graph::JobsPerformed => iter.map(|record| [(record.time.timestamp() - first) as f64 / 60.0, record.jobs_performed as f64]).collect(),
                };

                let line = egui::plot::Line::new(line);
                let graph_name = self.selected_graph.name();
                let plot = egui::plot::Plot::new("my_plot")
                    .auto_bounds_x()
                    .auto_bounds_y()
                    .allow_drag(egui::widgets::plot::AxisBools{x: true, y: false})
                    .allow_boxed_zoom(false)
                    .set_margin_fraction(egui::vec2(0.0, 0.3))
                    .allow_zoom(egui::widgets::plot::AxisBools{x: true, y: false})
                    .label_formatter(move |_name, value| {
                        if let LocalResult::Single(time) = Local.timestamp_opt((value.x * 60.0) as i64 + first, 0) {
                            let value = value.y as i64;
                            format!("time: {}\n{}: {}", time.format("%d/%m/%Y %R"), graph_name, value.to_formatted_string(&Locale::en))
                        }
                        else {
                            format!("time {}: {}", value.x, value.y)
                        }
                    })
                    .allow_scroll(false);
                    
                    if reset_graph {
                        plot.reset()
                    }
                    else {
                        plot
                    }.show(ui, |plot_ui| plot_ui.line(line) );
            }
        }
    }

    fn save_state(&self) {
        let _  = fs::create_dir_all(DATA_DIR);
        fs::write(STATE_PATH, serde_json::to_string(&self.state).unwrap().as_bytes()).unwrap();
    }
}

impl eframe::App for PlayerTracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |mut ui| {
            self.update_credentials(&mut ui);
            ui.separator();
            ui.allocate_ui_with_layout(ui.available_size(), egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                let mut selected_player_changed = false;
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width() * 0.2, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {

                        ui.horizontal(|ui| {
                            let input_lost_focus = ui.text_edit_singleline(&mut self.current_name).lost_focus();
                            let mut add = input_lost_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
                            if ui.button("Add").clicked() {
                                add = true;
                            }

                            if add && !self.current_name.is_empty() {
                                if self.state.trackers.iter().all(|name| *name != self.current_name) {
                                    self.state.trackers.push(self.current_name.drain(..).collect());
                                    self.save_state();
                                }
                                else {
                                    // TODO: Report an error
                                }
                            }
                        }); 
                        ui.add_space(5.0);

                        let weak_bg_fill =  ui.visuals().widgets.inactive.weak_bg_fill;
                        for tracker in &self.state.trackers {
                            if self.selected == *tracker {
                                ui.visuals_mut().widgets.inactive.weak_bg_fill = weak_bg_fill;
                            }
                            else {
                                ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                            }
                            let button = egui::Button::new(tracker)
                                .stroke(egui::Stroke::NONE)
                                .rounding(egui::Rounding::same(3.0))
                                .min_size(egui::vec2(ui.available_width(), 0.0));
                            if ui.add(button).clicked() {
                                self.selected = tracker.clone();
                                selected_player_changed = true;
                            }
                        }
                });
                ui.separator();
                ui.vertical(|ui| {
                    let mut changed = false;
                    ui.horizontal_wrapped(|ui| {
                        if ui.radio_value(&mut self.selected_graph, Graph::Level, "Level").changed() {
                            changed = true;
                        }
                        if ui.radio_value(&mut self.selected_graph, Graph::Credits, "Credits").changed() {
                            changed = true;
                        }
                        if ui.radio_value(&mut self.selected_graph, Graph::MissionsCompleted, "Missions completed").changed() {
                            changed = true;
                        }
                        if ui.radio_value(&mut self.selected_graph, Graph::Overdoses, "Overdoses").changed() {
                            changed = true;
                        }
                        if ui.radio_value(&mut self.selected_graph, Graph::CombatsWon, "Combats won").changed() {
                            changed = true;
                        }
                        if ui.radio_value(&mut self.selected_graph, Graph::ItemsCrafted, "Items crafted").changed() {
                            changed = true;
                        }
                        if ui.radio_value(&mut self.selected_graph, Graph::JobsPerformed, "Jobs performed").changed() {
                            changed = true;
                        }
                    });
                    self.update_graph(ui, changed || selected_player_changed);
                });
                
            });
        });
    }
}

#[derive(PartialEq)]
enum Graph {
    Level,
    Credits,
    MissionsCompleted,
    Overdoses,
    CombatsWon,
    ItemsCrafted,
    JobsPerformed,
}

impl Graph {
    fn name(&self) -> &'static str {
        match self {
            Graph::Level => "level",
            Graph::Credits => "credits",
            Graph::MissionsCompleted => "missions completed",
            Graph::Overdoses => "overdoses",
            Graph::CombatsWon => "combats won",
            Graph::ItemsCrafted => "items crafted",
            Graph::JobsPerformed => "jobs performed",
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct TrackerState {
    auth_id: String,
    api_key: String,
    trackers: Vec<String>
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct PlayerRecord {
    records: Vec<LeaderboardsRecord>
}

#[derive(Debug, Serialize, Deserialize)]
struct LeaderboardsRecord {
    time: DateTime<Local>,
    credits: usize,
    level: usize,
    overdoses: usize,
    combats_won: usize,
    items_crafted: usize,
    jobs_performed: usize,
    missions_completed: usize
}

fn update_records() {
    // TODO: make this job work offline without the app open
    let state: TrackerState = if let Ok(json) = fs::read_to_string(STATE_PATH) {
        serde_json::from_str(&json).unwrap()
    }
    else {
        return;
    };

    println!("running job");
    let start = Instant::now();
    let client = betweenworlds_api::Client::new(state.auth_id, state.api_key);
    let _ = fs::create_dir_all(TRACKERS_DIR);
    for player in state.trackers {
        let record_path = format!("{TRACKERS_DIR}/{player}.json");
        let mut record = if let Ok(json) = fs::read_to_string(&record_path) {
            serde_json::from_str(&json).unwrap()
        }
        else {
            PlayerRecord::default()
        };

        println!("player: {player}");
        let user = match client.get_leaderboard_user(&player, LeaderboardsFlags::all()) {
            Ok(user) => user,
            Err(_) => continue,
        };


        let now = Local::now();

        let leaderboards_record = LeaderboardsRecord {
            time: now,
            credits: user.credits.unwrap().credits,
            level: user.highest_levels.unwrap().level,
            overdoses: user.overdoses.unwrap().overdoses,
            combats_won: user.combats_won.unwrap().combats_won,
            items_crafted: user.items_crafted.unwrap().items_crafted,
            jobs_performed: user.jobs_performed.unwrap().jobs_performed,
            missions_completed: user.missions_completed.unwrap().missions_completed,
        };

        record.records.push(leaderboards_record);

        fs::write(&record_path, serde_json::to_string(&record).unwrap().as_bytes()).unwrap();
    }
    let duration = start.elapsed();
    println!("done. took {:?}", duration);
}
