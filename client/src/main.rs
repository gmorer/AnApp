use iced::scrollable::{self, Scrollable};
use iced::{
    alignment::Horizontal, pure, Application, Command, Container, Element, Length,
    Settings as IcedSettings, Subscription, Text,
};

use iced_native;

#[cfg(target_arch = "wasm32")]
use grpc_web_client::Client as Channel;

mod login;
use login::{Login, LoginMessage};

mod settings;
use settings::{Settings, SettingsMessage};

mod api;
use api::Api;

struct Pages {
    login: Login,
    login_state: pure::State,
    settings: Settings,
    settings_state: pure::State,
}

impl Pages {
    pub fn new(api: Api) -> Self {
        Self {
            login: Login::new(api.clone()),
            login_state: pure::State::new(),
            settings: Settings::new(api),
            settings_state: pure::State::new(),
        }
    }
}

enum IsConnected {
    Yes((Api, Pages)),
    No(u8),
}

struct App {
    is_connected: IsConnected,
    scroll: scrollable::State,
    should_exit: bool,
}

impl App {
    fn new() -> Self {
        App {
            is_connected: IsConnected::No(0),
            scroll: scrollable::State::default(),
            should_exit: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    GotApi(Result<Api, String>),
    NativeEvent(iced_native::Event),
    WaitedToConnect(u8),
    None,

    // Children
    Login(LoginMessage),
    Settings(SettingsMessage),
}

pub fn display_message<'a, T, M>(msg: T) -> Element<'a, M>
where
    T: Into<String>,
{
    //Container::new(
    Text::new(msg)
        .horizontal_alignment(Horizontal::Center)
        .size(50)
        .into()
    //)
    //.width(Length::Fill)
    //.height(Length::Fill)
    //.center_y()
    //.into()
}

fn connect_server() -> Command<Message> {
    Command::perform(api::Api::connect(), Message::GotApi)
}

fn wait_to_connect(x: u8) -> Command<Message> {
    Command::perform(
        tokio::time::sleep(std::time::Duration::from_secs(1)),
        move |_| Message::WaitedToConnect(x),
    )
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (App::new(), connect_server())
    }

    fn title(&self) -> String {
        "sasMy apap".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::None => Command::none(),
            Message::GotApi(Ok(api)) => {
                let pages = Pages::new(api.clone());
                (*self).is_connected = IsConnected::Yes((api, pages));
                Command::none()
            }
            Message::GotApi(Err(err)) => {
                eprintln!("connection error: {}", err);
                wait_to_connect(5)
            }
            Message::WaitedToConnect(x) => {
                self.is_connected = IsConnected::No(x);
                if x == 0 {
                    connect_server()
                } else {
                    wait_to_connect(x - 1)
                }
            }
            Message::NativeEvent(ev) => {
                if let iced_native::Event::Window(iced_native::window::Event::CloseRequested) = ev {
                    self.should_exit = true;
                }
                Command::none()
            }
            // TODO: 8 line per page doesnt scale
            Message::Login(msg) => {
                if let IsConnected::Yes((_, pages)) = &mut self.is_connected {
                    pages.login.update(msg)
                } else {
                    eprintln!("Login message without login pages");
                    Command::none()
                }
            }
            Message::Settings(msg) => {
                if let IsConnected::Yes((_, pages)) = &mut self.is_connected {
                    pages.settings.update(msg)
                } else {
                    eprintln!("Setting message settins login pages");
                    Command::none()
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::NativeEvent)
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn view(&mut self) -> Element<Message> {
        let content = match &mut self.is_connected {
            IsConnected::No(x) => {
                if *x == 0 {
                    display_message("Loading ...")
                } else {
                    display_message(format!(
                        "Error while connecting to the server, retrying in {}s ...",
                        x
                    ))
                }
            }
            IsConnected::Yes((api, pages)) => {
                if !api.as_creds() {
                    pure::Pure::new(&mut pages.login_state, pages.login.display()).into()
                } else {
                    pure::Pure::new(&mut pages.settings_state, pages.settings.display()).into()
                }
            }
        };
        Scrollable::new(&mut self.scroll)
            .padding(40)
            .push(Container::new(content).width(Length::Fill).center_x())
            .into()
    }
}

// GRPC example
// async fn say_hello(
//     mut this: UsersClient<Channel>,
//     request: impl tonic::IntoRequest<UsersReq>,
// ) -> Result<tonic::Response<UsersRes>, tonic::Status> {
//     this.say_hello(request).await
// }

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = App::run(IcedSettings {
        exit_on_close_request: false,
        ..IcedSettings::default()
    }) {
        eprintln!("Error from iced: {}", e);
    }
    // let channel = Channel::from_static("http://[::1]:5051").connect().await?;
    // let mut client = HelloClient::connect("http://[::1]:5051").await?;

    // let request = tonic::Request::new(UsersReq { message: "yoloswag".to_string() });
    // let res = client.say_hello(request).await;
    // println!("Hello, world!: {:?}", res);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = App::run(IcedSettings::default()) {
        eprintln!("Error from iced: {}", e);
    }
    // let channel = Channel::from_static("http://[::1]:5051").connect().await?;
    // let mut client = HelloClient::connect("http://[::1]:5051").await?;

    // let request = tonic::Request::new(HelloReq { message: "yoloswag".to_string() });
    // let res = client.say_hello(request).await;
    // println!("Hello, world!: {:?}", res);
    Ok(())
}
