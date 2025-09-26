// SPDX-License-Identifier: MPL-2.0
// 2024 - Dominic Gerhauser and contributors

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};

use crate::config::{Annotation, ClickMode, SkinToneMode};
use crate::config::{Config, CONFIG_VERSION};
#[allow(unused_imports)]
use crate::fl;
use crate::{utils, widget_copy};
use aho_corasick::AhoCorasick;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use cosmic::app::Core;
use cosmic::iced;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
#[allow(unused_imports)]
use cosmic::iced::{alignment, Alignment, Length};
use cosmic::iced::{Command, Limits};
use cosmic::iced_futures::Subscription;
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::iced_widget::scrollable;
use cosmic::widget::{self};
use cosmic::{cosmic_config, iced_core};
use cosmic::{Apply, Element, Theme};
use cosmic_time::Timeline;
pub const ID: &str = "dev.dominiccgeh.CosmicAppletEmojiSelector";
const ICON: &str = ID;
pub struct Window {
    snap: widget_copy::scrollable::RelativeOffset,
    viewport: Option<widget_copy::scrollable::Viewport>,
    all_emojis_aho_corasick: AhoCorasick,
    fuzzy_matcher: SkimMatcherV2,
    emojis_filtered: Vec<&'static emojis::Emoji>,
    favorites_filtered: VecDeque<&'static emojis::Emoji>,
    annotations: HashMap<String, Annotation>,
    core: Core,
    popup: Option<Id>,
    config: Config,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,
    timeline: Timeline,
    selected_group: Option<emojis::Group>,
    search: String,
    scrollable_id: widget::Id,
    font_family: cosmic::iced::font::Font,
    emoji_hovered: Option<&'static emojis::Emoji>,
    text_input_id: widget::Id,
}
#[derive(Clone, Debug)]
pub enum Message {
    Config(Box<Config>),
    TogglePopup,
    PopupClosed(Id),
    Group(Option<emojis::Group>),
    EmojiCopy(&'static emojis::Emoji, ClickMode),
    Search(String),
    Frame(std::time::Instant),
    EmojiHovered(&'static emojis::Emoji),
    Exit,
    FocusTextInput,
    Enter,
    ArrowRight,
    ArrowLeft,
    ScrollViewport(Box<widget_copy::scrollable::Viewport>),
    _ScrollPixels(f32),
    Snap(f32),
    ToggleColorButton(usize),
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
    pub annotations: HashMap<String, Annotation>,
    pub toggle_on_start: bool,
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        flags: Self::Flags,
    ) -> (Self, Command<cosmic::app::Message<Self::Message>>) {
        let selected_group = None;
        let config = flags.config;
        let font_family =
            iced::Font::with_name(Box::leak(config.font_family.clone().into_boxed_str()));

        let mut all_emojis = Vec::from_iter(utils::all_emojis(config.emoji_ordering));

        all_emojis.shrink_to_fit();
        let all_emojis_ac = AhoCorasick::builder()
            .match_kind(aho_corasick::MatchKind::LeftmostLongest)
            .build(&all_emojis)
            .unwrap();
        let mut annotations = flags.annotations;
        annotations.shrink_to_fit();

        let emojis_filtered = all_emojis.to_vec();
        let window = Window {
            snap: Default::default(),
            viewport: None,
            all_emojis_aho_corasick: all_emojis_ac,
            fuzzy_matcher: SkimMatcherV2::default(),
            emojis_filtered,
            favorites_filtered: VecDeque::new(),
            font_family,
            scrollable_id: widget::Id::unique(),
            selected_group,
            core,
            config,
            config_handler: flags.config_handler,
            popup: None,
            search: String::new(),
            timeline: Timeline::new(),
            emoji_hovered: None,
            text_input_id: widget::Id::unique(),
            annotations: annotations,
        };

        let initial_command = if flags.toggle_on_start {
            Command::batch([
                cosmic::command::message(Message::Search(String::new())),
                cosmic::command::message(Message::TogglePopup),
            ])
        } else {
            cosmic::command::message(Message::Search(String::new()))
        };

        (window, initial_command)
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Command<cosmic::app::Message<Self::Message>> {
        // Helper for updating config values efficiently
        #[allow(unused_macros)]
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("failed to save config {:?}: {}", stringify!($name), err);
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        eprintln!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name),
                        );
                    }
                }
            };
        }

        match message {
            Message::_ScrollPixels(_pixels) => {
                // if let Some(viewport) = &self.viewport {
                //     let mut absolute = viewport.absolute_offset();
                //     absolute.y += pixels;

                //     return scrollable::scroll_to(self.scrollable_id.clone(), absolute);
                // }
            }
            Message::ScrollViewport(viewport) => {
                self.snap = viewport.relative_offset();
                self.viewport = Some(*viewport);
            }
            Message::Snap(snap) => {
                self.snap.x = (self.snap.x + snap).clamp(0.0, 1.0);
                self.snap.y = (self.snap.y + snap).clamp(0.0, 1.0);
                return scrollable::snap_to(self.scrollable_id.clone(), self.snap);
            }
            Message::ToggleColorButton(idx) => {
                let mut color_buttons = self.config.color_buttons.clone();
                if let Some(color_button) = color_buttons.get_mut(idx) {
                    color_button.active = !color_button.active;
                    let mut skin_tone_mode = self.config.skin_tone_mode;
                    skin_tone_mode.set(color_button.skin_tone_mode, color_button.active);
                    config_set!(skin_tone_mode, skin_tone_mode);
                    config_set!(color_buttons, color_buttons);
                    return cosmic::command::message(Message::Search(self.search.clone()));
                }
            }
            Message::Config(config) => {
                if *config != self.config {
                    if config.font_family != self.config.font_family {
                        self.font_family = iced::Font::with_name(Box::leak(
                            config.font_family.clone().into_boxed_str(),
                        ));
                    }
                    self.config = *config
                }
            }
            Message::Frame(now) => self.timeline.now(now),

            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::MAIN, new_id, None, None, None);
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(490.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::EmojiCopy(emoji, click_mode) => {
                let mut commands = Vec::new();
                if click_mode.intersects(ClickMode::APPEND_SEARCH) {
                    self.search.push_str(emoji.as_str());
                }
                if click_mode.intersects(ClickMode::COPY) {
                    if !click_mode.intersects(ClickMode::PRIVATE) {
                        let mut last_used = self.config.last_used.clone();
                        if let Some(idx) = last_used.iter().position(|&e| e == emoji) {
                            last_used.remove(idx);
                        }
                        last_used.push_front(emoji);
                        last_used.truncate(self.config.last_used_limit);
                        if let Some(idx) = self.favorites_filtered.iter().position(|&e| e == emoji)
                        {
                            self.favorites_filtered.remove(idx);
                        }
                        self.favorites_filtered.push_front(emoji);
                        self.favorites_filtered
                            .truncate(self.config.last_used_limit);
                        config_set!(last_used, last_used);
                    }
                    commands.push(iced::clipboard::write(emoji.to_string()))
                }

                if click_mode.intersects(ClickMode::CLOSE) {
                    commands.push(cosmic::command::message(Message::Exit));
                }
                if click_mode.intersects(ClickMode::CLEAR_SEARCH) {
                    self.search.clear();
                    commands.push(cosmic::command::message(Message::Search(String::new())))
                }
                return Command::batch(commands);
            }
            Message::Search(search) => {
                let mut search_filtered = String::with_capacity(search.len());
                self.all_emojis_aho_corasick.replace_all_with(
                    &search,
                    &mut search_filtered,
                    |_, _, _| true,
                );

                self.search = search;

                self.emoji_hovered = None;
                self.emojis_filtered.clear();
                self.favorites_filtered.clear();
                let skin_tones_config = self.config.skin_tone_mode;
                let skin_tones_exact = skin_tones_config.intersects(SkinToneMode::ALL_EXACT);
                let skin_tones_intersect =
                    skin_tones_config.intersects(SkinToneMode::FILTER_INTERSECT);
                let skin_tones_mode_new = if skin_tones_exact {
                    SkinToneMode::new_exact
                } else {
                    SkinToneMode::new
                };

                if search_filtered.is_empty() {
                    for ref emoji in utils::all_emojis_in_optional_group(
                        self.config.emoji_ordering,
                        self.selected_group,
                    ) {
                        let emjoji_skin_tone_mode = emoji
                            .skin_tone()
                            .map_or(SkinToneMode::NO_SKIN, skin_tones_mode_new);
                        let config_skin_tone_contains_emoji =
                            if skin_tones_exact || skin_tones_intersect {
                                skin_tones_config.intersects(emjoji_skin_tone_mode)
                            } else {
                                skin_tones_config.contains(emjoji_skin_tone_mode)
                            };

                        let emojis_in_conf = self.config.last_used.contains(emoji);
                        if config_skin_tone_contains_emoji {
                            self.emojis_filtered.push(emoji);
                        }
                        if emojis_in_conf {
                            self.favorites_filtered.push_back(emoji);
                        }
                    }
                } else {
                    let mut scored_emojis: Vec<(&'static emojis::Emoji, i64)> = Vec::new();
                    let mut scored_favorites: Vec<(&'static emojis::Emoji, i64)> = Vec::new();

                    for ref emoji in utils::all_emojis_in_optional_group(
                        self.config.emoji_ordering,
                        self.selected_group,
                    ) {
                        let emjoji_skin_tone_mode = emoji
                            .skin_tone()
                            .map_or(SkinToneMode::NO_SKIN, skin_tones_mode_new);
                        let config_skin_tone_contains_emoji =
                            if skin_tones_exact || skin_tones_intersect {
                                skin_tones_config.intersects(emjoji_skin_tone_mode)
                            } else {
                                skin_tones_config.contains(emjoji_skin_tone_mode)
                            };

                        let emojis_in_conf = self.config.last_used.contains(emoji);
                        let emoji_name = self.emoji_name_localized(emoji);

                        let mut best_score = None;

                        if let Some(score) = self.fuzzy_matcher.fuzzy_match(emoji_name, &search_filtered) {
                            best_score = Some(score);
                        }

                        if let Some(shortcode) = emoji.shortcode() {
                            if let Some(score) = self.fuzzy_matcher.fuzzy_match(shortcode, &search_filtered) {
                                best_score = Some(best_score.map_or(score, |s| s.max(score)));
                            }
                        }

                        if emoji_name.contains(&search_filtered) {
                            best_score = Some(best_score.map_or(1000, |s| s.max(1000)));
                        }

                        if let Some(score) = best_score {
                            if config_skin_tone_contains_emoji {
                                scored_emojis.push((emoji, score));
                            }
                            if emojis_in_conf {
                                scored_favorites.push((emoji, score));
                            }
                        }
                    }

                    scored_emojis.sort_by(|a, b| b.1.cmp(&a.1));
                    scored_favorites.sort_by(|a, b| b.1.cmp(&a.1));

                    self.emojis_filtered = scored_emojis.into_iter().map(|(emoji, _)| emoji).collect();
                    self.favorites_filtered = scored_favorites.into_iter().map(|(emoji, _)| emoji).collect();
                }
            }
            Message::Group(group) => return self.update_group(group),

            Message::EmojiHovered(emoji) => self.emoji_hovered = Some(emoji),
            Message::Exit => {
                self.emojis_filtered.shrink_to_fit();
                self.favorites_filtered.shrink_to_fit();
                if let Some(p) = self.popup.take() {
                    return destroy_popup(p);
                }
            }
            Message::Enter => {
                let emoji_opt = self.emoji_selected();
                if let Some(emoji) = emoji_opt {
                    return cosmic::command::message(Message::EmojiCopy(
                        emoji,
                        ClickMode::COPY | ClickMode::CLOSE,
                    ));
                }
            }
            Message::FocusTextInput => {
                return widget::text_input::focus(self.text_input_id.clone());
            }
            Message::ArrowRight => return self.nav_move_right(),
            Message::ArrowLeft => return self.nav_move_left(),
        };
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button(ICON)
            .on_press(Message::TogglePopup)
            .into()
    }
    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let mut content = widget::column::with_capacity(4)
            .padding([8, 8])
            .spacing(cosmic::theme::active().cosmic().space_xxs());

        let groups = self.group_icons();
        content = content.push(groups);

        let search = self.search();
        content = content.push(search);

        let preview = self.preview();
        content = content.push(preview);

        let emojis_section = self.emojis_section();
        content = content.push(emojis_section);

        return self.core.applet.popup_container(content).into();
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        let config = cosmic_config::config_subscription(
            std::any::TypeId::of::<ConfigSubscription>(),
            Self::APP_ID.into(),
            CONFIG_VERSION,
        )
        .map(|update| {
            if !update.errors.is_empty() {
                eprintln!(
                    "errors loading config {:?}: {:?}",
                    update.keys, update.errors
                );
            }
            Message::Config(Box::new(update.config))
        });

        let timeline = self
            .timeline
            .as_subscription()
            .map(|(_, now)| Message::Frame(now));

        Subscription::batch(vec![config, timeline, nav::subscription()])
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}

