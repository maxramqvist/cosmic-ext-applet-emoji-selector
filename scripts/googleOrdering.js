const groups = require('./emoji_15_1_ordering.json');


export function* googleEmojisIter(includeCategories) {
    for (const group of groups) {
        if (includeCategories) {
            yield { kind: "group", group: group.group }
        }
        for (const emoji of group.emoji) {
            const { base, alternates } = emoji;
            if (alternates.length) {
                yield* Iterator.from(alternates).map(alternate => String.fromCodePoint(...alternate));
            } else {
                yield String.fromCodePoint(...base);
            }

        }
    }
}