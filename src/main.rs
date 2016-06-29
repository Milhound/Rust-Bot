extern crate discord;
extern crate hyper;
extern crate serde_json;
extern crate rand;
extern crate xi_unicode;

use discord::{Connection, Discord, State};
use discord::model::{ChannelId, Event, Message, MessageId, User};
use std::env;
use std::error::Error;
use rand::Rng;

const HELP_TEXT: &'static str = r#"
This bot is currently under Development by Milhound
```
Commands:
/cat -> Random Cat Picture
/boom -> Random Explosion (of 3)
/ping -> Pong!
/toast -> Tasty Toast
/play (url) -> Plays youtube in voice channel
/insult (@mention) -> Insults the mention(s)

In Development:
Inspirational Quotes
Slap (@mention)
Temperature Conversion
International Times
```
If you have anything you'd like to see in the future DM Milhound.
"#;

const INFO_TEXT: &'static str = "\
Rust bot was programmed in Rust Lang, using Discord-rs: https://github.com/SpaceManiac/discord-rs.\
";

const TOAST_TEXT: &'static str = r#"
```
Toast!

      ______
 ____((     )_
|'->==))   (= \
|  \ ||_____|_ \
|[> \___________\
| | |            |                                    |
 \  |            |             .--.                   |
  \ |            |)---.   .---'    `-.         .----(]|
   \|____________|     `-'            `.     .'       |
                                         `---'        |
```
"#;

struct Request {
    user: User,
    command: Command,
    channel_id: ChannelId,
    message_id: MessageId,
}

enum Command {
    Cat,
    Boom,
    Ping,
    Info,
    Help,
    Toast,
    Play(String),
    End,
    Insult(Vec<User>),
    Wipe(Option<u64>),
    User(Vec<User>),
    Quit,
}

enum CommandParseError {
    NotACommand,
    UnknownCommand,
    InvalidArgument,
}

impl Request {
    fn from_message(message: &Message) -> Result<Self, CommandParseError> {
        Command::parse(message).map(|command| {
            Request {
                command: command,
                user: message.author.clone(),
                channel_id: message.channel_id,
                message_id: message.id,
            }
        })
    }

    /// Checks whether the user is authorized to request this command.
    fn is_authorized(&self) -> bool {
        match &self.command {
            &Command::Wipe(_) |
            &Command::User(_) |
            &Command::Quit => {
                self.user.id.0 == 167693414156992512
            }
            _ => true
        }
    }

    /// Runs this command.
    fn execute(&self, connection: &mut Connection, state: &State, discord: &Discord) -> bool {
        match &self.command {
            &Command::Cat => {
                if let Ok(s) = get_cat() {
                    println!("{}", s);
                    let _ = discord.send_message(&self.channel_id, &s, "", false);
                }
            }
            &Command::Boom => {
                let images = vec!["src/boom.png", "src/boom1.png", "src/boom2.png"];
                let file = std::fs::File::open(rand::thread_rng().choose(&images).expect("image src incorrect")).expect("Missing image");
                let _ = discord.send_file(&self.channel_id, "Badda BOOM!!!", file, "boom1.png");
            }
            &Command::Ping => {
                let pong = format!("<@{:?}>, Pong", &self.user.id.0);
                let _ = discord.send_message(&self.channel_id, &pong , "", false);
            }
            &Command::Info => {
                let _ = discord.send_message(&self.channel_id, INFO_TEXT, "", false);
            }
            &Command::Help => {
                let _ = discord.send_message(&self.channel_id, HELP_TEXT, "", false);
            }
            &Command::Toast => {
                let _ = discord.send_message(&self.channel_id, TOAST_TEXT, "", false);
            }
            &Command::Play(ref song_url) => {
                let voice_channel = state.find_voice_user(self.user.id);
                let output = if let Some((server_id, channel_id)) = voice_channel {
                   match discord::voice::open_ytdl_stream(&song_url[..]) {
                       Ok(stream) => {
                           let voice = connection.voice(server_id);
                           voice.set_deaf(true);
                           voice.connect(channel_id);
                           voice.play(stream);
                           String::new()
                        },
                        Err(error) => format!("Error: {}", error),
                        }
                } else {
                    "You must be in a voice channel to play music.".to_owned()
                };
                if output.len() > 0 {
                    warn(discord.send_message(&self.channel_id, &output, "", false));
                }
            }
            &Command::End => {
                let voice_channel = state.find_voice_user(self.user.id);
                if let Some((server_id, _)) = voice_channel {
                    connection.drop_voice(server_id);
                }
            }
            &Command::Insult(ref mentions) => {
                for mention in mentions {
                    if let Ok(insult) = get_insult() {
                        let _ = discord.send_message(&self.channel_id, &format!("<@{:?}>, {}", mention.id.0, insult) , "", false);
                    }
                }
            }
            &Command::Wipe(Some(num)) => {
                let test = discord.get_messages(&self.channel_id, None, None, Some(num + 1));
                if let Ok(messages) = test {
                    for  wipe_msg in &messages {
                        let _ = discord.delete_message(&wipe_msg.channel_id, &wipe_msg.id);
                    }
                }
            }
            &Command::Wipe(None) => {
                let _ = discord.delete_message(&self.channel_id, &self.message_id);
            }
            &Command::User(ref mentions) => {
                for mention in mentions {
                    println!("{:?}", mention);
                }
            }
            &Command::Quit => {
                return false;
            }
        }
        return true;
    }
}

