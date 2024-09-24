use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::{gtk, FactorySender};
use relm4::adw::prelude::{ButtonExt, OrientableExt};
use relm4::gtk::Orientation;
use crate::models::Card;

#[derive(Debug)]
pub struct CardModel {
    name: String,
    image: String
}

#[derive(Debug)]
pub enum CardMsg{

}

#[derive(Debug)]
pub enum CardOutput{
    Add(DynamicIndex),
    Sub(DynamicIndex),
}

impl CardModel{
    pub fn default_parent() -> gtk::Box{
        gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .build()
    } 
}

#[relm4::factory(pub)]
impl FactoryComponent for CardModel{
    type Init = Card;
    type Input = CardMsg;
    type Output = CardOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    
    view!{
        #[root]
        gtk::Box{
            set_orientation: Orientation::Horizontal,
            
            gtk::Label{
                #[watch]
                set_label: &self.name 
            },
            
            gtk::Button{
                set_label: "1",
            }
        }
    }

    fn init_model(init: Self::Init, index: &Self::Index, sender: FactorySender<Self>) -> Self {
        Self{
            name: init.name,
            image: init.img,
        }
    }
    
}


