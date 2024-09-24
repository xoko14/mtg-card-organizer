use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use relm4::adw::{ApplicationWindow, HeaderBar, NavigationPage, NavigationSplitView, NavigationView, ToolbarView, ViewStack, ViewStackPage, ViewSwitcher};
use relm4::adw::ffi::AdwNavigationView;
use relm4::adw::glib::clone;
use relm4::adw::glib::ffi::g_ascii_tolower;
use relm4::adw::prelude::{ActionableExt, ActionableExtManual, AdwApplicationWindowExt, BoxExt, ButtonExt, GtkWindowExt};
use relm4::factory::{DynamicIndex, FactoryVecDeque};
use relm4::gtk::Orientation;
use relm4_icons::icon_names;
use crate::components::card::{CardModel, CardOutput};
use crate::models::Card;

pub struct AppModel {
    query_results: FactoryVecDeque<CardModel>
}
#[derive(Debug)]
pub enum AppInput {
    AddCard(DynamicIndex),
    SubCard(DynamicIndex),
    Test,
}

pub struct AppWidgets {

}


impl SimpleComponent for AppModel {
    type Input = AppInput;
    type Output = ();
    type Init = ();
    type Root = ApplicationWindow;
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root {
        ApplicationWindow::builder()
            .default_width(1000)
            .default_height(800)
            .build()
    }

    fn init(init: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = AppModel {
            query_results: FactoryVecDeque::builder()
                .launch(CardModel::default_parent())
                .forward(sender.input_sender(), |output| match output{
                    CardOutput::Add(idx) => AppInput::AddCard(idx),
                    CardOutput::Sub(idx) => AppInput::AddCard(idx),
                })
        };

        let header = HeaderBar::builder()
            .build();

        let navigation_view = NavigationView::builder()
            .animate_transitions(true)
            .build();
        let view_stack = ViewStack::builder()
            .build();

        let header_switcher= ViewSwitcher::builder()
            .stack(&view_stack)
            .build();
        header.set_title_widget(Some(&header_switcher));
        let btn_add_card = gtk::Button::builder()
            .icon_name(icon_names::PLUS_SQUARE_OUTLINE)
            .build();
        btn_add_card.set_action_name(Some("navigation.push"));
        btn_add_card.set_action_target(Some("create_deck"));
        header.pack_start(&btn_add_card);
        let toolbar_view = ToolbarView::builder()
            .build();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&view_stack));
        let main_page = NavigationPage::builder()
            .child(&toolbar_view)
            .build();
        navigation_view.add(&main_page);

        let create_deck_box = gtk::Box::builder()
            .build();
        let create_deck_toolbar = ToolbarView::builder()
            .build();
        let create_deck_navigation = HeaderBar::builder()
            .build();
        create_deck_toolbar.add_top_bar(&create_deck_navigation);
        create_deck_toolbar.set_content(Some(&create_deck_box));
        let create_deck_page = NavigationPage::builder()
            .tag("create_deck")
            .child(&create_deck_toolbar)
            .build();
        navigation_view.add(&create_deck_page);


        let decks_split = NavigationSplitView::builder()
            .build();
        let decks_page = view_stack.add(&decks_split);
        decks_page.set_name(Some("decks"));
        decks_page.set_title(Some("Decks"));
        decks_page.set_icon_name(Some(icon_names::MAILBOX));

        let cards_content = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .build();
        let cards_page = view_stack.add(&cards_content);
        cards_page.set_name(Some("cards"));
        cards_page.set_title(Some("Cards"));
        cards_page.set_icon_name(Some(icon_names::SMARTCARD));

        let searchentry = gtk::SearchEntry::builder()
            .hexpand(true)
            .build();
        let searchbar = gtk::SearchBar::builder()
            .build();
        searchbar.connect_entry(&searchentry);
        searchbar.set_child(Some(&searchentry));
        searchbar.set_key_capture_widget(Some(&cards_content));
        searchbar.set_search_mode(true);
        cards_content.append(&searchbar);
        let scrollable = gtk::ScrolledWindow::builder()
            .child(model.query_results.widget())
            .hexpand(true)
            .vexpand(true)
            .build();
        cards_content.append(&scrollable);

        root.set_content(Some(&navigation_view));

        let widgets = Self::Widgets {};


        ComponentParts {model, widgets}
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            AppInput::AddCard(_) => {}
            AppInput::SubCard(_) => {}
            AppInput::Test => {
                self.query_results.guard().push_front(Card{name: "test".to_string(), img: "test".to_string()});
            }
        };
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
    }
}