mod style;

impl Window {
    fn emoji_name_localized(&self, emoji: &'static emojis::Emoji) -> &str {
        let emoji_name = self
            .annotations
            .get(&emoji.as_str().replace(['\u{fe0f}', '\u{fe0e}'], ""))
            .and_then(|annotation| annotation.tts.first().map(String::as_str))
            .unwrap_or_else(|| emoji.name());
        emoji_name
    }

    fn update_group(
        &mut self,
        group: Option<emojis::Group>,
    ) -> Command<cosmic::app::Message<Message>> {
        self.emoji_hovered = None;
        self.selected_group = group;
        return Command::batch([
            scrollable::scroll_to(
                self.scrollable_id.clone(),
                scrollable::AbsoluteOffset::default(),
            ),
            cosmic::command::message(Message::Search(self.search.clone())),
        ]);
    }

    fn emojis_flex(
        &self,
        emojis_list: impl IntoIterator<Item = &'static emojis::Emoji>,
    ) -> widget::FlexRow<'static, Message> {
        let emojis_list = emojis_list.into_iter();
        let mut emojis_view = Vec::with_capacity(emojis_list.size_hint().0);

        let left_click_action = self.config.left_click_action;
        let right_click_action = self.config.right_click_action;
        let middle_click_action = self.config.middle_click_action;
        for emoji in emojis_list {
            let emoji_txt = widget::text(emoji.as_str())
                .size(25)
                .width(35)
                .height(35)
                .font(self.font_family)
                .shaping(iced_core::text::Shaping::Advanced)
                .wrap(iced::widget::text::Wrap::None)
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center);

