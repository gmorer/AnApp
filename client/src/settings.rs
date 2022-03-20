use crate::api::Api;
use crate::{display_message, Message};
use chrono::{TimeZone, Utc};
use iced::{
    button, container, Align, Button, Color, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Space, Text,
};
use proto::client::user::RefreshToken;

struct TokenContainerStyle;

impl container::StyleSheet for TokenContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Color::from_rgb(0.5, 0.5, 0.5).into(),
            text_color: Color::WHITE.into(),
            border_radius: 5.0,
            ..container::Style::default()
        }
    }
}

struct TokenDeleteButton;

impl button::StyleSheet for TokenDeleteButton {
    fn active(&self) -> button::Style {
        button::Style {
            background: Color::from_rgb(1.0, 0.0, 0.0).into(),
            text_color: Color::WHITE.into(),
            border_radius: 3.0,
            ..button::Style::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Password,
    Tokens,
}

#[derive(Debug, Clone)]
pub struct Settings {
    api: Api,
    tokens: Option<Vec<(RefreshToken, button::State)>>,
    page: Option<Page>,
    back_btn: button::State,
    goto_tokens_btn: button::State,
    goto_password_btn: button::State,
    logout_btn: button::State,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    // Init,
    Tokens(Vec<RefreshToken>),
    Error(String),
    GoTo(Option<Page>),
    DeleteToken(String),
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
            SettingsMessage::Tokens(t) => {
                self.tokens = Some(
                    t.into_iter()
                        .map(|a| (a, button::State::default()))
                        .collect(),
                )
            }
            SettingsMessage::GoTo(p) => {
                self.page = p.clone();
                if p.as_ref().map_or(false, |p| p == &Page::Tokens) {
                    let mut api = self.api.clone();
                    return Command::perform(
                        async move { api.get_refresh_tokens().await },
                        |res| match res {
                            Ok(t) => Message::Settings(SettingsMessage::Tokens(t)),
                            Err(e) => Message::Settings(SettingsMessage::Error(format!("{:?}", e))),
                        },
                    );
                }
            }
            SettingsMessage::Error(e) => eprintln!("{}", e),
            SettingsMessage::DeleteToken(t) => {
                let mut api = self.api.clone();
                self.tokens = None;
                return Command::perform(async move { api.delete_refresh_token(t).await }, |res| {
                    match res {
                        Ok(()) => Message::Settings(SettingsMessage::GoTo(Some(Page::Tokens))),
                        Err(e) => Message::Settings(SettingsMessage::Error(format!("{:?}", e))),
                    }
                });
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
            Some(Page::Tokens) => {
                if let Some(tokens) = &mut self.tokens {
                    Self::show_tokens(tokens)
                } else {
                    Text::new("Loading...").into()
                }
            }
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
    fn show_tokens<'a>(
        tokens: &'a mut Vec<(RefreshToken, button::State)>,
    ) -> Element<'a, SettingsMessage> {
        let mut columns = Column::new().spacing(20);
        for (token, button) in tokens.iter_mut() {
            let first_line = Row::new()
                .push(Text::new(token.token.clone()).size(30))
                .push(Space::with_width(Length::Fill))
                .push(Text::new(Utc.timestamp(token.last_use as i64, 0).to_string()).size(22));
            let second_line = Row::new()
                .push(Text::new(token.from.clone()).size(24))
                .push(Space::with_width(Length::Fill))
                .push(Text::new(Utc.timestamp(token.creation_date as i64, 0).to_string()).size(22));
            columns = columns.push(
                Container::new(
                    Row::new()
                        .align_items(Align::Center)
                        .push(
                            Column::new()
                                .width(Length::Fill)
                                .push(first_line)
                                .push(second_line),
                        )
                        .push(Space::with_width(Length::Units(10)))
                        .push(
                            Button::new(button, Text::new("D"))
                                // Button::new(button, Text::new("🗑").size(40))
                                .style(TokenDeleteButton)
                                .width(Length::Shrink)
                                .padding(10)
                                .on_press(SettingsMessage::DeleteToken(token.token.clone())),
                        ),
                )
                .width(Length::Fill)
                .padding(20)
                .style(TokenContainerStyle {}),
            );
        }
        columns.into()
    }
    fn change_password() -> Element<'static, SettingsMessage> {
        Text::new("unimplemented").into()
    }
}
