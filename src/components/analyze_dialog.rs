use relm4::{gtk, view, Component, ComponentParts, ComponentSender, RelmWidgetExt};
use relm4::adw::{Dialog, HeaderBar, ToolbarView};
use relm4::adw::prelude::*;
use relm4::gtk::{PolicyType, Widget};
use relm4::prelude::FactoryVecDeque;
use crate::components::analyzed_card::AnalyzedCard;
use crate::models::CardInDeck;

#[derive(Debug)]
pub struct AnalyzeDialog {
    deck_name: String,
    cards: Vec<CardInDeck>,
    analyze_results: FactoryVecDeque<AnalyzedCard>
}

#[derive(Debug)]
pub enum AnalyzeDialogInput{
    Open,
    Close,
    SetCards(Vec<CardInDeck>),
    SaveRequest
}

#[derive(Debug)]
pub enum AnalyzeDialogOutput{
    SaveDeck(Vec<CardInDeck>)
}

#[relm4::component(pub)]
impl Component for AnalyzeDialog {
    type Init = ();
    type CommandOutput = ();
    type Input = AnalyzeDialogInput;
    type Output = AnalyzeDialogOutput;
    
    view!{
        #[root]
        Dialog{
            set_height_request: 600,
            set_width_request: 394,
            #[wrap(Some)]
            set_child = &ToolbarView{
                add_top_bar = &HeaderBar{
                    
                },
                #[wrap(Some)]
                set_content = &gtk::Box{
                    set_hexpand: true,
                    set_vexpand: true,
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::ScrolledWindow{
                        set_vexpand: true,
                        set_hexpand: true,
                        set_hscrollbar_policy: PolicyType::Never,
                        model.analyze_results.widget() -> &gtk::Box{
                            set_orientation: gtk::Orientation::Vertical,
                            set_align: gtk::Align::Start,
                        }
                    },
                    gtk::Button{
                        set_label: "Save and continue",
                        connect_clicked[sender, root] => move |_| {
                            root.close();
                            sender.input(AnalyzeDialogInput::SaveRequest);
                        },
                    }
                }
            },
        }
    }

    fn init(init: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = AnalyzeDialog{
            deck_name: String::new(),
            cards: Vec::new(),
            analyze_results: FactoryVecDeque::builder()
                .launch(gtk::Box::builder().spacing(8).build())
                .detach()
        };
        
        let widgets = view_output!();
        
        ComponentParts{model, widgets}
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            AnalyzeDialogInput::Open => {
                root.present(Some(root))
            },
            AnalyzeDialogInput::Close => {
                root.close();
            }
            AnalyzeDialogInput::SetCards(cards) => {
                let mut ui_cards = self.analyze_results.guard();
                ui_cards.clear();
                for card in cards {
                    ui_cards.push_front(card);
                }
            }
            AnalyzeDialogInput::SaveRequest => {
                _ = sender.output(AnalyzeDialogOutput::SaveDeck(self.cards.clone()));
            }
        }
    }

    fn update_cmd(&mut self, message: Self::CommandOutput, sender: ComponentSender<Self>, root: &Self::Root) {
        todo!()
    }
}