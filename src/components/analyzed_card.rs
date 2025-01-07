use relm4::factory::{FactoryComponent, FactoryView};
use relm4::{adw, gtk, Component, ComponentController, Controller, FactorySender, RelmWidgetExt};
use relm4::adw::prelude::{BoxExt, PreferencesRowExt, WidgetExt};
use relm4::gtk::prelude::EditableExt;
use relm4_components::web_image::WebImage;
use crate::models::CardInDeck;

#[derive(Debug)]
pub struct AnalyzedCard {
    card: CardInDeck,
    card_image: Controller<WebImage>,
}

#[derive(Debug)]
pub enum AnalyzedCardInput {}
#[derive(Debug)]
pub enum AnalyzedCardOutput {}

#[relm4::factory(pub)]
impl FactoryComponent for AnalyzedCard {
    type Init = CardInDeck;
    type Input = AnalyzedCardInput;
    type Output = AnalyzedCardOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    
    view!{
        #[root]
        gtk::Box {
            set_align: gtk::Align::Fill,
            #[local_ref]
            card_image -> gtk::Box{
                set_hexpand: true,
                set_vexpand: true,
                set_width_request: 109,
            },
            adw::PreferencesGroup{
                adw::EntryRow{
                    set_hexpand: true,
                    set_title: "Card name",
                    set_text: &self.card.card.name,
                    
                },
                adw::SpinRow{
                    set_title: "Qty",
                    set_value: self.card.quantity as f64,
                    set_range: (0.0, 9999.0)
                }
            }
        }
    }

    fn init_model(init: Self::Init, index: &Self::Index, sender: FactorySender<Self>) -> Self {
        let img = init.card.img.clone();
        
        AnalyzedCard{
            card: init,
            card_image: WebImage::builder()
                .launch(img)
                .detach()
        }
    }

    fn init_widgets(&mut self, index: &Self::Index, root: Self::Root, returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget, sender: FactorySender<Self>) -> Self::Widgets {
        let card_image = self.card_image.widget();
        
        let widgets = view_output!();
        widgets
    }
    
}