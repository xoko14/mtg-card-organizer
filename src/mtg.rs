use crate::models::{Card, CardInDeck};
#[derive(Clone, Debug)]
pub struct CardErrorInsight {
    pub card_name: String,
    pub error: String,
}

impl CardErrorInsight {
    pub fn new(card_name: &str, error: &str) -> Self {
        Self {
            card_name: card_name.to_owned(),
            error: error.to_owned(),
        }
    }
}

pub async fn process_decklist(decklist: String) -> (Vec<CardInDeck>, Vec<CardErrorInsight>) {
    let card_list = decklist.lines().filter(|&l| !l.trim().is_empty()).map(|l| {
        let mut split = l.split_whitespace();
        let quantity: i32 = split.next().unwrap_or("0").parse().unwrap_or(0);
        let name = split.collect::<Vec<_>>().join(" ");
        (quantity, name)
    });

    let mut cards_in_deck = Vec::<CardInDeck>::new();
    let mut errors = Vec::<CardErrorInsight>::new();

    for card in card_list {
        if card.0 == 0 {
            errors.push(CardErrorInsight::new(&card.1, "Invalid quantity"));
            continue;
        }

        match scryfall::Card::named_fuzzy(&card.1).await {
            Ok(c) => cards_in_deck.push(CardInDeck {
                quantity: card.0,
                current_quantity: 0,
                card: Card {
                    name: c.name,
                    img: c
                        .image_uris
                        .map(|imgs| {
                            imgs.small
                                .or(imgs.png)
                                .map(|url| url.to_string())
                                .unwrap_or(String::default())
                        })
                        .unwrap_or(String::default()),
                },
            }),
            Err(e) => errors.push(CardErrorInsight::new(&card.1, &e.to_string())),
        };
    }

    (cards_in_deck, errors)
}
