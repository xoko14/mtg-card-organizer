use std::{collections::HashMap, fs};

use bytes::Bytes;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use iced::{
    executor,
    widget::{self, column, image::Handle, row, text_editor},
    Application, Command, Length, Theme,
};
use native_dialog::FileDialog;
use uuid::Uuid;

use crate::{
    models::{CardInDeck, Deck, IndexedCard},
    mtg::{self, CardErrorInsight},
};

const DEFAULT_IMAGE: &[u8] = include_bytes!("../assets/copy_token.png");

pub struct App {
    decks: HashMap<Uuid, Deck>,
    section: Section,
    deck_input_content: text_editor::Content,
    deck_output: String,
    deck_in_progress: Option<Vec<CardInDeck>>,
    deck_name: String,
    search_text: String,
    card_index: Vec<IndexedCard>,
    search_result: Vec<IndexedCard>,
    image_cache: HashMap<String, Bytes>,
    default_image: Bytes,
}

#[derive(Debug, Clone)]
pub enum Section {
    Decks,
    AddDeck,
    ViewDeck(Uuid),
    BuildDecks,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    ChangeSection(Section),
    EditDeckInput(text_editor::Action),
    AnalyzeDeck,
    AnalyzeDeckFinish(Vec<CardInDeck>, Vec<CardErrorInsight>),
    UpdateDeckName(String),
    CreateDeck,
    ViewDeck(Uuid),
    DeleteDeck(Uuid),
    Search(String),
    UpdateImageCache(String, Option<Bytes>),
    AddCard(Uuid, String),
    RemoveCard(Uuid, String),
    Import,
    Export,
}

type AppElement<'a> = iced::Element<'a, AppMessage, Theme, iced::Renderer>;

impl Application for App {
    type Executor = executor::Default;
    type Message = AppMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                ..Default::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "mtg card organizer".to_owned()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::ChangeSection(section) => {
                self.section = section;
            }
            AppMessage::EditDeckInput(action) => self.deck_input_content.perform(action),
            AppMessage::AnalyzeDeck => {
                self.deck_output = "Analyzing...".to_owned();
                let input = self.deck_input_content.text();
                return iced::Command::perform(
                    async move { mtg::process_decklist(input).await },
                    |res| AppMessage::AnalyzeDeckFinish(res.0, res.1),
                );
            }
            AppMessage::AnalyzeDeckFinish(cards_in_deck, errors) => {
                self.deck_output = String::default();
                let total_unique = cards_in_deck.len();
                let total_count = cards_in_deck
                    .iter()
                    .fold(0, |acc, card| acc + card.quantity);

                self.deck_output.push_str(&format!(
                    "Found {} unique cards ({} total cards).\n",
                    total_unique, total_count
                ));
                if errors.len() > 0 {
                    self.deck_output.push_str("Errors:\n");
                }

                for error in errors {
                    self.deck_output
                        .push_str(&format!("{}: {}\n", error.card_name, error.error))
                }

                self.deck_in_progress = Some(cards_in_deck)
            }
            AppMessage::UpdateDeckName(name) => self.deck_name = name,
            AppMessage::CreateDeck => {
                let deck_id = Uuid::new_v4();
                self.decks.insert(
                    deck_id,
                    Deck {
                        name: self.deck_name.clone(),
                        cards: self.deck_in_progress.clone().expect("oops"),
                    },
                );
                self.card_index.append(&mut build_card_index(
                    deck_id,
                    self.deck_in_progress.as_ref().expect("oops"),
                ));
                self.deck_in_progress = None;
                self.deck_output = String::new();
                self.deck_name = String::new();
                self.section = Section::Decks;
                self.deck_input_content = text_editor::Content::new();
            }
            AppMessage::ViewDeck(id) => {
                self.section = Section::ViewDeck(id);
                if let Some(deck) = self.decks.get(&id) {
                    return Command::batch(
                        deck.cards
                            .iter()
                            .filter(|&c| self.image_cache.get(&c.card.name).is_none())
                            .map(|c| {
                                let card = c.card.clone();
                                Command::perform(
                                    async move { download_image(&card.name, &card.img).await },
                                    |res| AppMessage::UpdateImageCache(res.0, res.1),
                                )
                            }),
                    );
                }
            }
            AppMessage::DeleteDeck(id) => {
                self.decks.remove(&id);
                self.card_index = self
                    .card_index
                    .iter()
                    .filter(|&c| c.deck_id != id)
                    .cloned()
                    .collect();
            }
            AppMessage::Search(query) => {
                self.search_text = query;
                self.search_result = fuzzy_top_n(&self.search_text, &self.card_index, 10);
                return Command::batch(
                    self.search_result
                        .iter()
                        .filter(|&r| self.image_cache.get(&r.name).is_none())
                        .map(|r| {
                            let card = r.clone();
                            Command::perform(
                                async move { download_image(&card.name, &card.img).await },
                                |res| AppMessage::UpdateImageCache(res.0, res.1),
                            )
                        }),
                );
            }
            AppMessage::UpdateImageCache(name, bytes) => match bytes {
                Some(b) => {
                    self.image_cache.insert(name, b);
                }
                None => {}
            },
            AppMessage::AddCard(deck_id, card_name) => match self.decks.get_mut(&deck_id) {
                Some(deck) => {
                    deck.cards
                        .iter_mut()
                        .filter(|c| c.card.name == card_name)
                        .for_each(|c| c.current_quantity += 1);
                }
                None => {}
            },
            AppMessage::RemoveCard(deck_id, card_name) => match self.decks.get_mut(&deck_id) {
                Some(deck) => {
                    deck.cards
                        .iter_mut()
                        .filter(|c| c.card.name == card_name)
                        .for_each(|c| c.current_quantity -= 1);
                }
                None => {}
            },
            AppMessage::Import => {
                let file = match FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .show_open_single_file()
                {
                    Ok(f) => match f {
                        Some(f) => f,
                        None => return iced::Command::none(),
                    },
                    Err(_) => return iced::Command::none(),
                };

                let json = fs::read_to_string(file).unwrap();

                self.decks = serde_json::from_str(&json).unwrap();
                self.search_result = Vec::new();
                self.image_cache = HashMap::new();
                self.card_index = Vec::new();
                self.search_text = String::new();

                for deck in &self.decks {
                    self.card_index
                        .append(&mut build_card_index(deck.0.clone(), &deck.1.cards));
                }
            }
            AppMessage::Export => {
                let file = match FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .show_save_single_file()
                {
                    Ok(f) => match f {
                        Some(f) => f,
                        None => return iced::Command::none(),
                    },
                    Err(_) => return iced::Command::none(),
                };

                let json = serde_json::to_string(&self.decks).unwrap();

                _ = fs::write(file, json);
            }
        };

        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let btn_decks = widget::button("Decks")
            .width(Length::Fixed(100.))
            .on_press(AppMessage::ChangeSection(Section::Decks));
        let btn_newdeck = widget::button("New deck")
            .width(Length::Fixed(100.))
            .on_press(AppMessage::ChangeSection(Section::AddDeck));
        let btn_buildecks = widget::button("Build")
            .width(Length::Fixed(100.))
            .on_press(AppMessage::ChangeSection(Section::BuildDecks));

        let list_btn = column!(btn_decks, btn_newdeck, btn_buildecks);

        let content = match self.section {
            Section::Decks => view_decks(self),
            Section::AddDeck => view_add_deck(self),
            Section::ViewDeck(deck_id) => view_deck(self, deck_id),
            Section::BuildDecks => view_deck_builder(self),
        };

        row!(list_btn, content).into()
    }
}

