use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Card {
    pub name: String,
    pub img: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deck {
    pub name: String,
    pub cards: Vec<CardInDeck>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CardInDeck {
    pub quantity: i32,
    pub current_quantity: i32,
    pub card: Card
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexedCard{
    pub name: String,
    pub img: String,
    pub deck_id: Uuid
}