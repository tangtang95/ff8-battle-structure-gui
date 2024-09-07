use ff8_battle_structure_gui::{BattleStructure, PackedBattleStructure};
use iced::{
    executor,
    font::Family,
    widget::{button, checkbox, column, pick_list, row, text},
    Application, Command, Font, Settings,
};
use rfd::AsyncFileDialog;

const BATTLE_STRUCTURE_NUMBER: usize = 1000;

fn main() -> iced::Result {
    MyApp::run(Settings::default())
}

#[derive(Debug, Clone)]
enum AppMessage {
    ButtonPressed,
    DoNothing,
}

struct MyApp {
    counter: usize,
    battle_structure: Option<[BattleStructure; BATTLE_STRUCTURE_NUMBER]>,
}

impl Application for MyApp {
    type Message = AppMessage;
    type Executor = executor::Default;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self { counter: 0, battle_structure: None }, Command::none())
    }

    fn title(&self) -> String {
        String::from("My App")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::ButtonPressed => {
                Command::perform(read_scene_out_file(), |battle_structure_list| {
                    match battle_structure_list {
                        Ok(battle_structure_list) => AppMessage::DoNothing,
                        Err(_) => AppMessage::DoNothing,
                    }
                })
            }
            AppMessage::DoNothing => Command::none(),
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let counter_text = text("Counter").font(Font {
            family: Family::Fantasy,
            ..Font::DEFAULT
        });

        column![
            counter_text,
            checkbox("Construct from function", false),
            pick_list(
                vec!["Special character 😊", "Another choice"],
                Some("Special character 😊"),
                |_| AppMessage::DoNothing
            ),
            text(self.counter),
            row![button("Increase").on_press(AppMessage::ButtonPressed),],
        ]
        .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

async fn read_scene_out_file() -> anyhow::Result<Vec<BattleStructure>> {
    let file_handle = AsyncFileDialog::new()
        .add_filter("text", &["txt", "rs"])
        .add_filter("rust", &["rs", "toml"])
        .set_directory("/")
        .pick_file()
        .await
        .ok_or(anyhow::anyhow!("File not found"))?;

    let bytes = file_handle.read().await;
    let mut battle_structure_list = Vec::with_capacity(BATTLE_STRUCTURE_NUMBER);
    for i in 0..BATTLE_STRUCTURE_NUMBER {
        let offset = i * size_of::<PackedBattleStructure>();
        let packed_bs = PackedBattleStructure::try_from_bytes(&bytes[offset..offset + size_of::<PackedBattleStructure>()])?;
        battle_structure_list.push(packed_bs.into_battle_structure());
    }
    Ok(battle_structure_list)
}
