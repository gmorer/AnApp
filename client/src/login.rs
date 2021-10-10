use iced::{
    button, text_input, Align, Button, Column, Command, Container, Element, HorizontalAlignment,
    Length, Row, Text, TextInput,
};

use crate::api::{self, Api};
use crate::Message;

#[derive(Debug, Clone)]
pub struct Credentials {}

#[derive(Debug, Clone)]
pub struct Login {
    api: Api,
    username: String,
    username_state: text_input::State,
    password: String,
    password_state: text_input::State,
    invite_code: String,
    invite_code_state: text_input::State,
    show_signup: bool,
    ok_button: button::State,
    swap_button: button::State,
    is_loading: bool,
}

#[derive(Debug, Clone)]
pub enum LoginMessage {
    UsernameChanged(String),
    PasswordChanged(String),
    Error(api::Error),
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
            username_state: text_input::State::default(),
            password: "".to_string(),
            password_state: text_input::State::default(),
            invite_code: "".to_string(),
            invite_code_state: text_input::State::default(),
            show_signup: false,
            ok_button: button::State::default(),
            swap_button: button::State::default(),
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
                        async move { api.signup(username, password, invite_code).await },
                        res,
                    );
                } else {
                    return Command::perform(
                        async move { api.login(username, password).await },
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
        let title = Text::new(if self.show_signup { "Signup" } else { "Login" })
            .width(Length::Fill)
            .size(100)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);
        let mut inputs = Column::new()
            .align_items(Align::Center)
            .max_width(600)
            .padding(20)
            .spacing(16)
            .push(title)
            .push(
                TextInput::new(
                    &mut self.username_state,
                    "Username",
                    &self.username,
                    LoginMessage::UsernameChanged,
                )
                .padding(10)
                .size(32),
            )
            .push(
                TextInput::new(
                    &mut self.password_state,
                    "Password",
                    &self.password,
                    LoginMessage::PasswordChanged,
                )
                .padding(10)
                .size(32)
                .password(),
            );
        if self.show_signup {
            inputs = inputs.push(
                TextInput::new(
                    &mut self.invite_code_state,
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
        let content: Element<'_, LoginMessage> = Container::new(
            inputs.push(
                Row::new()
                    .spacing(10)
                    .push(
                        Button::new(
                            &mut self.swap_button,
                            Text::new(switch_text)
                                .horizontal_alignment(HorizontalAlignment::Center),
                        )
                        .width(Length::Fill)
                        .on_press(if self.is_loading {
                            LoginMessage::None
                        } else {
                            LoginMessage::SwapClicked
                        }),
                    )
                    .push(
                        Button::new(
                            &mut self.ok_button,
                            Text::new(ok_text).horizontal_alignment(HorizontalAlignment::Center),
                        )
                        .width(Length::Fill)
                        .on_press(if self.is_loading {
                            LoginMessage::None
                        } else {
                            LoginMessage::OkClicked
                        }),
                    ),
            ),
        )
        .align_x(Align::Center)
        .align_y(Align::Center)
        .into();

        content.map(Message::Login)
    }
}
