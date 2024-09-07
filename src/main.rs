use async_std::task;
use egui::Context;
use ff8_battle_structure_gui::{BattleStructure, PackedBattleStructure};
use rfd::AsyncFileDialog;
use std::{
    future::Future,
    sync::mpsc::{channel, Receiver, Sender},
};

const BATTLE_STRUCTURE_NUMBER: usize = 1000;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "FF8 Battle Structure App",
        native_options,
        Box::new(|cc| Ok(Box::new(BSApp::new(cc)))),
    )
}

pub struct BSApp {
    file_bytes_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    battle_structure_list: Vec<BattleStructure>,
    battle_structure_index: usize,
    enemy_selected_index: usize,
}

impl BSApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            file_bytes_channel: channel(),
            battle_structure_list: Vec::new(),
            battle_structure_index: 0,
            enemy_selected_index: 0,
        }
    }
}

impl eframe::App for BSApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Ok(bytes) = self.file_bytes_channel.1.try_recv() {
            match read_battle_structures(&bytes) {
                Ok(battle_structure_list) => {
                    self.battle_structure_list = battle_structure_list;
                    self.battle_structure_index = 0;
                    self.enemy_selected_index = 0;
                }
                Err(_) => todo!(),
            }
        }

        egui::TopBottomPanel::top("app_top_bar")
            .frame(egui::Frame::none().inner_margin(4.0))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    egui::widgets::global_dark_light_mode_switch(ui);

                    ui.menu_button("File", |ui| {
                        ui.set_max_width(200.0);

                        if ui.button("Open...").clicked() {
                            let sender = self.file_bytes_channel.0.clone();
                            let task = AsyncFileDialog::new()
                                .add_filter("scene.out", &["out"])
                                .set_directory("/")
                                .pick_file();
                            let ctx = ui.ctx().clone();
                            execute(async move {
                                let file = task.await;
                                if let Some(file) = file {
                                    let bytes = file.read().await;
                                    let _ = sender.send(bytes);
                                    ctx.request_repaint();
                                }
                            });
                            ui.close_menu();
                        }

                        if ui.button("Save").clicked() {
                            // TODO
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.battle_structure_list.is_empty() {
                ui.add(
                    egui::Slider::new(
                        &mut self.battle_structure_index,
                        0..=BATTLE_STRUCTURE_NUMBER - 1,
                    )
                    .text("Encounter ID"),
                );

                match self
                    .battle_structure_list
                    .get_mut(self.battle_structure_index)
                {
                    Some(battle_structure) => {
                        ui.label("stage id: ".to_string() + &battle_structure.stage_id.to_string()); // TODO: fix stage id
                        ui.checkbox(&mut battle_structure.flags.cannot_escape, "Cannot escape");
                        ui.checkbox(&mut battle_structure.flags.no_exp, "No exp gained");
                        ui.checkbox(
                            &mut battle_structure.flags.scripted_battle,
                            "Scripted battle",
                        );
                        ui.checkbox(&mut battle_structure.flags.show_timer, "Show timer");
                        ui.checkbox(
                            &mut battle_structure.flags.force_back_attack,
                            "Force back attack",
                        );
                        ui.checkbox(
                            &mut battle_structure.flags.force_surprise_attack,
                            "Force surprise attack",
                        );
                        ui.checkbox(
                            &mut battle_structure.flags.disable_win_fanfare,
                            "Disable victory fanfare",
                        );
                        ui.checkbox(
                            &mut battle_structure.flags.disable_exp_screen,
                            "Do not show exp. screen",
                        );
                        ui.add(
                            egui::Slider::new(&mut battle_structure.main_camera.number, 0..=3)
                                .text("Main camera number"),
                        );
                        ui.add(
                            egui::Slider::new(&mut battle_structure.main_camera.animation, 0..=7)
                                .text("Main camera animation"),
                        );
                        ui.add(
                            egui::Slider::new(&mut battle_structure.secondary_camera.number, 0..=3)
                                .text("Secondary camera number"),
                        );
                        ui.add(
                            egui::Slider::new(
                                &mut battle_structure.secondary_camera.animation,
                                0..=7,
                            )
                            .text("Secondary camera animation"),
                        );
                        ui.vertical(|ui| {
                            for i in 0..8 {
                                ui.selectable_value(
                                    &mut self.enemy_selected_index,
                                    i,
                                    format!("Enemy {i}"),
                                );
                            }
                        });

                        match battle_structure.enemies.get_mut(self.enemy_selected_index) {
                            Some(enemy) => {
                                ui.add(egui::Slider::new(&mut enemy.level, 0..=255).text("Level"));
                                ui.label(format!("Enemy id: {}", enemy.id)); // TODO: fix enemy id
                                ui.checkbox(&mut enemy.enabled, "Enabled");
                                ui.checkbox(&mut enemy.not_loaded, "NOT loaded");
                                ui.checkbox(&mut enemy.invisible, "NOT visible");
                                ui.checkbox(&mut enemy.untargetable, "NOT targetable");

                                ui.add(
                                    egui::Slider::new(&mut enemy.coordinate.x, i16::MIN..=i16::MAX)
                                        .text("X"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut enemy.coordinate.y, i16::MIN..=i16::MAX)
                                        .text("Y"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut enemy.coordinate.z, i16::MIN..=i16::MAX)
                                        .text("Z"),
                                );

                                ui.add(
                                    egui::Slider::new(&mut enemy.unknown_1, 0..=u16::MAX)
                                        .text("Unknown 1").hexadecimal(1, false, true),
                                );
                                ui.add(
                                    egui::Slider::new(&mut enemy.unknown_2, 0..=u16::MAX)
                                        .text("Unknown 2").hexadecimal(1, false, true),
                                );
                                ui.add(
                                    egui::Slider::new(&mut enemy.unknown_3, 0..=u16::MAX)
                                        .text("Unknown 3").hexadecimal(1, false, true),
                                );
                                ui.add(
                                    egui::Slider::new(&mut enemy.unknown_4, 0..=u8::MAX)
                                        .text("Unknown 4").hexadecimal(1, false, true),
                                );
                            }
                            None => todo!(),
                        }
                    }
                    None => todo!(),
                }
            }
        });
    }
}

fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    task::spawn(f);
}

fn read_battle_structures(bytes: &[u8]) -> anyhow::Result<Vec<BattleStructure>> {
    let mut battle_structure_list = Vec::with_capacity(BATTLE_STRUCTURE_NUMBER);
    for i in 0..BATTLE_STRUCTURE_NUMBER {
        let offset = i * size_of::<PackedBattleStructure>();
        let packed_bs = PackedBattleStructure::try_from_bytes(
            &bytes[offset..offset + size_of::<PackedBattleStructure>()],
        )?;
        battle_structure_list.push(packed_bs.into_battle_structure());
    }
    Ok(battle_structure_list)
}
