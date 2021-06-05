use proto::client::hello::{ hello_client::HelloClient, HelloReq, HelloRes };
use iced::{ Application, Command, Clipboard, Element, Container, Text, Length, HorizontalAlignment, Settings, Column, Button, button };
use tonic::{ Request };

#[cfg(not(target_arch = "wasm32"))]
use tonic::transport::Channel;

#[cfg(target_arch = "wasm32")]
use grpc_web_client::Client as Channel;

const ADDR: &str = "http://127.0.0.1:5051";

enum State {
	Connecting,
	Connected,
	Loading,
	Loaded
}

struct App {
	state: State,
	btn: button::State,
	client: Option<HelloClient<Channel>>,
	msg: String
}

impl App {
	fn new() -> Self {
		App {
			state: State::Connecting,
			btn: button::State::new(),
			client: None,
			msg: "Is loading...".to_string()
		}
	}
}

#[derive(Debug, Clone)]
enum Message {
	IsConnected(Result<HelloClient<Channel>, String>),
	ReceveidData(Result<String, String>),
	ButtonClicked
}

fn display_message(msg: &str) -> Element<Message> {
	Container::new(
        Text::new(msg)
            .horizontal_alignment(HorizontalAlignment::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .into()
}

fn display_button<'a>(btn_state: &'a mut button::State, btn_text: &str, text: &str) -> Element<'a, Message> {
	let button = Button::new(btn_state, Text::new(btn_text))
		.on_press(Message::ButtonClicked)
		.padding(10);
		// .style(style::Button::Icon);

	let title = Text::new(text)
		.width(Length::Fill)
		.size(100)
		.color([0.5, 0.5, 0.5])
		.horizontal_alignment(HorizontalAlignment::Center);

	Column::new()
		.max_width(800)
		.spacing(20)
		.push(title)
		.push(button)
		.into()
}

#[cfg(not(target_arch = "wasm32"))]
fn connect_grpc() -> Command<Message> {
	async fn get_client() -> Result<HelloClient<Channel>, tonic::transport::Error> {
		Ok(HelloClient::new(Channel::from_static(ADDR).connect().await?))
	}
	Command::perform(get_client(), |c| Message::IsConnected(c.map_err(|e| e.to_string())))
}

#[cfg(target_arch = "wasm32")]
fn connect_grpc() -> Command<Message> {
	async fn to_fut<T>(a: T) -> T {
		a
	}
	Command::perform(to_fut(HelloClient::new(Channel::new(ADDR.to_string()))), |client| Message::IsConnected(Ok(client)))
}


impl Application for App {
	type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

	fn new(_flags: ()) -> (Self, Command<Message>) {
		(App::new(), connect_grpc())
	}

	fn title(&self) -> String {
		"My app".to_string()
	}

	fn update(&mut self, message: Message, _clip: &mut Clipboard) -> Command<Message> {
		match message {
			Message::IsConnected(c) => {
				match c {
					Ok(c) => {
						(*self).state = State::Connected;
						(*self).msg = "Connected succesfully :)".to_string();
						(*self).client = Some(c);
					}
					Err(e) => {
						(*self).state = State::Loaded;
						(*self).msg = format!("Connection error: {}", e);
					}
				}
				Command::none()
			},
			Message::ReceveidData(msg) => {
				(*self).state = State::Loaded;
				match msg {
					Ok(msg) => (*self).msg = msg,
					Err(msg) => (*self).msg = msg,
				}
				Command::none()
			},
			Message::ButtonClicked => {
				(*self).state = State::Loading;
				(*self).msg = "Loading...".to_string();
				let req = Request::new(HelloReq{message: "yo poto".to_string()});
				if let Some(client) = &self.client {
					Command::perform(
						say_hello(client.clone(), req),
						|a| match a {
							Ok(res) => {
								Message::ReceveidData(Ok(format!("{:?}", res.into_inner().message)))
							},
							Err(status) => {
								Message::ReceveidData(Err(status.to_string()))
							}
						}
					)
				} else {
					connect_grpc()
				}
			}
		}
	}

	fn view(&mut self) -> Element<Message> {
		match self.state {
			State::Connecting => display_message("Is connecting....."),
			State::Connected => display_button(&mut self.btn, "Say hello!", self.msg.as_ref()),
			State::Loading => display_message(&self.msg),
			State::Loaded  => display_button(&mut self.btn, if self.client.is_some() { "Resend a msg" } else { "try to reconnect" }, self.msg.as_ref()),

		}
	}
}

async fn say_hello(mut this: HelloClient<Channel>, request: impl tonic::IntoRequest<HelloReq>) ->  Result<tonic::Response<HelloRes>, tonic::Status> {
	this.say_hello(request).await
}


#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main()  -> Result<(), Box<dyn std::error::Error>>{

	if let Err(e) = App::run(Settings::default()) {
		eprintln!("Error from iced: {}", e);
	}
	// let channel = Channel::from_static("http://[::1]:5051").connect().await?;
    // let mut client = HelloClient::connect("http://[::1]:5051").await?;

	// let request = tonic::Request::new(HelloReq { message: "yoloswag".to_string() });
	// let res = client.say_hello(request).await;	
	// println!("Hello, world!: {:?}", res);
	Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main()  -> Result<(), Box<dyn std::error::Error>>{

	if let Err(e) = App::run(Settings::default()) {
		eprintln!("Error from iced: {}", e);
	}
	// let channel = Channel::from_static("http://[::1]:5051").connect().await?;
    // let mut client = HelloClient::connect("http://[::1]:5051").await?;

	// let request = tonic::Request::new(HelloReq { message: "yoloswag".to_string() });
	// let res = client.say_hello(request).await;	
	// println!("Hello, world!: {:?}", res);
	Ok(())
}
