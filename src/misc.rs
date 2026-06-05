use std::sync::LazyLock;

use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity::Activity, activity::Timestamps};

use thiserror::Error;




#[derive(Error, Debug)]
pub enum DiscordError {
    #[error("There was an error connecting to discord's IPC. {0}")]
    ClientConnection(#[from] discord_rich_presence::error::Error),

    #[error("Couldn't set the user's activity on discord")]
    ActivitySetting(discord_rich_presence::error::Error)
}

pub enum Action {
    Installing,
    Uninstalling,
    Enabling,
    Disabling,
    ListingMods,
    ChangingSettings,
    Playing
}
impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        match self {
            Action::Installing => "Altering Timelines...",
            Action::Uninstalling => "Unaltering Timelines",
            Action::Enabling => "Bringing back the chaos",
            Action::Disabling => "Taking a step back from the chaos",
            Action::ListingMods => "Taking a look at the mess they've made",
            Action::ChangingSettings => "Not-so-carefully playing with the cosmic dials",
            Action::Playing => "Enjoying their custom-made fan-fiction"
        }
    }
}

static START_TIME: LazyLock<i64> = LazyLock::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
});

pub fn update_discord_rich_presence(drp: &String, last_action: Action) -> Result<(), DiscordError> {
    let mut client = DiscordIpcClient::new("1512579862676639864");
    client.connect().map_err(|er| DiscordError::ClientConnection(er))?;

    let payload = Activity::new()
        .details(drp)
        .state(last_action.as_ref())
        .timestamps(Timestamps::new().start(*START_TIME));

    client.set_activity(payload).map_err(|er| DiscordError::ActivitySetting(er))?;
    
    Ok(())
}