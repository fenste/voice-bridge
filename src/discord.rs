use serenity::async_trait;
use serenity::all::{Context as SerenityContext, Ready};

// Poise imports
use poise::serenity_prelude as serenity;

// Songbird imports
use songbird::input::{Input, RawAdapter, Compose};
use songbird::events::EventContext;
use songbird::{Event, EventHandler as VoiceEventHandler};
use songbird::events::CoreEvent;

use crate::ListenerHolder;
use crate::BufferedPipeline;

// Poise context type
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Application data (shared state)
pub struct Data {}

pub struct Handler;

#[async_trait]
impl serenity::EventHandler for Handler {
    async fn ready(&self, _ctx: SerenityContext, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

/// Join a voice channel
#[poise::command(slash_command, guild_only)]
pub async fn join(
    ctx: Context<'_>,
    #[description = "Voice channel to join"] channel: serenity::Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;
    
    let connect_to = match channel {
        serenity::Channel::Guild(ch) => ch.id,
        _ => {
            ctx.say("Must specify a voice channel").await?;
            return Ok(());
        }
    };

    ctx.defer_ephemeral().await?;

    let manager = songbird::get(ctx.serenity_context()).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = manager.join(guild_id, connect_to).await?;

    // Get audio handlers
    let channel: crate::AudioBufferDiscord;
    let ts_buffer: crate::TsToDiscordPipeline;
    {
        let data_read = ctx.serenity_context().data.read().await;
        let (ts_buf, chan) = data_read
            .get::<ListenerHolder>()
            .expect("Expected audio handlers in TypeMap.")
            .clone();
        channel = chan;
        ts_buffer = ts_buf;
    }

    let mut handler = handler_lock.lock().await;
    
    let buffered = BufferedPipeline::new(ts_buffer.clone());
    buffered.start_filler();

    let discord_input = Input::from(RawAdapter::new(buffered, 48000, 2));
    let _track = handler.play_input(discord_input);

    handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), Receiver::new(channel.clone()));
    handler.add_global_event(CoreEvent::VoiceTick.into(), Receiver::new(channel.clone()));
    handler.add_global_event(CoreEvent::RtcpPacket.into(), Receiver::new(channel.clone()));
    handler.add_global_event(CoreEvent::ClientDisconnect.into(), Receiver::new(channel.clone()));
    handler.add_global_event(CoreEvent::RtpPacket.into(), Receiver::new(channel.clone()));

    ctx.say("Joined voice channel!").await?;
    Ok(())
}

/// Leave the voice channel
#[poise::command(slash_command, guild_only)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let manager = songbird::get(ctx.serenity_context()).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id).await?;
        ctx.say("Left voice channel").await?;
    } else {
        ctx.say("Not in a voice channel").await?;
    }

    Ok(())
}

/// Deafen the bot
#[poise::command(slash_command, guild_only)]
pub async fn deafen(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let manager = songbird::get(ctx.serenity_context()).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = manager.get(guild_id).ok_or("Not in a voice channel")?;
    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        ctx.say("Already deafened").await?;
    } else {
        handler.deafen(true).await?;
        ctx.say("Deafened").await?;
    }

    Ok(())
}

/// Undeafen the bot
#[poise::command(slash_command, guild_only)]
pub async fn undeafen(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let manager = songbird::get(ctx.serenity_context()).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = manager.get(guild_id).ok_or("Not in a voice channel")?;
    let mut handler = handler_lock.lock().await;

    handler.deafen(false).await?;
    ctx.say("Undeafened").await?;

    Ok(())
}

/// Mute the bot
#[poise::command(slash_command, guild_only)]
pub async fn mute(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let manager = songbird::get(ctx.serenity_context()).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = manager.get(guild_id).ok_or("Not in a voice channel")?;
    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        ctx.say("Already muted").await?;
    } else {
        handler.mute(true).await?;
        ctx.say("Now muted").await?;
    }

    Ok(())
}

/// Unmute the bot
#[poise::command(slash_command, guild_only)]
pub async fn unmute(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let manager = songbird::get(ctx.serenity_context()).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = manager.get(guild_id).ok_or("Not in a voice channel")?;
    let mut handler = handler_lock.lock().await;

    handler.mute(false).await?;
    ctx.say("Unmuted").await?;

    Ok(())
}

/// Ping the bot
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// Play audio from a URL
#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URL to play"] url: String,
) -> Result<(), Error> {
    if !url.starts_with("http") {
        ctx.say("Must provide a valid URL").await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let manager = songbird::get(ctx.serenity_context()).await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        ctx.defer().await?;

        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
        let mut src = songbird::input::YoutubeDl::new(client, url.clone());

        // Try to fetch metadata
        match src.aux_metadata().await {
            Ok(metadata) => {
                let title = metadata.title.as_deref().unwrap_or("<Unknown>");
                let artist = metadata.artist.as_deref().unwrap_or("<Unknown>");
                ctx.say(format!("Playing **{}** by **{}**", title, artist)).await?;
            }
            Err(why) => {
                println!("Error fetching metadata: {:?}", why);
                ctx.say(format!("Playing: {}", url)).await?;
            }
        }
        
        let _handle = handler.play_input(src.into());
    } else {
        ctx.say("Not in a voice channel to play in").await?;
    }

    Ok(())
}

struct Receiver {
    sink: crate::AudioBufferDiscord,
}

impl Receiver {
    pub fn new(voice_receiver: crate::AudioBufferDiscord) -> Self {
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
                println!("Speaking state: ssrc={}, user_id={:?}", speaking.ssrc, speaking.user_id);
            }
            EventContext::RtpPacket(rtp_data) => {
                let packet_bytes = &rtp_data.packet;

                if packet_bytes.len() < 12 {
                    return None;
                }

                let ssrc = u32::from_be_bytes([
                    packet_bytes[8],
                    packet_bytes[9],
                    packet_bytes[10],
                    packet_bytes[11],
                ]);

                let sequence = u16::from_be_bytes([packet_bytes[2], packet_bytes[3]]);

                let has_extension = (packet_bytes[0] & 0x10) != 0;
                let mut payload_offset = 12;

                if has_extension && packet_bytes.len() >= 16 {
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
                for (&ssrc, voice_data) in &tick.speaking {
                    if let Some(audio) = &voice_data.decoded_voice {
                        if audio.len() > 0 {
                            println!("Voice tick for SSRC {}: {} samples", ssrc, audio.len());
                        }
                    }
                }
            }
            EventContext::RtcpPacket(_rtcp_data) => {}
            EventContext::ClientDisconnect(disconnect) => {
                println!("Client disconnected: user {:?}", disconnect.user_id);
            }
            _ => {}
        }
        None
    }
}