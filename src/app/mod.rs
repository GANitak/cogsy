pub mod impl_app;
pub mod database;
pub mod request;
pub mod message;
pub mod response;
pub mod update;
pub mod fromcli;

use std::collections::BTreeMap;

use chrono::{
    DateTime,
    Utc,
};
use message::Message;
use crate::collection::Collection;

#[derive(Debug, Clone)]
pub struct App {
    pub user_id: String,
    pub token: String,
    pub message: Message,
    pub collection: Collection,
    pub modified: bool,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub username: String,
    pub real_name: String,
    pub registered: DateTime<Utc>,
    pub listings: u32,
    pub collection: u32,
    pub wantlist: u32,
    pub rated: u32,
    pub average_rating: f64,
}

pub struct ListenLogEntry {
    pub id: i64,
    pub title: String,
    pub time: DateTime<Utc>,
}

pub struct ListenLog { //wrapper around a BTreeMap
    pub contents: BTreeMap<DateTime<Utc>, String>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Release {
    pub id: i64,
    pub search_string: String,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub labels: Vec<String>,
    pub formats: Vec<String>,
    pub date_added: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Folders {
    pub contents: BTreeMap<String, Vec<Release>>,
}