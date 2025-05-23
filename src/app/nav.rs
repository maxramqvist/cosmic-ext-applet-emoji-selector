use super::Message;

use cosmic::iced_futures::Subscription;

pub(crate) fn group_from_key(key: u8) -> Option<emojis::Group> {
    use emojis::Group::*;
    let group = match key {
        b'1' => SmileysAndEmotion,
        b'2' => PeopleAndBody,
        b'3' => AnimalsAndNature,
        b'4' => FoodAndDrink,
        b'5' => TravelAndPlaces,
        b'6' => Activities,
        b'7' => Objects,
        b'8' => Symbols,
        b'9' => Flags,
        _ => return None,
    };
    return Some(group);
}

pub(crate) fn key_from_group(group: Option<emojis::Group>) -> u8 {
    use emojis::Group::*;
    let group = match group {
        Some(SmileysAndEmotion) => b'1',
        Some(PeopleAndBody) => b'2',
        Some(AnimalsAndNature) => b'3',
        Some(FoodAndDrink) => b'4',
        Some(TravelAndPlaces) => b'5',
        Some(Activities) => b'6',
        Some(Objects) => b'7',
        Some(Symbols) => b'8',
        Some(Flags) => b'9',
        None => b'0',
    };
    return group;
}

pub(crate) fn subscription() -> Subscription<Message> {
    use cosmic::iced::event;
    cosmic::iced_futures::event::listen_with(|event, status| {
        if status == event::Status::Captured {
            return None;
        }
        let event::Event::Keyboard(key_event) = event else {
            return None;
        };

        let cosmic::iced_runtime::keyboard::Event::KeyReleased { key, .. } = key_event else {
            return None;
        };
        match key {
            cosmic::iced_runtime::keyboard::Key::Named(key_named) => match key_named {
                cosmic::iced::keyboard::key::Named::Escape => return Some(Message::Exit),
                cosmic::iced::keyboard::key::Named::ArrowRight => return Some(Message::ArrowRight),
                cosmic::iced::keyboard::key::Named::ArrowLeft => return Some(Message::ArrowLeft),
                cosmic::iced::keyboard::key::Named::ArrowDown => {
                    return Some(Message::_ScrollPixels(-50.0))
                }
                cosmic::iced::keyboard::key::Named::ArrowUp => {
                    return Some(Message::_ScrollPixels(50.0))
                }

                cosmic::iced::keyboard::key::Named::End => return Some(Message::Snap(1.0)),
                cosmic::iced::keyboard::key::Named::Home => return Some(Message::Snap(-1.0)),
                cosmic::iced::keyboard::key::Named::PageDown => return Some(Message::Snap(0.15)),
                cosmic::iced::keyboard::key::Named::PageUp => return Some(Message::Snap(-0.15)),

                _ => {}
            },
            cosmic::iced_runtime::keyboard::Key::Character(key_character) => {
                if key_character == "/" {
                    return Some(Message::FocusTextInput);
                }
                if key_character.len() == 1 && key_character.as_bytes()[0].is_ascii_digit() {
                    return Some(Message::Group(group_from_key(key_character.as_bytes()[0])));
                }
            }
            _ => {}
        }
        return None;
    })
}
