use async_std::task;
use egui::{Color32, Context};
use ff8_battle_structure_gui::library::{
    battle_names::{ENEMY_NAMES, STAGE_NAMES},
    battle_structure::{BattleStructure, Enemy, PackedBattleStructure},
};
use rfd::AsyncFileDialog;
use std::{
    future::Future,
    sync::mpsc::{channel, Receiver, Sender},
};

const BATTLE_STRUCTURE_NUMBER: usize = 1024;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "FFVIII Battle Structure Editor",
        native_options,
        Box::new(|cc| Ok(Box::new(BattleStructureApp::new(cc)))),
    )
}

pub struct BattleStructureApp {
    file_bytes_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    battle_structure_list: Vec<BattleStructure>,
    battle_structure_index: usize,
    enemy_selected_index: usize,
}

impl BattleStructureApp {
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

impl eframe::App for BattleStructureApp {
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

                        if ui.button("Save as...").clicked() {
                            let task = rfd::AsyncFileDialog::new().save_file();
                            match write_packed_battle_structure(&self.battle_structure_list) {
                                Ok(contents) => {
                                    execute(async move {
                                        let file = task.await;
                                        if let Some(file) = file {
                                            _ = file.write(contents.as_ref()).await;
                                        }
                                    });
                                }
                                Err(_) => todo!()
                            };
                            ui.close_menu();
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
                        frame().show(ui, |ui| stage_contents(ui, battle_structure));
                        frame().show(ui, |ui| {
                            enemies_contents(ui, battle_structure, &mut self.enemy_selected_index)
                        });
                    }
                    None => todo!(),
                }
            }
        });
    }
}

fn stage_contents(ui: &mut egui::Ui, battle_structure: &mut BattleStructure) {
    egui::ComboBox::from_label("Battle stage")
        .selected_text(
            STAGE_NAMES
                .get(battle_structure.stage_id as usize)
                .unwrap_or(&"Invalid Stage Id!")
                .to_string(),
        )
        .show_ui(ui, |ui| {
            (0..STAGE_NAMES.len()).for_each(|i| {
                ui.selectable_value(&mut battle_structure.stage_id, i as u8, STAGE_NAMES[i]);
            });
        });

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
        egui::Slider::new(&mut battle_structure.secondary_camera.animation, 0..=7)
            .text("Secondary camera animation"),
    );
}

fn enemies_contents(
    ui: &mut egui::Ui,
    battle_structure: &mut BattleStructure,
    enemy_selected_index: &mut usize,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            for i in 0..battle_structure.enemies.len() {
                let enemy = &battle_structure.enemies[i];
                let enemy_name = format!(
                    "{i}. {}",
                    *ENEMY_NAMES
                        .get(enemy.id as usize)
                        .unwrap_or(&"Invalid enemy name!")
                );

                let enemy_name = if enemy.enabled {
                    enemy_name.to_string()
                } else {
                    enemy_name + " (disabled)"
                };

                let text_color = if enemy.enabled {
                    Color32::PLACEHOLDER
                } else {
                    Color32::DARK_GRAY
                };

                ui.selectable_value(
                    enemy_selected_index,
                    i,
                    egui::RichText::new(enemy_name).color(text_color),
                );
            }
        });

        ui.vertical(
            |ui| match battle_structure.enemies.get_mut(*enemy_selected_index) {
                Some(enemy) => enemy_contents(ui, enemy),
                None => todo!(),
            },
        )
    });
}

fn enemy_contents(ui: &mut egui::Ui, enemy: &mut Enemy) {
    frame().show(ui, |ui| {
        egui::ComboBox::from_label("Enemy")
            .selected_text(
                ENEMY_NAMES
                    .get(enemy.id as usize)
                    .unwrap_or(&"Invalid enemy id!")
                    .to_string(),
            )
            .show_ui(ui, |ui| {
                (0..ENEMY_NAMES.len()).for_each(|i| {
                    ui.selectable_value(&mut enemy.id, i as u8, ENEMY_NAMES[i]);
                });
            });
        ui.add(egui::Slider::new(&mut enemy.level, 0..=255).text("Level"));
        ui.checkbox(&mut enemy.enabled, "Enabled");
        ui.checkbox(&mut enemy.not_loaded, "NOT loaded");
        ui.checkbox(&mut enemy.invisible, "NOT visible");
        ui.checkbox(&mut enemy.untargetable, "NOT targetable");

        ui.add(egui::Slider::new(&mut enemy.coordinate.x, i16::MIN..=i16::MAX).text("X"));
        ui.add(egui::Slider::new(&mut enemy.coordinate.y, i16::MIN..=i16::MAX).text("Y"));
        ui.add(egui::Slider::new(&mut enemy.coordinate.z, i16::MIN..=i16::MAX).text("Z"));

        ui.collapsing("Advanced options", |ui| {
            ui.add(
                egui::Slider::new(&mut enemy.unknown_1, 0..=u16::MAX)
                    .text("Unknown 1")
                    .hexadecimal(1, false, true),
            );
            ui.add(
                egui::Slider::new(&mut enemy.unknown_2, 0..=u16::MAX)
                    .text("Unknown 2")
                    .hexadecimal(1, false, true),
            );
            ui.add(
                egui::Slider::new(&mut enemy.unknown_3, 0..=u16::MAX)
                    .text("Unknown 3")
                    .hexadecimal(1, false, true),
            );
            ui.add(
                egui::Slider::new(&mut enemy.unknown_4, 0..=u8::MAX)
                    .text("Unknown 4")
                    .hexadecimal(1, false, true),
            );
        });
    });
}

fn frame() -> egui::Frame {
    egui::Frame::none()
        .rounding(egui::Rounding::from(4.0))
        .stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
        .inner_margin(8.0)
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

fn write_packed_battle_structure(
    battle_structure_list: &Vec<BattleStructure>,
) -> anyhow::Result<Vec<u8>> {
    let mut bytes: Vec<u8> =
        Vec::with_capacity(BATTLE_STRUCTURE_NUMBER * size_of::<PackedBattleStructure>());
    for battle_structure in battle_structure_list {
        bytes.extend_from_slice(battle_structure.as_packed_bytes()?.as_ref());
    }
    Ok(bytes)
}
