/*
 * Minimal parser for Valve KeyValues, as used by the CS2 overview files in
 * resource/overviews/*.txt.
 *
 * It supports the subset those files use: quoted keys and values, nested
 * blocks, line comments and block comments. Keys are lower-cased so lookups
 * don't have to worry about AltitudeMax vs altitudemax.
 */

export type KeyValuesNode = {
    [key: string]: string | KeyValuesNode;
};

const isWhitespace = (char: string): boolean => char === " " || char === "\t" || char === "\r" || char === "\n";

const skipTrivia = (text: string, index: number): number => {
    while (index < text.length) {
        const char = text[index];

        if (isWhitespace(char)) {
            index++;
            continue;
        }

        if (char === "/" && text[index + 1] === "/") {
            const end = text.indexOf("\n", index);
            index = end === -1 ? text.length : end + 1;
            continue;
        }

        if (char === "/" && text[index + 1] === "*") {
            const end = text.indexOf("*/", index + 2);
            index = end === -1 ? text.length : end + 2;
            continue;
        }

        break;
    }

    return index;
};

const readToken = (text: string, index: number): { token: string, index: number } => {
    index = skipTrivia(text, index);

    if (text[index] === "\"") {
        index++;
        let token = "";
        while (index < text.length && text[index] !== "\"") {
            token += text[index];
            index++;
        }
        return { token, index: index + 1 };
    }

    let token = "";
    while (index < text.length && !isWhitespace(text[index]) && text[index] !== "{" && text[index] !== "}" && text[index] !== "\"") {
        token += text[index];
        index++;
    }
    return { token, index };
};

const parseBlock = (text: string, index: number): { node: KeyValuesNode, index: number } => {
    const node: KeyValuesNode = {};
    index = skipTrivia(text, index);

    if (text[index] === "{") {
        index++;
    }

    while (index < text.length) {
        index = skipTrivia(text, index);

        if (index >= text.length || text[index] === "}") {
            return { node, index: index + 1 };
        }

        const key = readToken(text, index);
        index = skipTrivia(text, key.index);

        if (text[index] === "{") {
            const child = parseBlock(text, index);
            node[key.token.toLowerCase()] = child.node;
            index = child.index;
            continue;
        }

        const value = readToken(text, index);
        node[key.token.toLowerCase()] = value.token;
        index = value.index;
    }

    return { node, index };
};

export const parseKeyValues = (text: string): KeyValuesNode => {
    const { node } = parseBlock(text, 0);
    return node;
};