impl Command {
    /// Parses a command.
    fn parse(message: &Message) -> Result<Self, CommandParseError> {
        if !message.content.starts_with("/") {
            // This message is not a command.
            return Err(CommandParseError::NotACommand);
        }

        let mut split = message.content.split(" ");
        let command = split.next().unwrap_or("");
        let argument = split.next().unwrap_or("");

        match &command[1..] {
            "cat" => Ok(Command::Cat),
            "boom" => Ok(Command::Boom),
            "ping" => Ok(Command::Ping),
            "info" => Ok(Command::Info),
            "help" => Ok(Command::Help),
            "toast" => Ok(Command::Toast),
            "play" => Ok(Command::Play(argument.to_string())),
            "end" => Ok(Command::End),
            "insult" => Ok(Command::Insult(message.mentions.clone())),
            "wipe" => {
                if argument.is_empty() {
                    Ok(Command::Wipe(None))
                } else {
                    match argument.parse() {
                        Ok(num) => Ok(Command::Wipe(Some(num))),
                        Err(_) => Err(CommandParseError::InvalidArgument),
                    }
                }
            }
            "user" => Ok(Command::User(message.mentions.clone())),
            "quit" => Ok(Command::Quit),
            _ => Err(CommandParseError::UnknownCommand),
        }
    }
}

fn warn<T, E: ::std::fmt::Debug>(result: Result<T, E>) {
    match result {
        Ok(_) => {},
            Err(err) => println!("[Warning] {:?}", err)
    }
}

fn get_cat() -> Result<String, Box<Error>> {
    use std::io::Read;
    use serde_json::Value;

    let url: &str = "http://random.cat/meow";
    let client = hyper::Client::new();
    let mut response = try!(client.get(url).send());
    let mut buff = String::new();
    try!(response.read_to_string(&mut buff));
    let decode: Value = try!(serde_json::from_str(&buff));
    let data = decode.as_object().expect("Invalid JSON");
    let file = data.get("file").expect("File not found");
    Ok(file.as_string().unwrap().to_string())
}

fn get_insult() -> Result<String, Box<Error>> {
    use std::io::Read;
    use serde_json::Value;

    let url: &str = "http://quandyfactory.com/insult/json";
    let client = hyper::Client::new();
    let mut response = try!(client.get(url).send());
    let mut buff = String::new();
    try!(response.read_to_string(&mut buff));
    let decode: Value = try!(serde_json::from_str(&buff));
    let data = decode.as_object().expect("Invalid JSON");
    let item = data.get("insult").expect("Unable to locate insult key.");
    Ok(item.as_string().unwrap().to_string())

}

fn main() {
    let discord = Discord::from_bot_token(&env::var("RUST_BOT_TOKEN").expect("DISCORD TOKEN")).expect("Discord Token Error");
	let (mut connection, ready) = discord.connect().expect("Connection Failed.");
	println!("Rust Bot Ready...");

    /* USED TO SET THE AVATAR OF THE BOT TO "src/img.txt" BASE64 ENCODED
    let mut file = vec![];
    std::fs::File::open("src/img.txt").expect("src/img.txt Missing").read_to_end(&mut file).unwrap();
    let file = String::from_utf8(file).unwrap();

    discord.edit_profile(|x| {x.avatar(Some(&file))}).expect("Failed to update avatar");
    */

    let mut state = State::new(ready);

    loop {
        let event = match connection.recv_event() {
            Ok(event) => event,
            Err(err) => {
                println!("[Warning] Receive error: {:?}", err);
				if let discord::Error::WebSocket(..) = err {
					// Handle the websocket connection being dropped
					let (new_connection, ready) = discord.connect().expect("connect failed");
					connection = new_connection;
					state = State::new(ready);
					println!("[Ready] Reconnected successfully.");
				}
				if let discord::Error::Closed(..) = err {
					break
				}
				continue
            },
        };

        state.update(&event);

        match event {
			Event::MessageCreate(message) => {
				println!("{} says: {}", message.author.name, message.content);

                match Request::from_message(&message) {
                    Ok(request) => {
                        if request.is_authorized() {
                            // Execute the command.
                            let should_continue = request.execute(&mut connection, &state, &discord);
                            if !should_continue {
                                break;
                            }
                        } else {
                            // Unauthorized request.
                            let _ = discord.send_message(&message.channel_id, "You are not authorized to use this command.", "", false);
                        }
                    }
                    Err(error) => {
                        match error {
                            CommandParseError::InvalidArgument => {
                                let _ = discord.send_message(&message.channel_id, "Invalid argument", "", false);
                            }
                            _ => {}
                        }
                    }
                }
			}
            Event::VoiceStateUpdate(server_id, _) => {
				// If someone moves/hangs up, and we are in a voice channel,
				if let Some(cur_channel) = connection.voice(server_id).current_channel() {
					// and our current voice channel is empty, disconnect from voice
					if let Some(srv) = state.servers().iter().find(|srv| srv.id == server_id) {
						if srv.voice_states.iter().filter(|vs| vs.channel_id == Some(cur_channel)).count() <= 1 {
							connection.voice(server_id).disconnect();
						}
					}
				}
			}
			_ => {}
		}
	}
	// Log out from the API
	discord.logout().expect("logout failed");
}
