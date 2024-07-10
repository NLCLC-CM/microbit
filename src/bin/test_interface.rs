use iced::alignment::{Alignment};
use iced::executor;
use iced::widget::{column, container, keyed_column, scrollable, text, text_input};
use iced::{Application, Command, Element, Settings, Theme};

#[derive(Debug)]
struct Message {
    author: String,
    text: String,
}

impl Message {
    fn view(&self) -> Element<Action> {
        let author = text(&self.author);
        let text = text(&self.text);

        column![author, text,].align_items(Alignment::Start).into()
    }
}

#[derive(Debug)]
struct State {
    input: String,
    messages: Vec<Message>,
}

#[derive(Debug, Clone)]
enum Action {
    InputChanged(String),
    AddMessage,
}

impl Application for State {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Action;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                input: String::new(),
                messages: Vec::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Messaging app")
    }

    fn update(&mut self, action: Self::Message) -> Command<Self::Message> {
        match action {
            Action::InputChanged(value) => {
                self.input = value;

                Command::none()
            }
            Action::AddMessage => {
                self.messages.push(Message {
                    author: String::from("Cheuk Yin Ng"),
                    text: self.input.clone(),
                });
                self.input = String::new();

                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let messages: Element<_> = keyed_column(
            self.messages
                .iter()
                .enumerate()
                .map(|(i, msg)| (i, msg.view())),
        )
        .into();

        let input = text_input("Send a message...", &self.input)
            .on_input(Self::Message::InputChanged)
            .on_submit(Self::Message::AddMessage);

        column![scrollable(container(messages)), input,].into()
    }
}

fn main() -> iced::Result {
    State::run(Settings::default())
}
