import { MapDefinition } from "../..";

export default {
    pos_x: -2308,
    pos_y: 2078,
    scale: 4.082077,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 20000,
            altitudeMin: -50,
        },
        {
            name: "lower",
            altitudeMax: -50,
            altitudeMin: -5000,
        }
    ]
} satisfies MapDefinition;
