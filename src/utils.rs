use crate::{config::EmojiOrdering, google_ordering::GOOGLE_ORDERING};

pub(crate) fn all_emojis(
    emoji_ordering: EmojiOrdering,
) -> impl Iterator<Item = &'static emojis::Emoji> {
    let all_emojis = match emoji_ordering {
        EmojiOrdering::Google => Either::Left(google_emojis()),
        EmojiOrdering::Unicode => Either::Right(unicode_emojis()),
    };

    return all_emojis;
}
pub(crate) fn all_emojis_in_group(
    emoji_ordering: EmojiOrdering,
    group: emojis::Group,
) -> impl Iterator<Item = &'static emojis::Emoji> {
    let all_emojis = match emoji_ordering {
        EmojiOrdering::Google => Either::Left(google_emojis_in_group(group)),
        EmojiOrdering::Unicode => Either::Right(unicode_emojis_in_group(group)),
    };

    return all_emojis;
}

pub(crate) fn all_emojis_in_optional_group(
    emoji_ordering: EmojiOrdering,
    group: Option<emojis::Group>,
) -> impl Iterator<Item = &'static emojis::Emoji> {
    match group {
        Some(group) => return Either::Left(all_emojis_in_group(emoji_ordering, group)),
        None => return Either::Right(all_emojis(emoji_ordering)),
    };
}
pub(crate) fn unicode_emojis() -> impl Iterator<Item = &'static emojis::Emoji> {
    let all_emojis = unicode_emojis_internal(emojis::iter());
    return all_emojis;
}
pub(crate) fn unicode_emojis_in_group(
    group: emojis::Group,
) -> impl Iterator<Item = &'static emojis::Emoji> {
    let all_emojis = unicode_emojis_internal(group.emojis());
    return all_emojis;
}

fn unicode_emojis_internal(
    iter: impl Iterator<Item = &'static emojis::Emoji>,
) -> impl Iterator<Item = &'static emojis::Emoji> {
    let flat_map = iter.flat_map(|emoji| match emoji.skin_tones() {
        Some(skin_tones) => Either::Left(skin_tones),
        None => Either::Right(std::iter::once(emoji)),
    });
    flat_map
}

pub(crate) fn google_emojis() -> impl Iterator<Item = &'static emojis::Emoji> {
    google_emojis_internal(GOOGLE_ORDERING)
}
fn google_emojis_internal(
    slice: &'static [&'static str],
) -> impl Iterator<Item = &'static emojis::Emoji> {
    slice.iter().map(|&emoji| emojis::get(emoji).unwrap())
}
pub(crate) fn google_emojis_in_group(
    group: emojis::Group,
) -> impl Iterator<Item = &'static emojis::Emoji> {
    let (start, end) = match group {
        emojis::Group::SmileysAndEmotion => crate::google_ordering::SMILEYS_AND_EMOTIONS,
        emojis::Group::PeopleAndBody => crate::google_ordering::PEOPLE,
        emojis::Group::AnimalsAndNature => crate::google_ordering::ANIMALS_AND_NATURE,
        emojis::Group::FoodAndDrink => crate::google_ordering::FOOD_AND_DRINK,
        emojis::Group::TravelAndPlaces => crate::google_ordering::TRAVEL_AND_PLACES,
        emojis::Group::Activities => crate::google_ordering::ACTIVITIES_AND_EVENTS,
        emojis::Group::Objects => crate::google_ordering::OBJECTS,
        emojis::Group::Symbols => crate::google_ordering::SYMBOLS,
        emojis::Group::Flags => crate::google_ordering::FLAGS,
    };
    return google_emojis_internal(&GOOGLE_ORDERING[start..start + end]);
}

enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}

impl<Left, Right> Iterator for Either<Left, Right>
where
    Left: Iterator,

    Right: Iterator<Item = Left::Item>,
{
    type Item = Left::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(left) => left.next(),
            Either::Right(right) => right.next(),
        }
    }
}

pub(crate) struct Lines<'a> {
    pub(crate) s: &'a str,
}

// reimplenting str::Lines
impl<'a> Lines<'a> {
    pub(crate) fn new(s: &'a str) -> Self {
        Self { s }
    }
    // because of this method
    pub(crate) fn remainder(&self) -> &'a str {
        self.s
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.s == "" {
            return None;
        }
        let Some((mut line, rest)) = self.s.split_once("\n") else {
            let line = self.s;
            self.s = "";
            return Some(line);
        };
        line = line.strip_suffix("\r").unwrap_or(line);
        self.s = rest;
        return Some(line);
    }
}
