use cosmic::{iced, widget, Theme};

pub(crate) fn color_button_apperance(
    color: [f32; 4],
    selected: Option<bool>,
    theme: &Theme,
) -> widget::button::Appearance {
    let is_selected = selected.is_some_and(|s| s);
    let color_button = widget::button::Appearance {
        background: Some(iced::Color::from(color).into()),
        border_radius: theme.cosmic().radius_s().into(),
        border_width: if is_selected { 2.0 } else { 0.0 },
        border_color: if is_selected {
            theme.cosmic().accent.border.into()
        } else {
            Default::default()
        },
        ..Default::default()
    };
    return color_button;
}
