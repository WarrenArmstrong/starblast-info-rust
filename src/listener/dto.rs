use std::fmt::Binary;

use serde::de::Error as DeError;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::Error;

////////////////////////////////////////////////////////////////////////////////
// simstatus
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct SystemDTO {
    pub name: String,
    pub id: i32,
    pub mode: String,
    pub players: i32,
    pub unlisted: bool,
    pub open: bool,
    pub survival: bool,
    pub time: i32,
    pub criminal_activity: i32,
    pub mod_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsageDTO {
    pub cpu: i32,
    pub memory: i32,
    pub ctime: Option<i32>,
    pub elapsed: Option<f64>,
    pub timestamp: Option<i64>,
    pub pid: Option<i32>,
    pub ppid: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ServerDTO {
    pub location: String,
    pub address: String,
    pub current_players: i32,
    pub systems: Vec<SystemDTO>,
    pub usage: UsageDTO,
}

#[derive(Debug)]
pub struct SimStatusDTO {
    pub servers: Vec<ServerDTO>,
}

impl<'de> Deserialize<'de> for SimStatusDTO {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?
            .as_array()
            .ok_or(D::Error::custom("Expected array for SimStatus"))?
            .to_owned();

        let servers: Vec<ServerDTO> = v
            .iter()
            .map(ServerDTO::deserialize)
            .filter_map(Result::ok)
            .collect();

        Ok(SimStatusDTO { servers: servers })
    }
}

////////////////////////////////////////////////////////////////////////////////
// ws text player name dto
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct WSPlayerNameCustomDTO {
    pub badge: String,
    pub finish: String,
    pub laser: Option<String>,
    pub hue: u16,
}

#[derive(Debug, Deserialize)]
pub struct WSPlayerNameDataDTO {
    pub id: u8,
    pub hue: u16,
    pub friendly: u8,
    pub player_name: String,
    pub custom: Option<WSPlayerNameCustomDTO>,
}

#[derive(Debug, Deserialize)]
pub struct WSPlayerNameDTO {
    pub name: String,
    pub data: WSPlayerNameDataDTO,
}

////////////////////////////////////////////////////////////////////////////////
// ws text welcome dto
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct WSWelcomeModuleDTO {
    pub id: u8,
    #[serde(rename = "type")]
    pub module_type: String,
    pub x: f64,
    pub y: f64,
    pub dir: u8,
}

#[derive(Debug, Deserialize)]
pub struct WSWelcomeStationDTO {
    pub modules: Vec<WSWelcomeModuleDTO>,
    pub phase: f64,
}

#[derive(Debug, Deserialize)]
pub struct WSWelcomeTeamDTO {
    pub faction: String,
    pub base_name: String,
    pub hue: u16,
    pub station: WSWelcomeStationDTO,
}

#[derive(Debug, Deserialize)]
pub struct WSWelcomeModeDTO {
    pub max_players: i32,
    pub crystal_value: f64,
    pub crystal_drop: i32,
    pub map_size: i32,
    pub map_density: Option<i32>,
    pub lives: i32,
    pub max_level: i32,
    pub friendly_colors: i32,
    pub close_time: i32,
    pub close_number: i32,
    pub map_name: Option<String>,
    pub unlisted: bool,
    pub survival_time: i32,
    pub survival_level: i32,
    pub starting_ship: i32,
    pub starting_ship_maxed: bool,
    pub asteroids_strength: i32,
    pub friction_ratio: f64,
    pub speed_mod: f64,
    pub rcs_toggle: bool,
    pub weapon_drop: i32,
    pub mines_self_destroy: bool,
    pub mines_destroy_delay: i32,
    pub healing_enabled: bool,
    pub healing_ratio: f64,
    pub shield_regen_factor: f64,
    pub power_regen_factor: f64,
    pub auto_refill: bool,
    pub projectile_speed: f64,
    pub strafe: f64,
    pub release_crystal: bool,
    pub large_grid: bool,
    pub bouncing_lasers: f64,
    pub max_tier_lives: i32,
    pub auto_assign_teams: bool,
    pub station_size: i32,
    pub crystal_capacity: Vec<i32>,
    pub deposit_shield: Vec<i32>,
    pub spawning_shield: Vec<i32>,
    pub structure_shield: Vec<i32>,
    pub deposit_regen: Vec<i32>,
    pub spawning_regen: Vec<i32>,
    pub structure_regen: Vec<i32>,
    pub repair_threshold: f64,
    pub all_ships_can_dock: bool,
    pub all_ships_can_respawn: bool,
    pub id: String,
    pub restore_ship: Option<bool>,
    pub teams: Vec<WSWelcomeTeamDTO>,
}

#[derive(Debug, Deserialize)]
pub struct WSWelcomeDataDTO {
    pub version: i32,
    pub seed: i32,
    pub servertime: i32,
    pub name: String,
    pub systemid: i32,
    pub size: i32,
    pub mode: WSWelcomeModeDTO,
    pub region: String,
}

#[derive(Debug, Deserialize)]
pub struct WSWelcomeDTO {
    pub name: String,
    pub data: WSWelcomeDataDTO,
}

////////////////////////////////////////////////////////////////////////////////
// binary get trait
////////////////////////////////////////////////////////////////////////////////

pub trait BinaryGet {
    fn get_byte(&self, index: usize) -> Result<u8, Error>;
    fn get_range_from(&self, range: std::ops::RangeFrom<usize>) -> Result<&[u8], Error>;
    fn get_range(&self, range: std::ops::Range<usize>) -> Result<&[u8], Error>;
}

impl<T: AsRef<[u8]> + ?Sized> BinaryGet for &T {
    fn get_byte(&self, index: usize) -> Result<u8, Error> {
        let slice = self.as_ref();
        slice
            .get(index)
            .ok_or(Error::BinaryIndexOutOfBounds {
                index,
                binary: slice.to_vec(),
            })
            .map(|&byte| byte)
    }

    fn get_range_from(&self, range: std::ops::RangeFrom<usize>) -> Result<&[u8], Error> {
        let slice = self.as_ref();
        slice
            .get(range.clone())
            .ok_or(Error::BinaryRangeFromOutOfBounds {
                range,
                binary: slice.to_vec(),
            })
    }

    fn get_range(&self, range: std::ops::Range<usize>) -> Result<&[u8], Error> {
        let slice = self.as_ref();
        slice
            .get(range.clone())
            .ok_or(Error::BinaryRangeOutOfBounds {
                range,
                binary: slice.to_vec(),
            })
    }
}

////////////////////////////////////////////////////////////////////////////////
// ws binary 200 dto
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Binary200PlayerDTO {
    pub id: u8,
    pub score: u32,
    pub ship_id: u32,
    pub is_alive: bool,
    pub x: u8,
    pub y: u8,
}

impl Binary200PlayerDTO {
    fn parse(binary: &[u8; 8]) -> Result<Self, Error> {
        let id = binary.get_byte(0)?;
        let score = 0xFFFFFF & u32::from_le_bytes(binary.get_range(4..8)?.try_into()?);

        let ship_index = 1 + (u32::from_le_bytes(binary.get_range(4..8)?.try_into()?) >> 24);
        let ship_level = 1 + ((binary.get_byte(3)? >> 5) & 7);
        let ship_id = (100 * u32::from(ship_level)) + ship_index;

        let is_alive = (1 & binary.get_byte(3)?) == 1;
        let x = binary.get_byte(1)?;
        let y = binary.get_byte(2)?;

        Ok(Self {
            id,
            score,
            ship_id,
            is_alive,
            x,
            y,
        })
    }
}

#[derive(Debug)]
pub struct Binary200DTO {
    pub msg_type: u8,
    pub player_count: u8,
    pub players: Vec<Binary200PlayerDTO>,
}

impl Binary200DTO {
    fn parse(binary: &[u8]) -> Result<Self, Error> {
        let msg_type = binary.get_byte(0)?;

        let player_count = binary.get_byte(1)?;

        let players_bytes = binary.get_range_from(2..)?;

        let mut players = Vec::new();

        for chunk in players_bytes.chunks_exact(8).take(player_count as usize) {
            let player = Binary200PlayerDTO::parse(chunk.try_into()?)?;
            players.push(player);
        }

        Ok(Self {
            msg_type,
            player_count,
            players,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// ws binary 205 dto
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Binary205ModuleDTO {
    pub id: u8,
    pub status: u8,
}

#[derive(Debug)]
pub struct Binary205TeamDTO {
    pub id: u8,
    pub is_open: bool,
    pub base_level: u8,
    pub crystals: u32,
    pub module_count: u8,
    pub modules: Vec<Binary205ModuleDTO>,
}

impl Binary205TeamDTO {
    fn parse(binary: &[u8; 19], idx: u8) -> Result<Self, Error> {
        let is_open = binary.get_byte(0)? == 1;
        let base_level = binary.get_byte(1)?;
        let crystals = u32::from_le_bytes(binary.get_range(2..6)?.try_into()?);
        let module_count = binary.get_byte(6)?;

        let modules_bytes = binary.get_range_from(7..)?;

        let mut modules = Vec::new();

        for (idx, status) in modules_bytes.iter().enumerate() {
            let module = Binary205ModuleDTO {
                id: idx.try_into()?,
                status: status.to_owned(),
            };
            modules.push(module);
        }

        Ok(Self {
            id: idx,
            is_open,
            base_level,
            crystals,
            module_count,
            modules,
        })
    }
}

#[derive(Debug)]
pub struct Binary205DTO {
    pub msg_type: u8,
    pub teams: Vec<Binary205TeamDTO>,
}

impl Binary205DTO {
    fn parse(binary: &[u8]) -> Result<Self, Error> {
        let msg_type = binary.get_byte(0)?;
        let content_bytes = binary.get_range_from(1..)?;

        let mut teams = Vec::new();

        for (idx, chunk) in content_bytes.chunks_exact(19).take(3).enumerate() {
            let team = Binary205TeamDTO::parse(chunk.try_into()?, idx.try_into()?)?;
            teams.push(team);
        }

        Ok(Self { msg_type, teams })
    }
}

////////////////////////////////////////////////////////////////////////////////
// ws binary 206 dto
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Binary206AlienDTO {
    pub id: u16,
    pub x: u8,
    pub y: u8,
}

impl Binary206AlienDTO {
    fn parse(binary: &[u8; 5]) -> Result<Self, Error> {
        let id = u16::from_le_bytes(binary.get_range(0..2)?.try_into()?);
        let x = binary.get_byte(3)?;
        let y = binary.get_byte(4)?;

        Ok(Self { id, x, y })
    }
}

#[derive(Debug)]
pub struct Binary206DTO {
    pub msg_type: u8,
    pub wave: u8,
    pub wave_start_time: u32,
    pub alien_count: u16,
    pub aliens: Vec<Binary206AlienDTO>,
}

impl Binary206DTO {
    fn parse(binary: &[u8]) -> Result<Self, Error> {
        let msg_type = binary.get_byte(0)?;
        let wave = binary.get_byte(1)?;
        let wave_start_time = u32::from_le_bytes(binary.get_range(2..6)?.try_into()?);
        let alien_count = u16::from_le_bytes(binary.get_range(8..10)?.try_into()?);

        let mut aliens = Vec::new();

        for chunk in binary
            .get_range_from(10..)?
            .chunks_exact(5)
            .take(alien_count.into())
        {
            let alien = Binary206AlienDTO::parse(chunk.try_into()?)?;
            aliens.push(alien);
        }

        Ok(Self {
            msg_type,
            wave,
            wave_start_time,
            alien_count,
            aliens,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// ws dto
////////////////////////////////////////////////////////////////////////////////

use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Message};

pub enum WSDTO {
    WSWelcomeDTO(WSWelcomeDTO),
    WSPlayerNameDTO(WSPlayerNameDTO),
    CannotJoin,

    Binary200DTO(Binary200DTO),
    Binary205DTO(Binary205DTO),
    Binary206DTO(Binary206DTO),

    Close(Option<CloseFrame<'static>>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

impl WSDTO {
    pub fn parse(message: Message) -> Result<Self, Error> {
        match message {
            Message::Binary(binary) => match binary.get(0) {
                Some(200) => Ok(WSDTO::Binary200DTO(Binary200DTO::parse(binary.as_slice())?)),
                Some(205) => Ok(WSDTO::Binary205DTO(Binary205DTO::parse(binary.as_slice())?)),
                Some(206) => Ok(WSDTO::Binary206DTO(Binary206DTO::parse(binary.as_slice())?)),
                _ => todo!(),
            },
            Message::Text(message) => {
                let v: serde_json::Value = serde_json::from_str(&message)?;

                match v.get("name").and_then(Value::as_str) {
                    Some("player_name") => {
                        Ok(WSDTO::WSPlayerNameDTO(WSPlayerNameDTO::deserialize(v)?))
                    }
                    Some("welcome") => Ok(WSDTO::WSWelcomeDTO(WSWelcomeDTO::deserialize(v)?)),
                    Some("cannot_join") => Ok(WSDTO::CannotJoin),
                    Some(name) => {
                        let err_msg = format!(
                            "unknown variant: {}, expected one of [\"player_name\", \"welcome\", \"cannot_join\"]",
                            name
                        );
                        Err(serde_json::Error::custom(err_msg).into())
                    }
                    None => Err(serde_json::Error::custom(
                        "name field not present in websocket text message",
                    )
                    .into()),
                }
            }
            Message::Ping(ping) => Ok(WSDTO::Ping(ping)),
            Message::Pong(pong) => Ok(WSDTO::Pong(pong)),
            Message::Close(close_frame_option) => Ok(WSDTO::Close(close_frame_option)),
        }
    }
}
