import { MapDefinition } from "../..";

export default {
    pos_x: -3168, // upper left world coordinate
    pos_y: 1762,
    scale: 4,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 20000,
            altitudeMin: 11700,
        },
        {
            name: "lower",
            altitudeMax: 11700,
            altitudeMin: -10000,
        }
    ]
} satisfies MapDefinition;
