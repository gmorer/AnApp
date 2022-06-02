use crate::api::Api;
use crate::Message;
use chrono::{TimeZone, Utc};
use iced::pure::{self, button, column, container, row, text, text_input, vertical_space, Element};
use iced::{
    alignment::{Horizontal, Vertical},
    Alignment, Color, Command, Length,
};
use proto::client::user::RefreshToken;

struct TokenContainerStyle;

// TODO style
/*
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
*/

struct TokenDeleteButton;
// TODO put back the style
/*
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
*/

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Password(bool), // bool -> is loading
    Tokens,
}

#[derive(Debug, Clone)]
pub struct Settings {
    api: Api,
    tokens: Option<Vec<RefreshToken>>,
    page: Option<Page>,
    old_password: String,
    new_password: String,
    new2_password: String,
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
            old_password: String::new(),
            new_password: String::new(),
            new2_password: String::new(),
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
            None => text("Settings")
                .width(Length::Fill)
                .size(100)
                .color([0.5, 0.5, 0.5])
                .horizontal_alignment(Horizontal::Center)
                .into(),
            Some(Page::Tokens) => row()
                .spacing(10)
                .push(
                    // Button::new(&mut self.back_btn, Text::new("Back"))
                    button("Back").on_press(SettingsMessage::GoTo(None)),
                )
                .push(text("Refresh tokens"))
                .into(),
            Some(Page::Password(..)) => row()
                .spacing(10)
                .push(button(text("Back")).on_press(SettingsMessage::GoTo(None)))
                .push(text("Change password"))
                .into(),
        };
        let body = match self.page {
            None => Self::menu(),
            Some(Page::Tokens) => {
                if let Some(tokens) = &mut self.tokens {
                    Self::show_tokens(tokens)
                } else {
                    text("Loading...").into()
                }
            }
            Some(Page::Password(false)) => Self::change_password(
                &mut self.old_password,
                &mut self.new_password,
                &mut self.new2_password,
            ),
            Some(Page::Password(true)) => text("Changing the password...").into(),
        };
        let content: Element<'_, SettingsMessage> = container(
            column()
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

    fn menu<'a>() -> Element<'a, SettingsMessage> {
        column()
            .align_items(Alignment::Center)
            .max_width(600)
            .padding(20)
            .spacing(16)
            .push(
                button(text("Refresh Tokens")).on_press(SettingsMessage::GoTo(Some(Page::Tokens))),
            )
            .push(
                button(text("Change password"))
                    .on_press(SettingsMessage::GoTo(Some(Page::Password(false)))),
            )
            .push(button(text("Logout")))
            .into()
    }
    fn show_tokens<'a>(tokens: &'a mut Vec<RefreshToken>) -> Element<'a, SettingsMessage> {
        let mut columns = column().spacing(20);
        for token in tokens.iter_mut() {
            let first_line = row()
                .push(text(token.token.clone()).size(30))
                .push(vertical_space(Length::Fill))
                .push(text(Utc.timestamp(token.last_use as i64, 0).to_string()).size(22));
            let second_line = row()
                .push(text(token.from.clone()).size(24))
                .push(vertical_space(Length::Fill))
                .push(text(Utc.timestamp(token.creation_date as i64, 0).to_string()).size(22));
            columns = columns.push(
                // column(
                row()
                    .align_items(Alignment::Center)
                    .push(
                        column()
                            .width(Length::Fill)
                            .push(first_line)
                            .push(second_line),
                    )
                    .push(vertical_space(Length::Units(10)))
                    .push(
                        button(text("D"))
                            // Button::new(button, Text::new("🗑").size(40))
                            // .style(TokenDeleteButton)
                            .width(Length::Shrink)
                            .padding(10)
                            .on_press(SettingsMessage::DeleteToken(token.token.clone())),
                    )
                    // )
                    .width(Length::Fill)
                    .padding(20), // .style(TokenContainerStyle {}),
            );
        }
        columns.into()
    }
    fn change_password<'a>(
        old_password: &str,
        new_password: &str,
        new2_password: &str,
    ) -> Element<'a, SettingsMessage> {
        let column = column()
            .align_items(Alignment::Center)
            .max_width(600)
            .padding(20)
            .spacing(16)
            .push(
                text_input(
                    "Old password",
                    old_password,
                    SettingsMessage::OldPasswordChange,
                )
                .password()
                .padding(10)
                .size(32),
            )
            .push(
                text_input(
                    "New password",
                    new_password,
                    SettingsMessage::NewPasswordChange,
                )
                .password()
                .padding(10)
                .size(32),
            )
            .push(
                text_input(
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
                .push(button(text("Change password")).on_press(SettingsMessage::ChangePassword))
                .into()
        } else {
            column.push(text("Password missmatch")).into()
        }
    }
}