            let mut emoji_btn: widget::Button<'_, Message> =
                widget::button(emoji_txt).style(cosmic::theme::Button::Transparent);
            if left_click_action != ClickMode::NONE {
                emoji_btn = emoji_btn.on_press(Message::EmojiCopy(emoji, left_click_action));
            }
            let mut emoji_mouse_area =
                widget_copy::MouseArea::new(emoji_btn).on_enter(Message::EmojiHovered(emoji));

            if right_click_action != ClickMode::NONE {
                emoji_mouse_area = emoji_mouse_area
                    .on_right_release(Message::EmojiCopy(emoji, right_click_action));
            }
            if middle_click_action != ClickMode::NONE {
                emoji_mouse_area = emoji_mouse_area
                    .on_middle_release(Message::EmojiCopy(emoji, middle_click_action));
            }
            emojis_view.push(emoji_mouse_area.into());
        }

        let flex_row = widget::flex_row(emojis_view)
            .row_spacing(0)
            .column_spacing(0)
            // .align_items(Alignment::Center)
            // .justify_content(widget::JustifyContent::Stretch)
            .width(Length::Fill);
        flex_row
    }

    fn search(&self) -> widget::TextInput<'_, Message> {
        let search = widget::search_input(fl!("search-for-emojis"), &self.search)
            .on_clear(Message::Search(String::new()))
            .id(self.text_input_id.clone())
            .on_paste(Message::Search)
            .on_input(Message::Search)
            .on_submit(Message::Enter);
        search
    }

    fn group_icons(&self) -> widget::Row<'static, Message> {
        let mut groups = widget::row::with_capacity(9).width(Length::Fill);
        use crate::style_copy::button;
        for group in emojis::Group::iter() {
            let is_selected = self.selected_group.is_some_and(|sel| sel == group);
            let buton_style = cosmic::theme::Button::Custom {
                active: Box::new(move |focused, theme| {
                    button::active(theme, focused, is_selected, button::Button::Icon)
                }),
                pressed: Box::new(move |focused, theme| {
                    button::pressed(theme, focused, is_selected, button::Button::Icon)
                }),
                hovered: Box::new(move |focused, theme| {
                    button::hovered(theme, focused, is_selected, button::Button::Icon)
                }),
                disabled: Box::new(|_theme| widget::button::Appearance::new()),
            };
            let group_btn =
                widget::button::icon(widget::icon::from_name(group_icon(group)).symbolic(true))
                    .font_size(20)
                    .icon_size(24)
                    .line_height(24)
                    .padding(cosmic::theme::active().cosmic().space_xxs())
                    .selected(is_selected)
                    .style(buton_style)
                    .on_press(Message::Group((!is_selected).then_some(group)))
                    .apply(widget::container)
                    .width(Length::Fill)
                    .center_y()
                    .center_x();

            groups = groups.push(group_btn);
        }
        groups
    }

    fn color_buttons(&self) -> widget::Row<'static, Message> {
        let color_buttons_conf = &self.config.color_buttons;
        let mut color_buttons = widget::row::with_capacity(color_buttons_conf.len());
        for (idx, color_button) in color_buttons_conf.iter().enumerate() {
            let color = color_button.color;
            let active = color_button.active;
            let button_style = cosmic::theme::Button::Custom {
                active: Box::new(move |_selected, theme| {
                    style::color_button_apperance(color, Some(active), theme)
                }),
                disabled: Box::new(move |theme| style::color_button_apperance(color, None, theme)),
                hovered: Box::new(move |_selected, theme| {
                    style::color_button_apperance(color, Some(active), theme)
                }),
                pressed: Box::new(move |_selected, theme| {
                    style::color_button_apperance(color, Some(active), theme)
                }),
            };

            color_buttons = color_buttons.push(
                widget::button(widget::horizontal_space(0.1))
                    .width(20)
                    .height(20)
                    .style(button_style)
                    .on_press(Message::ToggleColorButton(idx)),
            );
        }
        color_buttons
    }

    fn emoji_selected(&self) -> Option<&'static emojis::Emoji> {
        let emoji_opt = self
            .emoji_hovered
            .or_else(|| self.favorites_filtered.front().copied())
            .or_else(|| self.emojis_filtered.first().copied());
        emoji_opt
    }

    fn emojis_section(&self) -> widget::Container<'static, Message, Theme> {
        let mut emojis_section =
            widget::column::with_capacity(3).spacing(cosmic::theme::active().cosmic().space_xxs());
        if !self.favorites_filtered.is_empty() {
            let flex_row_history = self.emojis_flex(self.favorites_filtered.iter().copied());
            emojis_section = emojis_section.push(flex_row_history);
            emojis_section = emojis_section.push(widget::divider::horizontal::default());
        }

        let flex_row = self.emojis_flex(self.emojis_filtered.iter().copied());
        emojis_section = emojis_section.push(flex_row);

        let emojis_section_container = emojis_section
            .apply(widget_copy::Scrollable::new)
            .id(self.scrollable_id.clone())
            .height(Length::Fill)
            .width(Length::Fill)
            .on_scroll(|in_0| Message::ScrollViewport(Box::new(in_0)))
            .apply(widget::container)
            .width(Length::Fill)
            .height(500);
        emojis_section_container
    }

    fn preview(&self) -> widget::Container<'_, Message, Theme> {
        let preview_emoji_opt = self.emoji_selected();
        let mut preview_row = widget::row()
            .spacing(cosmic::theme::active().cosmic().space_xxs())
            .align_items(Alignment::Center);
        match preview_emoji_opt {
            Some(preview_emoji) => {
                let emoji_txt = widget::text(preview_emoji.as_str())
                    .size(35)
                    .width(50)
                    .height(50)
                    .font(self.font_family)
                    .shaping(iced_core::text::Shaping::Advanced)
                    .wrap(iced::widget::text::Wrap::None)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center);
                preview_row = preview_row.push(emoji_txt);
                let mut name_column = widget::column::with_capacity(2);

                let emoji_name = self.emoji_name_trimmed(preview_emoji);

                name_column = name_column.push(widget::text::body(emoji_name));
                if let Some(shortcode) = preview_emoji.shortcode() {
                    name_column = name_column.push(widget::text::caption(shortcode))
                }
                preview_row = preview_row.push(name_column);
            }
            None => {
                let group_str = self
                    .selected_group
                    .map_or_else(|| fl!("emojis-and-favorites"), group_string);
                preview_row = preview_row.push(widget::text::title1(group_str));
            }
        }
        if show_color_buttons(self.selected_group) {
            let color_buttons = self.color_buttons();
            preview_row = preview_row.push(widget::horizontal_space(Length::Fill));
            preview_row = preview_row.push(color_buttons);
        }
        // use cosmic::prelude::ElementExt;
        let center_y = widget::container(preview_row)
            .height(50)
            .max_height(50)
            .center_y();
        return center_y;
    }

    fn emoji_name_trimmed(&self, preview_emoji: &'static emojis::Emoji) -> Cow<'_, str> {
        let mut emoji_name = self.emoji_name_localized(preview_emoji);

        let emoji_name_len = emoji_name.len();
        let cut_off_idx = emoji_name
            .char_indices()
            .nth(40)
            .map_or(emoji_name_len, |(i, _)| i);
        emoji_name = emoji_name.get(..cut_off_idx).unwrap_or(emoji_name);
        let emoji_name = if emoji_name_len == emoji_name.len() {
            Cow::from(emoji_name)
        } else {
            Cow::from(emoji_name.to_owned() + "...")
        };
        emoji_name
    }

    fn nav_move_right(&mut self) -> Command<cosmic::app::Message<Message>> {
        let mut key = nav::key_from_group(self.selected_group);
        key = if key >= b'9' { b'0' } else { key + 1 };
        return self.update_group(nav::group_from_key(key));
    }

    fn nav_move_left(&mut self) -> Command<cosmic::app::Message<Message>> {
        let mut key = nav::key_from_group(self.selected_group);
        key = if key <= b'0' { b'9' } else { key - 1 };
        return self.update_group(nav::group_from_key(key));
    }
}
macro_rules! icon {
    ($name:expr) => {{
        concat!("dev.dominiccgeh.", $name)
    }};
}

