extern crate discord;
extern crate reqwest;
use serde::Deserialize;
extern crate serde_json;
extern crate rand;
extern crate xi_unicode;

use discord::{Discord, State};
use discord::model::{Event};
use std::env;
use std::error::Error;
use std::iter;
use std::fmt::Write;

use xi_unicode::LineBreakIterator;

const HELP_TEXT: &'static str = r#"
This bot is currently under Development by Milhound
```
Commands:
ping -> Pong!
toast -> Tasty Toast
help -> This help text
cowsay (Text) -> Share your message with a cow
cat -> Random Cat Picture
insult -> A Shaksperean Insult

In Development:
boom -> Random Explosion (of 3)
quote -> Inspirational Quotes
tof (temp) -> / toc (temp) -> Temperature Conversion
time (zone code) -> International Times

Possibly in the future:
play (url) -> Plays youtube in voice channel
```
If you have anything you'd like to see in the future DM Milhound.
"#;

const INFO_TEXT: &'static str = "\
Rusty was programmed in Rust, using Discord-rs. Check him out: https://github.com/Milhound/Rust-Bot/\
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

const COW_TEXT: &'static str = r"
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||
```
";

const COWSAY_LINE_LENGTH: usize = 40;

async fn get_cat() -> Result<String, Box<dyn Error>> {
    #[derive(Deserialize, Debug)]
    struct Cat {
        url: String,
    }
    let url: &str = "https://api.thecatapi.com/v1/images/search";
    let client = reqwest::Client::new();
    let response = client.get(url).send().await.expect("Failed to send Cat request.");

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<Vec<Cat>>().await {
                Ok(data) => {
                    let url = &data.first().unwrap().url;
                    Ok(url.to_string())
                },
                Err(_) => {
                    Err("Cat API returned a response that didn't include a url.".into())
                },
            }
        },
        _ => {
            Ok("https://cdn2.thecatapi.com/images/ad5.jpg".to_string())
        }
    }
}

async fn get_insult() -> Result<String, Box<dyn Error>> {
    #[derive(Deserialize, Debug)]
    struct Insult {
        insult: String,
    }
    let url: &str = "http://quandyfactory.com/insult/json";
        let client = reqwest::Client::new();
    let response = client.get(url).send().await.expect("Failed to send insult request.");

    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<Insult>().await {
                Ok(data) => {
                    Ok(data.insult)
                },
                Err(_) => Err("Unable to parse insult response json.".into()),
            }
        },
        _ => Err(format!("Error: quandryfactory.com returned a http error {:?}", response.status()).into())
    }
}

/// Cow says
fn cowsay(say: &str) -> String {
    let mut lines = vec![];
    let break_offsets = LineBreakIterator::new(say).map(|(offset, _hard)| offset);
    let mut line_start = 0;
    let mut line_end = 0;
    for offset in break_offsets {
        if say[line_start..offset].chars().count() <= COWSAY_LINE_LENGTH {
            // Line within limit
            line_end = offset;
        } else {
            // Limit exceeded
            lines.push(say[line_start .. line_end].to_string());
            line_start = line_end;
            line_end = offset;
        }
    }
    lines.push(say[line_start..].to_string());
    let border_len = lines.iter().map(|line| line.chars().count()).max().unwrap();
    // pad to constant length
    for line in &mut lines {
        let pad: String = iter::repeat(" ").take(border_len - line.chars().count()).collect();
        line.push_str(&pad[..]);
    }
    let mut text = "```".to_string();
    let top: String = iter::repeat("_").take(border_len).collect();
    let bottom: String = iter::repeat("‾").take(border_len).collect();
    let _ = writeln!(&mut text, "  {}", top);
    for (i, line) in lines.iter().enumerate() {
        let s = if lines.len() == 1 {
            format!(r"< {} >", line)
        } else if i == 0 {
            format!(r"/ {} \\", line)
        } else if i == lines.len() - 1 {
            format!(r"\\ {} /", line)
        } else {
            format!(r"| {} |", line)
        };
        text.push_str(&s[..]);
        text.push_str("\n");
    }
    let _ = write!(&mut text, "  {}", bottom);
    text.push_str(COW_TEXT);
    text
}
#[tokio::main]
async fn main() {
    let discord = Discord::from_bot_token(&env::var("RUST_BOT_TOKEN").expect("DISCORD TOKEN")).expect("Discord Token Error");
	let (mut connection, ready) = discord.connect().expect("Connection Failed.");
	println!("Rust Bot Ready...");
    let state = State::new(ready);

    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                if message.author.id == state.user().id {
                    continue;
                }
                let channel = discord.get_channel(message.channel_id).unwrap();
                match channel{
                    discord::model::Channel::Public(public_channel) => {
                        let channel_name = public_channel.name;
                        println!("{} in {} says: {}", message.author.name,channel_name, message.content);
                    },
                    discord::model::Channel::Private(_) => {
                        println!("{} wispered Rusty: {}", message.author.name, message.content);
                    },
                    _ => ()
                }
                // reply to a command if there was one
                let mut split = message.content.split(' ');
                let first_word = split.next().unwrap_or("");
                let command = split.next().unwrap_or("");
                let argument: String = split.map(|s| s.to_string()+" ").collect();
                if first_word.eq_ignore_ascii_case("Rusty") {
                    match command {
                        "ping" => {
                            let _ = discord.send_message(message.channel_id, "Pong!","", false);
                        },
                        "cowsay" => {
                            let moo = cowsay(&argument.trim());
                            let _ = discord.send_message(message.channel_id, &moo, "", false);
                        },
                        "help" => {
                            let _ = discord.send_message(message.channel_id, HELP_TEXT, "", false);
                        },
                        "info" => {
                            let _ = discord.send_message(message.channel_id, INFO_TEXT, "", false);
                        },
                        "toast" => {
                            let _ = discord.send_message(message.channel_id, TOAST_TEXT, "", false);
                        },
                        "cat" => {
                            let _ = discord.send_message(message.channel_id, &get_cat().await.expect("No cats today"), "", false);
                        },
                        "insult" => {
                            let _ = discord.send_message(message.channel_id, &get_insult().await.expect("No insults today"), "", false);
                        }
                        &_ => {},
                    }
                }
            },
            Ok(_) => {},
            Err(discord::Error::Closed(code, body)) => {
                println!("Gateway closed on us with code {:?}: {}", code, body);
                break;
            },
            Err(err) => println!("Receive error: {:?}", err),
        };

	}
}
