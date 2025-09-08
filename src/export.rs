use std::fs;

use crate::db::DB;
use crate::db::get_category_cards;
use crate::db::Card;
use chrono::Datelike;
use chrono::Timelike;
use chrono::{Local};
use quick_xml::*;


pub fn export_pleco(category: &str, db: &DB) {
    let category_count = get_category_cards(db);
    let local_time = Local::now();
    let export_file = fs::File::create(format!("{}-{}-{}-{}_{}:{}.xml", category, local_time.year(), local_time.month(), local_time.day(), local_time.hour(), local_time.minute()));
    match export_file {
        Ok(file) => {
            match category_count.get(category) {
                Some(cards) => export_cards_to_pleco(cards, file),
                None => println!("Category {} not found. Available Categories: {:?}", category, category_count.keys())
            }
        }
        Err(error) => {
            println!("Failed to create export file: {:?}", error);
        }
    }
}

pub fn export_cards_to_pleco(cards: &Vec<Card>, file: fs::File) {
    let mut xml_writer = Writer::new(file);
    let decl = events::BytesDecl::new("1.0", Some("UTF-8"), None);
    assert!(xml_writer.write_event(events::Event::Decl(decl)).is_ok());
    let mut elem = events::BytesStart::new("plecoflash");
    elem.push_attribute(("formatversion", "2"));
    elem.push_attribute(("creator", "Pleco User 19080708"));
    elem.push_attribute(("generator", "Pleco 2.0 Flashcard Exporter"));
    elem.push_attribute(("platform", "iPhone OS"));
    elem.push_attribute(("created", "1757266198"));
    assert!(xml_writer.write_event(events::Event::Start(elem)).is_ok());
    assert!(xml_writer.write_event(events::Event::Start(events::BytesStart::new("cards"))).is_ok());

    let created= Local::now().timestamp().to_string();
    for card in cards {
        let mut card_event: events::BytesStart<'_> = events::BytesStart::new("card");
        card_event.push_attribute(("language", "chinese"));
        card_event.push_attribute(("created", created.as_str()));
        card_event.push_attribute(("modified", created.as_str()));
        assert!(xml_writer.write_event(events::Event::Start(card_event)).is_ok());

        assert!(xml_writer.write_event(events::Event::Start(events::BytesStart::new("entry"))).is_ok());

        let mut headword = events::BytesStart::new("headword");
        headword.push_attribute(("charset", "sc"));
        assert!(xml_writer.write_event(events::Event::Start(headword)).is_ok());

        assert!(xml_writer.write_event(events::Event::Text(events::BytesText::new(&card.character))).is_ok());

        assert!(xml_writer.write_event(events::Event::End(events::BytesEnd::new("headword"))).is_ok());

        assert!(xml_writer.write_event(events::Event::End(events::BytesEnd::new("entry"))).is_ok());
        // dict-id?

        let mut catassign = events::BytesStart::new("catassign");
        match card.category.get(0) {
            Some(category) => catassign.push_attribute(("category", category.as_str())),
            None => println!("No category for {}", card.character),
        }
        assert!(xml_writer.write_event(events::Event::Empty(catassign)).is_ok());

        assert!(xml_writer.write_event(events::Event::End(events::BytesEnd::new("card"))).is_ok());
    }

    assert!(xml_writer.write_event(events::Event::End(events::BytesEnd::new("cards"))).is_ok());
    assert!(xml_writer.write_event(events::Event::End(events::BytesEnd::new("plecoflash"))).is_ok());
}