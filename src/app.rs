use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use relm4::adw::{ApplicationWindow, HeaderBar, NavigationPage, NavigationSplitView, NavigationView, ToolbarView, ViewStack, ViewSwitcher, Dialog};
use relm4::adw::prelude::*;
use relm4::binding::{ConnectBindingExt, StringBinding};
use relm4::factory::{DynamicIndex, FactoryVecDeque};
use relm4::gtk::Orientation;
use crate::components::card::{CardModel, CardOutput};
use crate::models::{Card, CardInDeck};
use crate::mtg;
use crate::mtg::CardErrorInsight;
use crate::icon_names;

pub struct AppModel {
    query_results: FactoryVecDeque<CardModel>,
    is_deck_dialog_open: bool,
    card_list_raw: String,
}
#[derive(Debug)]
pub enum AppInput {
    AddCard(DynamicIndex),
    SubCard(DynamicIndex),
    ChangedDecklist(String),
    AnalyzeDeck,
    Test,
}

#[derive(Debug)]
pub enum CommandMsg{
    DeckProcessResult(Vec<CardInDeck>, Vec<CardErrorInsight>)
}

#[relm4::component(pub)]
impl Component for AppModel {
    type Input = AppInput;
    type Output = ();
    type Init = ();
    type CommandOutput = CommandMsg;

    view!{
        #[root]
        ApplicationWindow{
            set_default_width: 1000,
            set_default_height: 800,

            #[wrap(Some)]
            set_content = &NavigationView::builder().build(){
                set_animate_transitions: true,

                NavigationPage{
                    ToolbarView{
                        add_top_bar = &HeaderBar::builder().build(){
                            #[wrap(Some)]
                            set_title_widget = &ViewSwitcher{
                                set_stack: Some(&main_view_stack)
                            },
                            pack_start = &gtk::Button::builder().build(){
                                set_action_name: Some("navigation.push"),
                                set_action_target: Some("create_deck"),
                                set_icon_name: icon_names::PLUS_SQUARE_OUTLINE
                            }
                        },
                        #[wrap(Some)]
                        set_content: main_view_stack = &ViewStack{
                            add = &NavigationSplitView::builder().build(){

                            } -> {
                                set_name: Some("decks"),
                                set_title: Some("Decks"),
                                set_icon_name: Some(icon_names::MAILBOX),
                            },
                            add: search_widget = &gtk::Box{
                                set_orientation: Orientation::Vertical,
                                gtk::SearchBar{
                                    #[wrap(Some)]
                                    set_child: cards_searchentry = &gtk::SearchEntry{
                                        set_hexpand:true,
                                    },
                                    connect_entry: &cards_searchentry,
                                    set_key_capture_widget: Some(&search_widget),
                                    set_search_mode: true,
                                },
                                gtk::ScrolledWindow{
                                    set_child: Some(model.query_results.widget()),
                                    set_vexpand: true,
                                },
                            } -> {
                                set_name: Some("cards"),
                                set_title: Some("Cards"),
                                set_icon_name: Some(icon_names::SMARTCARD),
                            }
                        },
                    }
                },

                NavigationPage{
                    set_title: "Create deck",
                    set_tag: Some("create_deck"),
                    #[wrap(Some)]
                    set_child: toolbar_new_deck = &ToolbarView{
                        add_top_bar = &HeaderBar::builder().build(){

                        },

                        gtk::Box{
                            set_orientation: Orientation::Vertical,
                            set_spacing: 8,
                            set_margin_all: 8,

                            gtk::Entry{
                                set_placeholder_text: Some("Deck name"),
                            },

                            gtk::ScrolledWindow{
                                set_vexpand: true,
                                #[name = "decklist_text"]
                                gtk::TextView{
                                    #[wrap(Some)]
                                    set_buffer = &gtk::TextBuffer{
                                        connect_changed[sender] => move |buffer| {
                                            let (start, end) = buffer.bounds();
                                            let text = buffer.text(&start, &end, false).as_str().to_owned();
                                            sender.input(AppInput::ChangedDecklist(text))
                                        } 
                                    }
                                },
                            },
                            gtk::Button{
                                set_label: "Analyze deck"
                            },
                        }
                    }
                },

            },
        },
        
        //deck_dialog = Dialog{
        //    #[track(self.is_deck_dialog_open)]
        //    present: Some(&toolbar_new_deck),
        //    #[track(!self.is_deck_dialog_open)]
        //    close: ()
        //}
    }

    fn init(init: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = AppModel {
            query_results: FactoryVecDeque::builder()
                .launch(CardModel::default_parent())
                .forward(sender.input_sender(), |output| match output{
                    CardOutput::Add(idx) => AppInput::AddCard(idx),
                    CardOutput::Sub(idx) => AppInput::AddCard(idx),
                }),
            is_deck_dialog_open: false,
            card_list_raw: String::new(),
        };

        let widgets = view_output!();

        ComponentParts {model, widgets}
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            AppInput::AddCard(_) => {}
            AppInput::SubCard(_) => {}
            AppInput::Test => {
                self.query_results.guard().push_front(Card{name: "test".to_string(), img: "test".to_string()});
            }
            AppInput::AnalyzeDeck => {
                let decklist = self.card_list_raw.clone();
                
                sender.oneshot_command(async move {
                    let (cards, errors) = mtg::process_decklist(decklist).await;
                    CommandMsg::DeckProcessResult(cards, errors)
                })
            }
            _ => {}
        };
    }

}