fn view_decks(app: &App) -> AppElement {
    let btn_export = widget::button("Export").on_press(AppMessage::Export);
    let btn_import = widget::button("Import").on_press(AppMessage::Import);

    let row_buttons = row!(btn_export, btn_import);

    let col_decks = widget::column(app.decks.iter().map(|(k, v)| view_deck_general(k, v)));

    column!(row_buttons, col_decks).into()
}

fn view_deck_general<'a>(id: &'a Uuid, deck: &'a Deck) -> AppElement<'a> {
    let deck_total_cards = deck.cards.iter().fold(0, |acc, card| acc + card.quantity);
    let deck_current_cards = deck
        .cards
        .iter()
        .fold(0, |acc, card| acc + card.current_quantity);
    let deck_text = format!(
        "{} ({}/{} cards)",
        deck.name, deck_current_cards, deck_total_cards
    );
    let txt_name = widget::text(deck_text).width(Length::Fill);
    let btn_view = widget::button("View").on_press(AppMessage::ViewDeck(id.clone()));
    let btn_delete = widget::button("Delete").on_press(AppMessage::DeleteDeck(id.clone()));

    row!(txt_name, btn_view, btn_delete).into()
}

fn view_add_deck(app: &App) -> AppElement {
    let deck_input = widget::text_editor(&app.deck_input_content)
        .on_action(AppMessage::EditDeckInput)
        .height(400);

    let btn_analyze = widget::button("Analyze").on_press(AppMessage::AnalyzeDeck);

    let output = widget::scrollable(widget::text(&app.deck_output));

    let mut column = column!(deck_input, btn_analyze, output);

    if app.deck_in_progress.is_some() {
        let field_deck_name =
            widget::text_input("Deck name", &app.deck_name).on_input(AppMessage::UpdateDeckName);
        let btn_create_deck = widget::button("Create deck").on_press(AppMessage::CreateDeck);

        column = column.push(field_deck_name);
        column = column.push(btn_create_deck);
    }

    column.into()
}

fn view_deck_builder(app: &App) -> AppElement {
    let search_box =
        widget::text_input("search card...", &app.search_text).on_input(AppMessage::Search);

    let card_results = widget::scrollable(widget::column(
        app.search_result.iter().map(|c| view_card_result(app, c)),
    ))
    .width(Length::Fill);

    column!(search_box, card_results).into()
}

