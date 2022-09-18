use crate::Message;
use chrono::{TimeZone, Utc};
use client_lib::Api;
use iced::pure::{button, column, container, row, text, text_input, Element};
use iced::{
    alignment::{Horizontal, Vertical},
    Alignment, Color, Command, Length,
};
use iced_pure::widget::{button as pureButton, container as pureContainer};
use proto::client::user::{InviteToken, RefreshToken};

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    // Init,
    RefreshTokens(Vec<RefreshToken>),
    Invites(Vec<InviteToken>),
    Error(String),
    GoTo(Option<Page>),
    DeleteToken(String),
    OldPasswordChange(String),
    NewPasswordChange(String),
    NewPasswordChangeBis(String),
    ChangePassword,
    CreateInvite,
}

struct TokenRow {}

// TODO: not working
impl pureContainer::StyleSheet for TokenRow {
    fn style(&self) -> pureContainer::Style {
        pureContainer::Style {
            background: Color::from_rgb(0.5, 0.5, 0.5).into(),
            text_color: Color::WHITE.into(),
            border_radius: 5.0,
            ..pureContainer::Style::default()
        }
    }
}

// TODO: not working
impl pureButton::StyleSheet for TokenRow {
    fn active(&self) -> pureButton::Style {
        pureButton::Style {
            background: Color::from_rgb(1.0, 0.0, 0.0).into(),
            text_color: Color::WHITE.into(),
            border_radius: 3.0,
            ..pureButton::Style::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Password(bool), // bool -> is loading
    RefreshTokens,
    Invites,
}

#[derive(Debug, Clone)]
pub struct Settings {
    api: Api,
    refresh_tokens: Option<Vec<RefreshToken>>,
    invites: Option<Vec<InviteToken>>,
    page: Option<Page>,
    old_password: String,
    new_password: String,
    new2_password: String,
}

impl Settings {
    pub fn new(api: Api) -> Self {
        Self {
            api,
            refresh_tokens: None,
            invites: None,
            page: None,
            old_password: String::new(),
            new_password: String::new(),
            new2_password: String::new(),
        }
    }

    pub fn update(&mut self, msg: SettingsMessage) -> Command<Message> {
        println!("received a msg: {:?}", msg);
        match msg {
            SettingsMessage::RefreshTokens(t) => self.refresh_tokens = Some(t),
            SettingsMessage::GoTo(p) => {
                self.page = p;
                match self.page {
                    Some(Page::RefreshTokens) => {
                        let mut api = self.api.clone();
                        return Command::perform(
                            async move { api.user.get_refresh_tokens().await },
                            |res| match res {
                                Ok(t) => Message::Settings(SettingsMessage::RefreshTokens(t)),
                                Err(e) => {
                                    Message::Settings(SettingsMessage::Error(format!("{:?}", e)))
                                }
                            },
                        );
                    }
                    Some(Page::Password(_)) => {
                        self.old_password.clear();
                        self.new_password.clear();
                        self.new2_password.clear();
                    }
                    Some(Page::Invites) => {
                        let mut api = self.api.clone();
                        return Command::perform(
                            async move { api.user.get_invites().await },
                            |res| match res {
                                Ok(t) => Message::Settings(SettingsMessage::Invites(t)),
                                Err(e) => {
                                    Message::Settings(SettingsMessage::Error(format!("{:?}", e)))
                                }
                            },
                        );
                    }
                    None => {}
                }
            }
            SettingsMessage::Error(e) => eprintln!("{}", e),
            SettingsMessage::DeleteToken(t) => {
                let mut api = self.api.clone();
                self.refresh_tokens = None;
                return Command::perform(
                    async move { api.user.delete_refresh_token(t).await },
                    |res| match res {
                        Ok(()) => {
                            Message::Settings(SettingsMessage::GoTo(Some(Page::RefreshTokens)))
                        }
                        Err(e) => Message::Settings(SettingsMessage::Error(format!("{:?}", e))),
                    },
                );
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
                    async move { api.user.change_password(old_password, new_password).await },
                    |res| match res {
                        Ok(()) => {
                            Message::Settings(SettingsMessage::GoTo(Some(Page::Password(false))))
                        }
                        Err(e) => Message::Settings(SettingsMessage::Error(format!("{:?}", e))),
                    },
                );
            }
            SettingsMessage::Invites(invites) => self.invites = Some(invites),
            SettingsMessage::CreateInvite => {
                self.invites = None;
                let mut api = self.api.clone();
                return Command::perform(async move { api.user.create_invite().await }, |res| {
                    match res {
                        Ok(_token) => Message::Settings(SettingsMessage::GoTo(Some(Page::Invites))),
                        Err(e) => Message::Settings(SettingsMessage::Error(format!("{:?}", e))),
                    }
                });
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
            Some(Page::RefreshTokens) => row()
                .spacing(10)
                .push(button("Back").on_press(SettingsMessage::GoTo(None)))
                .push(text("Refresh tokens"))
                .into(),
            Some(Page::Password(..)) => row()
                .spacing(10)
                .push(button(text("Back")).on_press(SettingsMessage::GoTo(None)))
                .push(text("Change password"))
                .into(),
            Some(Page::Invites) => row()
                .spacing(10)
                .push(button(text("Back")).on_press(SettingsMessage::GoTo(None)))
                .push(text("Invites"))
                .into(),
        };
        let body = match self.page {
            None => Self::menu(),
            Some(Page::RefreshTokens) => {
                if let Some(refresh_tokens) = &self.refresh_tokens {
                    Self::show_refresh_tokens(refresh_tokens)
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
            Some(Page::Invites) => {
                if let Some(invites) = &self.invites {
                    Self::show_invites(invites)
                } else {
                    text("Loading...").into()
                }
            }
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
            .push(button(text("Invites")).on_press(SettingsMessage::GoTo(Some(Page::Invites))))
            .push(
                button(text("Refresh Tokens"))
                    .on_press(SettingsMessage::GoTo(Some(Page::RefreshTokens))),
            )
            .push(
                button(text("Change password"))
                    .on_press(SettingsMessage::GoTo(Some(Page::Password(false)))),
            )
            .push(button(text("Logout")))
            .into()
    }

    fn show_refresh_tokens<'a>(tokens: &'a Vec<RefreshToken>) -> Element<'a, SettingsMessage> {
        let mut columns = column().spacing(20);
        for token in tokens.iter() {
            let first_line = row()
                .push(text(&token.token).size(30))
                .push(text(Utc.timestamp(token.last_use as i64, 0).to_string()).size(22));
            let second_line = row()
                .push(text(&token.from).size(24))
                .push(text(Utc.timestamp(token.creation_date as i64, 0).to_string()).size(22));
            columns = columns.push(
                container(
                    row()
                        .align_items(Alignment::Center)
                        .push(
                            column()
                                .width(Length::Fill)
                                .push(first_line)
                                .push(second_line),
                        )
                        .push(
                            button(text("D"))
                                .width(Length::Shrink)
                                .padding(10)
                                .style(TokenRow {})
                                .on_press(SettingsMessage::DeleteToken(token.token.clone())),
                        ),
                )
                .width(Length::Fill)
                .padding(20)
                .style(TokenRow {}),
            );
        }
        columns.into()
    }

    fn show_invites<'a>(invites: &'a Vec<InviteToken>) -> Element<'a, SettingsMessage> {
        let mut columns = column()
            .spacing(20)
            .push(button(text("Create invite")).on_press(SettingsMessage::CreateInvite));
        for invite in invites.iter() {
            columns = columns.push(
                container(row().push(text(&invite.token)).push(text(if invite.used {
                    "used"
                } else {
                    "not used"
                })))
                .width(Length::Fill)
                .padding(20)
                .style(TokenRow {}),
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
