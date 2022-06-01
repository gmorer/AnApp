use crate::api::Api;
use crate::{display_message, Message};
use chrono::{TimeZone, Utc};
use iced::{
    alignment::{Horizontal, Vertical},
    button, container, text_input, Alignment, Button, Color, Column, Command, Container, Element,
    Length, Row, Space, Text, TextInput,
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
    Password(bool), // bool -> is loading
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
    old_password_state: text_input::State,
    old_password: String,
    new_password_state: text_input::State,
    new_password: String,
    new2_password_state: text_input::State,
    new2_password: String,
    change_password_btn: button::State,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    // Init,
    Tokens(Vec<RefreshToken>),
    Error(String),
    GoTo(Option<Page>),
    DeleteToken(String),
    OldPasswordChange(String),
    NewPasswordChange(String),
    NewPasswordChangeBis(String),
    ChangePassword,
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
            old_password_state: text_input::State::new(),
            old_password: String::new(),
            new_password_state: text_input::State::new(),
            new_password: String::new(),
            new2_password_state: text_input::State::new(),
            new2_password: String::new(),
            change_password_btn: button::State::new(),
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
                } else if p == Some(Page::Password(true)) || p == Some(Page::Password(false)) {
                    self.old_password.clear();
                    self.new_password.clear();
                    self.new2_password.clear();
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
            SettingsMessage::OldPasswordChange(old_pwd) => self.old_password = old_pwd,
            SettingsMessage::NewPasswordChange(new_pwd) => self.new_password = new_pwd,
            SettingsMessage::NewPasswordChangeBis(new_pwd_bis) => self.new2_password = new_pwd_bis,
            SettingsMessage::ChangePassword => {
                let mut api = self.api.clone();
                self.page = Some(Page::Password(true));
                let old_password = self.old_password.clone();
                let new_password = self.new_password.clone();
                return Command::perform(
                    async move { api.change_password(old_password, new_password).await },
                    |res| match res {
                        Ok(()) => {
                            Message::Settings(SettingsMessage::GoTo(Some(Page::Password(false))))
                        }
                        Err(e) => Message::Settings(SettingsMessage::Error(format!("{:?}", e))),
                    },
                );
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
                .horizontal_alignment(Horizontal::Center)
                .into(),
            Some(Page::Tokens) => Row::new()
                .spacing(10)
                .push(
                    Button::new(&mut self.back_btn, Text::new("Back"))
                        .on_press(SettingsMessage::GoTo(None)),
                )
                .push(display_message("Refresh tokens"))
                .into(),
            Some(Page::Password(..)) => Row::new()
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
            Some(Page::Password(false)) => Self::change_password(
                &mut self.old_password_state,
                &mut self.new_password_state,
                &mut self.new2_password_state,
                &mut self.old_password,
                &mut self.new_password,
                &mut self.new2_password,
                &mut self.change_password_btn,
            ),
            Some(Page::Password(true)) => Text::new("Changing the password...").into(),
        };
        let content: Element<'_, SettingsMessage> = Container::new(
            Column::new()
                .align_items(Alignment::Center)
                .max_width(600)
                .padding(20)
                .spacing(16)
                .push(title)
                .push(body),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(Message::Settings)
    }

    fn menu<'a>(
        goto_tokens: &'a mut button::State,
        goto_password: &'a mut button::State,
        logout: &'a mut button::State,
    ) -> Element<'a, SettingsMessage> {
        Column::new()
            .align_items(Alignment::Center)
            .max_width(600)
            .padding(20)
            .spacing(16)
            .push(
                Button::new(goto_tokens, Text::new("Refresh Tokens"))
                    .on_press(SettingsMessage::GoTo(Some(Page::Tokens))),
            )
            .push(
                Button::new(goto_password, Text::new("Change password"))
                    .on_press(SettingsMessage::GoTo(Some(Page::Password(false)))),
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
                        .align_items(Alignment::Center)
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
    fn change_password<'a>(
        old_password_state: &'a mut text_input::State,
        new_password_state: &'a mut text_input::State,
        new2_password_state: &'a mut text_input::State,
        old_password: &str,
        new_password: &str,
        new2_password: &str,
        change_password_btn: &'a mut button::State,
    ) -> Element<'a, SettingsMessage> {
        let column = Column::new()
            .align_items(Alignment::Center)
            .max_width(600)
            .padding(20)
            .spacing(16)
            .push(
                TextInput::new(
                    old_password_state,
                    "Old password",
                    old_password,
                    SettingsMessage::OldPasswordChange,
                )
                .password()
                .padding(10)
                .size(32),
            )
            .push(
                TextInput::new(
                    new_password_state,
                    "New password",
                    new_password,
                    SettingsMessage::NewPasswordChange,
                )
                .password()
                .padding(10)
                .size(32),
            )
            .push(
                TextInput::new(
                    new2_password_state,
                    "New password",
                    new2_password,
                    SettingsMessage::NewPasswordChangeBis,
                )
                .password()
                .padding(10)
                .size(32),
            );
        if new_password == new2_password {
            column
                .push(
                    Button::new(change_password_btn, Text::new("Change password"))
                        .on_press(SettingsMessage::ChangePassword),
                )
                .into()
        } else {
            column.push(Text::new("Password missmatch")).into()
        }
    }
}