fn view_card_result<'a>(app: &'a App, card: &'a IndexedCard) -> AppElement<'a> {
    let img_bytes = app
        .image_cache
        .get(&card.name)
        .unwrap_or(&app.default_image);
    let img = widget::image::<Handle>(Handle::from_memory(img_bytes.clone()))
        .content_fit(iced::ContentFit::ScaleDown)
        .height(100);

    let deck = app.decks.get(&card.deck_id).unwrap();
    let card_in_deck = deck.cards.iter().find(|&c| c.card.name == card.name);
    let card_totals = match card_in_deck {
        Some(c) => format!("{}/{}", c.current_quantity, c.quantity),
        None => String::default(),
    };
    let card_info = widget::text(format!("{} ({})", card.name, deck.name));
    let card_totals = widget::text(card_totals);

    let should_allow_add = if card_in_deck.map(|c| c.current_quantity).unwrap_or(0)
        < card_in_deck.map(|c| c.quantity).unwrap_or(0)
    {
        Some(AppMessage::AddCard(card.deck_id, card.name.clone()))
    } else {
        None
    };
    let should_allow_remove = if card_in_deck.map(|c| c.current_quantity).unwrap_or(0) > 0 {
        Some(AppMessage::RemoveCard(card.deck_id, card.name.clone()))
    } else {
        None
    };

    let btn_add_card = widget::button("Add card").on_press_maybe(should_allow_add);
    let btn_remove_card = widget::button("Remove card").on_press_maybe(should_allow_remove);

    let card_col = column!(card_info, card_totals, row!(btn_add_card, btn_remove_card));

    row!(img, card_col).into()
}

fn view_deck<'a>(app: &'a App, deck_id: Uuid) -> AppElement<'a> {
    let deck = match app.decks.get(&deck_id) {
        Some(d) => d,
        None => return widget::text("oopsies").into(),
    };

    let txt_title = widget::text(&deck.name);

    let cards = widget::scrollable(widget::column(
        deck.cards
            .iter()
            .map(|c| view_card_in_deck(app, deck_id, c)),
    ))
    .width(Length::Fill);

    column!(txt_title, cards).into()
}

fn view_card_in_deck<'a>(
    app: &'a App,
    deck_id: Uuid,
    card_in_deck: &'a CardInDeck,
) -> AppElement<'a> {
    let img_bytes = app
        .image_cache
        .get(&card_in_deck.card.name)
        .unwrap_or(&app.default_image);
    let img = widget::image::<Handle>(Handle::from_memory(img_bytes.clone()))
        .content_fit(iced::ContentFit::ScaleDown)
        .height(100);

    let card_info = widget::text(card_in_deck.card.name.clone());
    let card_totals = widget::text(format!(
        "{}/{}",
        card_in_deck.current_quantity, card_in_deck.quantity
    ));

    let should_allow_add = if card_in_deck.current_quantity < card_in_deck.quantity {
        Some(AppMessage::AddCard(
            deck_id.clone(),
            card_in_deck.card.name.clone(),
        ))
    } else {
        None
    };
    let should_allow_remove = if card_in_deck.current_quantity > 0 {
        Some(AppMessage::RemoveCard(
            deck_id.clone(),
            card_in_deck.card.name.clone(),
        ))
    } else {
        None
    };

    let btn_add_card = widget::button("Add card").on_press_maybe(should_allow_add);
    let btn_remove_card = widget::button("Remove card").on_press_maybe(should_allow_remove);

    let card_col = column!(card_info, card_totals, row!(btn_add_card, btn_remove_card));
    row!(img, card_col).into()
}

fn build_card_index(deck_id: Uuid, cards: &Vec<CardInDeck>) -> Vec<IndexedCard> {
    cards
        .iter()
        .map(|c| IndexedCard {
            name: c.card.name.clone(),
            img: c.card.img.clone(),
            deck_id: deck_id,
        })
        .collect()
}

fn fuzzy_top_n(query: &str, cards: &Vec<IndexedCard>, top: usize) -> Vec<IndexedCard> {
    let matcher = SkimMatcherV2::default();
    let mut sorted = cards
        .iter()
        .map(|c| (c, matcher.fuzzy_match(&c.name, query).unwrap_or(0)))
        .collect::<Vec<_>>();
    sorted.sort_by(|(_, a), (_, b)| b.cmp(a));

    let max_results = if top < sorted.len() {
        top
    } else {
        sorted.len()
    };

    sorted[0..max_results]
        .to_vec()
        .iter()
        .map(|&(a, _)| a)
        .cloned()
        .collect()
}

async fn download_image(card_name: &str, card_img: &str) -> (String, Option<Bytes>) {
    let request = reqwest::get(card_img).await.ok();
    let img = match request {
        Some(res) => res.bytes().await.ok(),
        None => None,
    };
    (card_name.to_owned(), img)
}

impl Default for App {
    fn default() -> Self {
        Self {
            decks: Default::default(),
            section: Section::Decks,
            deck_input_content: text_editor::Content::new(),
            deck_output: Default::default(),
            deck_in_progress: Default::default(),
            deck_name: Default::default(),
            search_text: Default::default(),
            card_index: Default::default(),
            search_result: Default::default(),
            image_cache: Default::default(),
            default_image: Bytes::from_static(DEFAULT_IMAGE),
        }
    }
}
