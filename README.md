![Screenshot_2025-05-23_22-39-32](https://github.com/user-attachments/assets/dbc9f0fe-f99b-4eb0-bfcc-c869207033e0)

# Install 
```sh
git clone https://github.com/leb-kuchen/cosmic-ext-applet-emoji-selector 
cd cosmic-ext-applet-emoji-selector 
cargo b -r
sudo just install
```

# Config
The configuration directory is `~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/`.
In addition, the default schema can be installed with `just install-schema`. 
More documentation is provided [here](CONFIG.md).

# Usage
After installation a smiling emoji icon will appear in the applet tray. Click it, select an emoji. The emoji is now in the clipboard so you can paste it into wherever you want.

## Keyboard Shortcut (Recommended)
For quick access, you can set up a global keyboard shortcut to open the emoji selector:

1. Open COSMIC Settings
2. Navigate to Keyboard → Shortcuts → Custom Shortcuts
3. Add a new custom shortcut with:
   - **Name**: "Emoji Selector"
   - **Command**: `cosmic-ext-applet-emoji-selector --toggle`
   - **Shortcut**: `Super+Shift+E`

This allows you to quickly open the emoji selector from anywhere in the system using `Super+Shift+E`.

## Keyboard Navigation
When the emoji selector is open, you can use these keyboard shortcuts:
- `Escape` - Close the emoji selector
- `Arrow keys` - Navigate between emoji groups
- `Numbers (1-9)` - Quickly switch to different emoji groups
- `/` - Focus the search input
- `Enter` - Select the highlighted emoji
- `Home/End` - Jump to beginning/end of emoji list
- `Page Up/Down` - Scroll through emojis

# Emoji font
`Noto Color Emoji` is the default emoji font and is required by default. 
The default can be changed in `~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/font_family`.
A font which supports Unicode 15.1 is generally recommended.

# License
Files without an SPDX identifier are licensed under the MIT LICENSE
