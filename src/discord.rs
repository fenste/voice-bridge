//! Discord handler
use anyhow::bail;
use serenity::async_trait;
use serenity::all::{
    Command,
    CommandInteraction,
    CommandOptionType,
    CommandDataOptionValue,
    CreateCommand,
    CreateCommandOption,
    CreateInteractionResponse,
    CreateInteractionResponseMessage,
    EditInteractionResponse,
    Context,
    EventHandler,
    Interaction,
    Message,
    Ready,
};
use serenity::framework::standard::{ Args, CommandResult, macros::{ command, group } };
use serenity::Result as SerenityResult;
use std::sync::Arc;
use std::io::Read;

// Songbird imports
use songbird::input::{ Input, RawAdapter, Compose };
use songbird::events::EventContext;
use songbird::{ Event, EventHandler as VoiceEventHandler };
use songbird::events::CoreEvent;

use crate::ListenerHolder;
use crate::BufferedPipeline;

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {:#?}", command);
            let result: Result<(), anyhow::Error> = match command.data.name.as_str() {
                "join_voice" => handle_join(&ctx, &command).await,
                _ => Err(anyhow::Error::msg("not implemented :(")),
            };

            if let Err(err) = result {
                println!("Failed to run command: {}", err);
                if
                    let Err(why) = (if command.get_response(&ctx.http).await.is_err() {
                        command.create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new().content(err.to_string())
                            )
                        ).await
                    } else {
                        command
                            .edit_response(
                                &ctx.http,
                                EditInteractionResponse::new().content(err.to_string())
                            ).await
                            .map(|_| ())
                    })
                {
                    println!("Cannot respond to slash command: {}", why);
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let command = CreateCommand::new("join_voice")
            .description("Join voice channel")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "channel to join"
                ).required(true)
            );

        Command::create_global_command(&ctx.http, command).await.expect("Failed creating commands");
    }
}

#[group]
#[commands(deafen, leave, mute, play, ping, undeafen, unmute)]
pub struct General;

#[command]
#[only_in(guilds)]
async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = {
        let guild = msg.guild(&ctx.cache).expect("No guild found!");
        guild.id
    }; // guild is dropped here

    let manager = songbird
        ::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        check_msg(msg.channel_id.say(&ctx.http, "Already deafened").await);
    } else {
        if let Err(e) = handler.deafen(true).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Deafened").await);
    }

    Ok(())
}

fn register_join(command: CreateCommand) -> CreateCommand {
    command
        .name("join_voice")
        .description("Join voice channel")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Channel,
                "channel",
                "channel to join"
            ).required(true)
        )
}

async fn handle_join(ctx: &Context, interaction: &CommandInteraction) -> anyhow::Result<()> {
    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => bail!("Command can't be used outside of servers!"),
    };
    let option = interaction.data.options.get(0).expect("Expected channel option");

    let connect_to = match &option.value {
        CommandDataOptionValue::Channel(channel_id) => *channel_id,
        _ => bail!("Expected channel argument!"),
    };
    // let guild = msg.guild(&ctx.cache).expect("No guild found!");
    // let guild_id = guild.id;

    // let channel_id = guild
    //     .voice_states.get(&msg.author.id)
    //     .and_then(|voice_state| voice_state.channel_id);

    // let connect_to = match channel_id {
    //     Some(channel) => channel,
    //     None => {
    //         check_msg(msg.reply(ctx, "Not in a voice channel").await);

    //         return Ok(None);
    //     }
    // };

    interaction.create_response(
        &ctx.http,
        CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new().ephemeral(true))
    ).await?;

    let manager = songbird
        ::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = manager.join(guild_id, connect_to).await?;

    // if let Ok(_) = conn_result {
    // NOTE: this skips listening for the actual connection result.
    let channel: crate::AudioBufferDiscord;
    let ts_buffer: crate::TsToDiscordPipeline;
    {
        let data_read = ctx.data.read().await;
        let (ts_buf, chan) = data_read
            .get::<ListenerHolder>()
            .expect("Expected CommandCounter in TypeMap.")
            .clone();
        channel = chan;
        ts_buffer = ts_buf;
    }
    let mut handler = handler_lock.lock().await;
    // TODO: Need to implement proper custom audio source for Songbird 0.5.x
    // The TeamSpeak->Discord audio pipeline needs to be redesigned for 0.5.x
    println!(
        "Warning: TeamSpeak to Discord audio forwarding not yet implemented for Songbird 0.5.x"
    );
    // Skip playing the input for now
    let buffered = BufferedPipeline::new(ts_buffer.clone());
    buffered.start_filler(); // Start the background task

    let discord_input = Input::from(RawAdapter::new(buffered, 48000, 2));
    let _track = handler.play_input(discord_input);

    handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), Receiver::new(channel.clone()));

    handler.add_global_event(CoreEvent::VoiceTick.into(), Receiver::new(channel.clone()));

    handler.add_global_event(CoreEvent::RtcpPacket.into(), Receiver::new(channel.clone()));

    handler.add_global_event(CoreEvent::ClientDisconnect.into(), Receiver::new(channel.clone()));

    handler.add_global_event(CoreEvent::RtpPacket.into(), Receiver::new(channel.clone()));

    //     check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())).await);
    // } else {
    //     check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel").await);
    // }
    println!("joined");
    interaction.edit_response(&ctx.http, EditInteractionResponse::new().content("Joined")).await?;
    // interaction.create_followup_message(&ctx.http, |response| {
    //     response.content("Joined")
    // }).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = {
        let guild = msg.guild(&ctx.cache).expect("No guild found!");
        guild.id
    }; // guild is dropped here

    let manager = songbird
        ::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = {
        let guild = msg.guild(&ctx.cache).expect("No guild found!");
        guild.id
    }; // guild is dropped here

    let manager = songbird
        ::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        check_msg(msg.channel_id.say(&ctx.http, "Already muted").await);
    } else {
        if let Err(e) = handler.mute(true).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Now muted").await);
    }

    Ok(())
}

