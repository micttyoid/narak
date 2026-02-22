use bevy::prelude::*;

// Base: #476712 (olive / forest green)
// RGB(71, 103, 18) -> srgb(0.278, 0.404, 0.071)

// Text colors
/// White – normal / high-contrast text (e.g. credits)
pub const NORMAL_TEXT_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
/// #9ca87a - Muted sage for labels
pub const LABEL_TEXT: Color = Color::srgb(0.612, 0.659, 0.478);
/// #e0ebc8 - Light cream-green for headers
pub const HEADER_TEXT: Color = Color::srgb(0.878, 0.922, 0.784);
/// #e8f0dc - Soft off-white for button text
pub const BUTTON_TEXT: Color = Color::srgb(0.910, 0.941, 0.863);

// Button backgrounds (based on #476712)
/// #476712 - Base olive green
pub const BUTTON_BACKGROUND: Color = Color::srgb(0.278, 0.404, 0.071);
/// #5c7e1a - Lighter olive (hovered state)
pub const BUTTON_HOVERED_BACKGROUND: Color = Color::srgb(0.361, 0.494, 0.102);
/// #354d0e - Darker olive (pressed state)
pub const BUTTON_PRESSED_BACKGROUND: Color = Color::srgb(0.208, 0.302, 0.055);

// Additional accent colors
/// #7a9b3a - Lighter green-gold border
pub const BUTTON_BORDER: Color = Color::srgb(0.478, 0.608, 0.227);
/// #1a2410 - Very dark green
pub const BACKGROUND_DARK: Color = Color::srgb(0.102, 0.141, 0.063);
/// Green-tinted UI overlay
pub const GAMEPLAY_UI_BACKGROUND: Color = Color::srgb(0.15, 0.35, 0.08);

// Credits menu
/// Semi-transparent dark backdrop behind credits text
pub const CREDITS_BACKDROP: Color = Color::srgba(0.08, 0.12, 0.05, 0.75);
