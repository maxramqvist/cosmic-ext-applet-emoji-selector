const LINE_REGEX = /^(?<codePoints>[0-9A-F\x20]+);\x20*(?<status>[a-z\-]+)\x20*#/u;

const SUBGROUP_REGEX = /^# group: (?<group>.+)/

const fs = require('fs');
const emojiTestText = fs.readFileSync('./emoji_15_1-test.txt', 'utf-8');

// gendered family emoji have lost their gender since unicode 15.1, 
// noto color emoji displays non-gendered fallback emoji
const FAMILY_GENDER = new Set([
    "👨‍👩‍👦",
    "👨‍👩‍👧",
    "👨‍👩‍👧‍👦",
    "👨‍👩‍👦‍👦",
    "👨‍👩‍👧‍👧",
    "👨‍👨‍👦",
    "👨‍👨‍👧",
    "👨‍👨‍👧‍👦",
    "👨‍👨‍👦‍👦",
    "👨‍👨‍👧‍👧",
    "👩‍👩‍👦",
    "👩‍👩‍👧",
    "👩‍👩‍👧‍👦",
    "👩‍👩‍👦‍👦",
    "👩‍👩‍👧‍👧",
    "👨‍👦",
    "👨‍👦‍👦",
    "👨‍👧",
    "👨‍👧‍👦",
    "👨‍👧‍👧",
    "👩‍👦",
    "👩‍👦‍👦",
    "👩‍👧",
    "👩‍👧‍👦",
    "👩‍👧‍👧",
    "👪",
]);

export function* unicodeEmojisIter(includeCategories) {
    for (const line of emojiTestText.split("\n")) {
        const groupMatch = line.match(SUBGROUP_REGEX);
        if (groupMatch && includeCategories) {
            const group = groupMatch.groups.group;
            if (group === "Component") { continue }
            yield { kind: "group", group: group.replace("&", "and") }
            continue
        }
        const emojiMatch = line.match(LINE_REGEX);
        if (!emojiMatch || emojiMatch.groups.status !== 'fully-qualified') continue;
        const emoji = String.fromCodePoint(...emojiMatch.groups.codePoints.trim().split(' ').map(codePoint => parseInt(codePoint, 16)));
        if (FAMILY_GENDER.has(emoji)) { continue }
        yield emoji
    }
}