fn show_color_buttons(group: Option<emojis::Group>) -> bool {
    use emojis::Group::*;
    return matches!(group, None | Some(PeopleAndBody));
}

// todo icon cache
fn group_icon(group: emojis::Group) -> &'static str {
    let icon = match group {
        emojis::Group::SmileysAndEmotion => icon!("emotion-satisfied"),
        emojis::Group::PeopleAndBody => icon!("people-nearby"),
        emojis::Group::AnimalsAndNature => icon!("pets"),
        emojis::Group::FoodAndDrink => icon!("food"),
        emojis::Group::TravelAndPlaces => icon!("world-1"),
        emojis::Group::Activities => icon!("walking"),
        emojis::Group::Objects => icon!("objects-column"),
        emojis::Group::Symbols => icon!("symbols"),
        emojis::Group::Flags => icon!("black-flag-icon"),
    };
    icon
}

mod nav;
fn group_string(group: emojis::Group) -> String {
    match group {
        emojis::Group::SmileysAndEmotion => fl!("smileys-and-emotion"),
        emojis::Group::PeopleAndBody => fl!("people-and-body"),
        emojis::Group::AnimalsAndNature => fl!("animals-and-nature"),
        emojis::Group::FoodAndDrink => fl!("food-and-drink"),
        emojis::Group::TravelAndPlaces => fl!("travel-and-places"),
        emojis::Group::Activities => fl!("activities"),
        emojis::Group::Objects => fl!("objects"),
        emojis::Group::Symbols => fl!("symbols"),
        emojis::Group::Flags => fl!("flags"),
    }
}
