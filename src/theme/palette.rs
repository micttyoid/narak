use bevy::prelude::*;

// Text colors
/// #ddd369 - Muted gold for labels
pub const LABEL_TEXT: Color = Color::srgb(0.867, 0.827, 0.412);
/// #fcfbcc - Bright cream for headers
pub const HEADER_TEXT: Color = Color::srgb(0.988, 0.984, 0.800);
/// #ececec - Light gray for button text
pub const BUTTON_TEXT: Color = Color::srgb(0.925, 0.925, 0.925);

// Button backgrounds
/// #8b3a2e - Deep reddish-brown
pub const BUTTON_BACKGROUND: Color = Color::srgb(0.545, 0.227, 0.180);
/// #c4543a - Warm rust/terra cotta (hovered state)
pub const BUTTON_HOVERED_BACKGROUND: Color = Color::srgb(0.769, 0.329, 0.227);
/// #5c2818 - Dark chocolate brown (pressed state)
pub const BUTTON_PRESSED_BACKGROUND: Color = Color::srgb(0.361, 0.157, 0.094);

// Additional accent colors
/// #e8b44a - Golden yellow
pub const BUTTON_BORDER: Color = Color::srgb(0.910, 0.706, 0.290);
/// #2a1f1a - Very dark brown
pub const BACKGROUND_DARK: Color = Color::srgb(0.165, 0.122, 0.102);
