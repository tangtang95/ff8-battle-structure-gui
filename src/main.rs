use iced::{
    executor, font::Family, widget::{button, checkbox, column, pick_list, row, text}, Application, Command, Font, Settings
};
use rfd::AsyncFileDialog;

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
}

impl Application for MyApp {
    type Message = AppMessage;
    type Executor = executor::Default;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self { counter: 0 }, Command::none())
    }

    fn title(&self) -> String {
        String::from("My App")
    }

    fn update(&mut self, message: Self::Message) -> Command::<Self::Message> {
        match message {
            AppMessage::ButtonPressed => {
                let pick_files = AsyncFileDialog::new()
                    .add_filter("text", &["txt", "rs"])
                    .add_filter("rust", &["rs", "toml"])
                    .set_directory("/")
                    .pick_file();
                Command::perform(pick_files, |_| {
                    AppMessage::DoNothing
                })
            },
            AppMessage::DoNothing => Command::none()
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
