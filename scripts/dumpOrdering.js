const { unicodeEmojisIter } = require('./unicodeOrdering.js');
const { googleEmojisIter } = require('./googleOrdering.js');
const fs = require("fs")
const { stdout } = process;

const WRITE_STDOUT = true;


fs.writeFileSync("./unicodeOrdering.txt", arrayToString(unicodeEmojisIter(true)))
fs.writeFileSync("./googleOrdering.txt", arrayToString(googleEmojisIter(true)))



// const googleEmojis = googleEmojisIter().toArray();
// const unicodeEmojis = unicodeEmojisIter().toArray();

// let diff1 = new Set(unicodeEmojis).difference(new Set(googleEmojis))
// let diff2 = new Set(googleEmojis).difference(new Set(unicodeEmojis))
// console.log(diff1, diff2)


if (WRITE_STDOUT) {
    const googleEmojis = googleEmojisIter().toArray();
    stdout.write(`pub const GOOGLE_ORDERING: &[&str] = &[${googleEmojis.map(emoji => `"${emoji}"`).join(", ")} ];\n`);
    writeGroupIndices()
}


function arrayToString(iter) {
    let format = ""
    let count = 0;
    for (const emoji of iter) {
        if (typeof emoji === "object") {
            const group = emoji.group;
            if (!format.endsWith("\n") && format) {
                format += "\n"
            }
            format += `--- ${group} ---\n`
            count = 0
            continue
        }
        format += emoji
        count += 1;
        if (count % 9 == 0) {
            format += "\n"
        } else {
            format += " "
        }
    }
    return format
}


function writeGroupIndices() {
    const googleGroupArray = [];
    let count = 0;
    for (const emoji of googleEmojisIter(true)) {
        if (typeof emoji === "object") {
            googleGroupArray.push({ index: count, group: emoji.group });
            continue
        }
        count += 1;
    }
    let lastCount = count;

    for (const [count, item] of googleGroupArray.entries()) {
        const startIndex = item.index;
        const length = (googleGroupArray[count + 1]?.index ?? lastCount) - startIndex;
        stdout.write(`pub const ${item.group.toUpperCase().replaceAll(" ", "_")}: (usize, usize) = (${startIndex}, ${length});\n`);
    }
}


