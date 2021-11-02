use crate::api::{self, Api};
use crate::{display_message, Message};
use iced::{
    button, text_input, Align, Button, Column, Command, Container, Element, HorizontalAlignment,
    Length, Row, Text, TextInput,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Password,
    Tokens,
}

#[derive(Debug, Clone)]
pub struct Settings {
    api: Api,
    tokens: Option<Vec<String>>,
    page: Option<Page>,
    back_btn: button::State,
    goto_tokens_btn: button::State,
    goto_password_btn: button::State,
    logout_btn: button::State,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    // Init,
    Tokens(Vec<String>),
    GoTo(Option<Page>),
}

impl Settings {
    pub fn new(api: Api) -> Self {
        Self {
            api,
            tokens: None,
            page: None,
            back_btn: button::State::default(),
            goto_tokens_btn: button::State::default(),
            goto_password_btn: button::State::default(),
            logout_btn: button::State::default(),
        }
    }

    pub fn update(&mut self, msg: SettingsMessage) -> Command<Message> {
        println!("received a msg: {:?}", msg);
        match msg {
            SettingsMessage::Tokens(t) => self.tokens = Some(t),
            SettingsMessage::GoTo(p) => {
                self.page = p.clone();
                if p.as_ref().map_or(false, |p| p == &Page::Tokens) {
                    let mut api = self.api.clone();
                    return Command::perform(
                        async move { api.get_refresh_tokens().await },
                        |res| {
                            Message::Settings(SettingsMessage::Tokens(match res {
                                Err(e) => vec![format!("{:?}", e)],
                                Ok(t) => t,
                            }))
                        },
                    );
                }
            }
        };
        Command::none()
    }

    pub fn display(&mut self) -> Element<Message> {
        let title: Element<'_, SettingsMessage> = match self.page {
            None => Text::new("Settings")
                .width(Length::Fill)
                .size(100)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(HorizontalAlignment::Center)
                .into(),
            Some(Page::Tokens) => Row::new()
                .spacing(10)
                .push(
                    Button::new(&mut self.back_btn, Text::new("Back"))
                        .on_press(SettingsMessage::GoTo(None)),
                )
                .push(display_message("Refresh tokens"))
                .into(),
            Some(Page::Password) => Row::new()
                .spacing(10)
                .push(
                    Button::new(&mut self.back_btn, Text::new("Back"))
                        .on_press(SettingsMessage::GoTo(None)),
                )
                .push(display_message("Change password"))
                .into(),
        };
        let body = match self.page {
            None => Self::menu(
                &mut self.goto_tokens_btn,
                &mut self.goto_password_btn,
                &mut self.logout_btn,
            ),
            Some(Page::Tokens) => Self::show_tokens(&self.tokens),
            Some(Page::Password) => Self::change_password(),
        };
        let content: Element<'_, SettingsMessage> = Container::new(
            Column::new()
                .align_items(Align::Center)
                .max_width(600)
                .padding(20)
                .spacing(16)
                .push(title)
                .push(body),
        )
        .align_x(Align::Center)
        .align_y(Align::Center)
        .into();

        content.map(Message::Settings)
    }

    fn menu<'a>(
        goto_tokens: &'a mut button::State,
        goto_password: &'a mut button::State,
        logout: &'a mut button::State,
    ) -> Element<'a, SettingsMessage> {
        Column::new()
            .align_items(Align::Center)
            .max_width(600)
            .padding(20)
            .spacing(16)
            .push(
                Button::new(goto_tokens, Text::new("Refresh Tokens"))
                    .on_press(SettingsMessage::GoTo(Some(Page::Tokens))),
            )
            .push(
                Button::new(goto_password, Text::new("Change password"))
                    .on_press(SettingsMessage::GoTo(Some(Page::Password))),
            )
            .push(Button::new(logout, Text::new("Logout")))
            .into()
    }
    fn show_tokens(tokens: &Option<Vec<String>>) -> Element<'static, SettingsMessage> {
        if let Some(tokens) = tokens {
            tokens
                .into_iter()
                .fold(Column::new().spacing(20), |column, token| {
                    column.push(display_message(&*token))
                })
                .into()
        } else {
            Text::new("Loading...").into()
        }
    }
    fn change_password() -> Element<'static, SettingsMessage> {
        Text::new("unimplemented").into()
    }
}
