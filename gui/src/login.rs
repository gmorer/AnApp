use iced::pure::{self, button, column, container, row, text, text_input, Element};
use iced::{
    alignment::{Horizontal, Vertical},
    Alignment, Command, Length,
};

use crate::Message;
use client_lib::{Api, Error as ApiError};

#[derive(Debug, Clone)]
pub struct Credentials {}

#[derive(Debug, Clone)]
pub struct Login {
    api: Api,
    username: String,
    password: String,
    invite_code: String,
    show_signup: bool,
    is_loading: bool,
}

#[derive(Debug, Clone)]
pub enum LoginMessage {
    UsernameChanged(String),
    PasswordChanged(String),
    Error(ApiError),
    InviteCodeChanged(String),
    Loading(bool),
    OkClicked,
    SwapClicked,
    None,
}

impl Login {
    pub fn new(api: Api) -> Login {
        Self {
            api,
            username: "".to_string(),
            password: "".to_string(),
            invite_code: "".to_string(),
            show_signup: false,
            is_loading: false,
        }
    }

    pub fn update(&mut self, message: LoginMessage) -> Command<Message> {
        match message {
            LoginMessage::None => {}
            LoginMessage::Loading(is_loading) => self.is_loading = is_loading,
            LoginMessage::UsernameChanged(username) => self.username = username,
            LoginMessage::PasswordChanged(password) => self.password = password,
            LoginMessage::InviteCodeChanged(invite_code) => self.invite_code = invite_code,
            LoginMessage::Error(e) => {
                self.is_loading = false;
                self.password.clear();
                eprintln!("{:?}", e)
            }
            LoginMessage::OkClicked => {
                self.is_loading = true;
                let res = |res| {
                    if let Err(e) = res {
                        Message::Login(LoginMessage::Error(e))
                    } else {
                        Message::Login(LoginMessage::Loading(false))
                    }
                };
                let mut api = self.api.clone();
                let username = self.username.to_string();
                let password = self.password.to_string();
                let invite_code = self.invite_code.to_string();
                if self.show_signup {
                    return Command::perform(
                        async move { Api::signup(&username, &password, &invite_code).await },
                        res,
                    );
                } else {
                    return Command::perform(
                        async move { Api::login(&username, &password).await },
                        res,
                    );
                }
            }
            LoginMessage::SwapClicked => {
                self.show_signup = !self.show_signup;
                self.password.clear();
            }
        }
        Command::none()
    }

    pub fn display(&mut self) -> Element<Message> {
        let title = text(if self.show_signup { "Signup" } else { "Login" })
            .width(Length::Fill)
            .size(100)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(Horizontal::Center);
        let mut inputs = column()
            .align_items(Alignment::Center)
            .max_width(600)
            .padding(20)
            .spacing(16)
            .push(title)
            .push(
                text_input("Username", &self.username, LoginMessage::UsernameChanged)
                    .padding(10)
                    .size(32),
            )
            .push(
                text_input("Password", &self.password, LoginMessage::PasswordChanged)
                    .padding(10)
                    .size(32)
                    .password(),
            );
        if self.show_signup {
            inputs = inputs.push(
                text_input(
                    "Invite code",
                    &self.invite_code,
                    LoginMessage::InviteCodeChanged,
                )
                .padding(10)
                .size(32),
            );
        }
        let switch_text = if self.is_loading {
            "..."
        } else if self.show_signup {
            "Switch to Login"
        } else {
            "Switch to Signup"
        };
        let ok_text = if self.is_loading {
            "..."
        } else if self.show_signup {
            "Signup"
        } else {
            "Login"
        };
        let content: Element<'_, LoginMessage> = container(
            inputs.push(
                row()
                    .spacing(10)
                    .push(
                        button(text(switch_text).horizontal_alignment(Horizontal::Center))
                            .width(Length::Fill)
                            .on_press(if self.is_loading {
                                LoginMessage::None
                            } else {
                                LoginMessage::SwapClicked
                            }),
                    )
                    .push(
                        button(text(ok_text).horizontal_alignment(Horizontal::Center))
                            .width(Length::Fill)
                            .on_press(if self.is_loading {
                                LoginMessage::None
                            } else {
                                LoginMessage::OkClicked
                            }),
                    ),
            ),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(Message::Login)
    }
}
