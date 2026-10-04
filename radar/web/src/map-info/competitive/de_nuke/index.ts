import { MapDefinition } from "../..";

export default {
    pos_x: -3453, // upper left world coordinate
    pos_y: 2887,
    scale: 7,


    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -495,
        },
        {
            name: "lower",
            altitudeMax: -495,
            altitudeMin: -10000,
        }
    ]
} satisfies MapDefinition;
