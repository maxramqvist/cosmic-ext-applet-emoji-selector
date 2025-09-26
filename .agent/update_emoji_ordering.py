#!/usr/bin/env python3
"""
Script to update emoji ordering from Google Fonts emoji metadata.
Downloads the latest emoji metadata and converts it to Rust format.
"""

import json
import requests
from typing import List, Dict, Any

def unicode_to_emoji(code_points: List[int]) -> str:
    """Convert Unicode code points to emoji string."""
    return ''.join(chr(cp) for cp in code_points)

def process_emoji_data(emoji_data: List[Dict[str, Any]]) -> tuple:
    """Process emoji data and return ordered emoji list and group ranges."""
    all_emojis = []
    group_ranges = {}
    current_start = 0

    for group_data in emoji_data:
        group_name = group_data['group']
        group_emojis = []

        for emoji_entry in group_data['emoji']:
            # Convert base emoji
            base_emoji = unicode_to_emoji(emoji_entry['base'])
            group_emojis.append(base_emoji)

            # Add alternate versions (like skin tones)
            for alternate in emoji_entry.get('alternates', []):
                alt_emoji = unicode_to_emoji(alternate)
                group_emojis.append(alt_emoji)

        # Map group name to our enum names
        group_mapping = {
            'Smileys and emotions': 'SMILEYS_AND_EMOTIONS',
            'People': 'PEOPLE',
            'Animals and nature': 'ANIMALS_AND_NATURE',
            'Food and drink': 'FOOD_AND_DRINK',
            'Travel and places': 'TRAVEL_AND_PLACES',
            'Activities and events': 'ACTIVITIES_AND_EVENTS',
            'Objects': 'OBJECTS',
            'Symbols': 'SYMBOLS',
            'Flags': 'FLAGS'
        }

        if group_name in group_mapping:
            group_ranges[group_mapping[group_name]] = (current_start, len(group_emojis))
            current_start += len(group_emojis)
            all_emojis.extend(group_emojis)

    return all_emojis, group_ranges

def generate_rust_code(emojis: List[str], group_ranges: Dict[str, tuple]) -> str:
    """Generate Rust code for the emoji ordering."""

    # Generate emoji array
    emoji_array = 'pub const GOOGLE_ORDERING: &[&str] = &[\n'
    for emoji in emojis:
        emoji_array += f'    "{emoji}",\n'
    emoji_array += '];\n\n'

    # Generate group range constants
    range_constants = ''
    for group_name, (start, count) in group_ranges.items():
        range_constants += f'pub const {group_name}: (usize, usize) = ({start}, {count});\n'

    return emoji_array + range_constants

def main():
    """Main function to download and process emoji data."""
    url = "https://raw.githubusercontent.com/googlefonts/emoji-metadata/main/emoji_17_0_ordering.json"

    print("Downloading emoji metadata...")
    response = requests.get(url)
    response.raise_for_status()

    emoji_data = response.json()

    print("Processing emoji data...")
    emojis, group_ranges = process_emoji_data(emoji_data)

    print(f"Found {len(emojis)} emojis in {len(group_ranges)} groups")

    # Generate Rust code
    rust_code = generate_rust_code(emojis, group_ranges)

    # Write to file
    output_file = "/home/max/git/maxramqvist/cosmic-ext-applet-emoji-selector/src/google_ordering_new.rs"
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(rust_code)

    print(f"Generated {output_file}")
    print("\nGroup ranges:")
    for group, (start, count) in group_ranges.items():
        print(f"  {group}: {start} -> {start + count} ({count} emojis)")

if __name__ == "__main__":
    main()