#[command]
async fn ping(context: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&context.http, "Pong!").await);

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(
                msg.channel_id.say(&ctx.http, "Must provide a URL to a video or audio").await
            );
            return Ok(());
        }
    };

    if !url.starts_with("http") {
        check_msg(msg.channel_id.say(&ctx.http, "Must provide a valid URL").await);
        return Ok(());
    }

    let guild_id = {
        let guild = msg.guild(&ctx.cache).expect("No guild found!");
        guild.id
    }; // guild is dropped here

    let manager = songbird
        ::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        // Create a reqwest client (ideally you'd share this across requests in production)
        let client = reqwest::Client::new();

        // Create a lazy YouTube DL source
        let mut src = songbird::input::YoutubeDl::new(client, url.clone());

        // Optionally fetch metadata first
        match src.aux_metadata().await {
            Ok(metadata) => {
                let title = metadata.title.as_deref().unwrap_or("<Unknown>");
                let artist = metadata.artist.as_deref().unwrap_or("<Unknown>");

                check_msg(
                    msg.channel_id.say(
                        &ctx.http,
                        format!("Playing **{}** by **{}**", title, artist)
                    ).await
                );

                // Play the source
                let _handle = handler.play_input(src.into());
            }
            Err(why) => {
                println!("Error fetching metadata: {:?}", why);
                check_msg(msg.channel_id.say(&ctx.http, "Error fetching audio source").await);
            }
        }
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = {
        let guild = msg.guild(&ctx.cache).expect("No guild found!");
        guild.id
    }; // guild is dropped here

    let manager = songbird
        ::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(false).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Undeafened").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to undeafen in").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = {
        let guild = msg.guild(&ctx.cache).expect("No guild found!");
        guild.id
    }; // guild is dropped here

    let manager = songbird
        ::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Unmuted").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to unmute in").await);
    }

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

struct Receiver {
    sink: crate::AudioBufferDiscord,
}

impl Receiver {
    pub fn new(voice_receiver: crate::AudioBufferDiscord) -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        Self {
            sink: voice_receiver,
        }
    }
}

#[async_trait]
impl VoiceEventHandler for Receiver {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        match ctx {
            EventContext::SpeakingStateUpdate(speaking) => {
                // Handle speaking state updates
                println!("Speaking state: ssrc={}, user_id={:?}", speaking.ssrc, speaking.user_id);
            }
            EventContext::RtpPacket(rtp_data) => {
                // Parse the RTP packet manually from bytes
                // RTP header is at least 12 bytes
                let packet_bytes = &rtp_data.packet;

                if packet_bytes.len() < 12 {
                    return None; // Too short to be valid RTP
                }

                // Parse RTP header (simplified)
                // Byte 0: V(2), P(1), X(1), CC(4)
                // Bytes 4-7: SSRC
                // Bytes 2-3: Sequence number
                // Payload starts at byte 12 (or more if there are extensions)

                let ssrc = u32::from_be_bytes([
                    packet_bytes[8],
                    packet_bytes[9],
                    packet_bytes[10],
                    packet_bytes[11],
                ]);

                let sequence = u16::from_be_bytes([packet_bytes[2], packet_bytes[3]]);

                // Check for extension (X bit in byte 0)
                let has_extension = (packet_bytes[0] & 0x10) != 0;
                let mut payload_offset = 12;

                if has_extension && packet_bytes.len() >= 16 {
                    // Extension header is 4 bytes, then extension data
                    let ext_len =
                        (u16::from_be_bytes([packet_bytes[14], packet_bytes[15]]) as usize) * 4;
                    payload_offset = 16 + ext_len;
                }

                if payload_offset < packet_bytes.len() {
                    let opus_data = &packet_bytes[payload_offset..];

                    let dur;
                    {
                        let time = std::time::Instant::now();
                        let mut lock = self.sink.lock().await;
                        dur = time.elapsed();
                        if let Err(e) = lock.handle_packet(ssrc, sequence, opus_data.to_vec()) {
                            eprintln!("Failed to handle Discord voice packet: {}", e);
                        }
                        if dur.as_millis() > 1 {
                            eprintln!("Acquiring lock took {}ms", dur.as_millis());
                        }
                    }
                }
            }
            EventContext::VoiceTick(tick) => {
                // VoiceTick fires every 20ms with decoded PCM audio
                for (&ssrc, voice_data) in &tick.speaking {
                    if let Some(audio) = &voice_data.decoded_voice {
                        // This is decoded PCM audio (Vec<i16>)
                        // You can process it here or pass to your audio handler
                        if audio.len() > 0 {
                            println!("Voice tick for SSRC {}: {} samples", ssrc, audio.len());
                            // TODO: Adapt your audio handler to work with decoded PCM
                            // instead of raw Opus packets
                        }
                    }
                }
            }
            EventContext::RtcpPacket(_rtcp_data) => {
                // Handle RTCP packets if needed
            }
            EventContext::ClientDisconnect(disconnect) => {
                println!("Client disconnected: user {:?}", disconnect.user_id);
            }
            _ => {}
        }
        None
    }
}
