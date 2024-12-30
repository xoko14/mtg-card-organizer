use relm4::{gtk, view, Component, ComponentParts, ComponentSender};
use relm4::adw::{Dialog, HeaderBar, ToolbarView};
use relm4::adw::prelude::*;
use relm4::gtk::Widget;
use crate::models::CardInDeck;

#[derive(Debug)]
pub struct AnalyzeDialog {
    deck_name: String,
    cards: Vec<CardInDeck>,
}

#[derive(Debug)]
pub enum AnalyzeDialogInput{
    Open,
    Close,
}

#[derive(Debug)]
pub enum AnalyzeDialogOutput{
    
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
            #[wrap(Some)]
            set_child = &ToolbarView{
                add_top_bar = &HeaderBar{
                    
                }
            }
        }
    }

    fn init(init: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = AnalyzeDialog{
            deck_name: String::new(),
            cards: Vec::new(),
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
        }
    }

    fn update_cmd(&mut self, message: Self::CommandOutput, sender: ComponentSender<Self>, root: &Self::Root) {
        todo!()
    }
}