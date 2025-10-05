import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { persistReducer } from "redux-persist";
import { kReduxPersistLocalStorage } from "./storage";

export type RadarSettingsState = {
    dialogOpen: boolean;

    displayStatistics: boolean,

    iconSize: number,
    displayBombDetails: boolean,
    bombDetailsOpacity: number,
    showAllLayers: boolean,

    mapStyle: string,
    mapScale: number,
    mapMarginLeft: number,
    mapMarginRight: number,
    mapMarginTop: number,
    mapMarginBottom: number,

    colorDotCT: string;
    colorDotT: string;
    colorDotOwn: string;
    showDotOwn: boolean;

    disablePadding: boolean;
    hideMapTitle: boolean;
};

export const kDefaultRadarSettings: RadarSettingsState = {
    dialogOpen: false,
    iconSize: 3.0,
    displayBombDetails: true,
    bombDetailsOpacity: 1.0,
    showAllLayers: true,

    displayStatistics: false,

    mapStyle: "Official",
    mapScale: 1,

    mapMarginTop: 0,
    mapMarginLeft: 0,
    mapMarginRight: 0,
    mapMarginBottom: 0,

    colorDotCT: "#0007ff",
    colorDotT: "#ffc933",
    colorDotOwn: "#e91e63",

    showDotOwn: true,
    disablePadding: false,
    hideMapTitle: false
};
const slice = createSlice({
    name: "radar-settings",
    initialState: () => kDefaultRadarSettings,
    reducers: {
        updateRadarSettings: (state, action: PayloadAction<Partial<RadarSettingsState>>) => {
            Object.assign(state, action.payload);
        },
    },
});

export default persistReducer(
    {
        key: "radar-settings",
        storage: kReduxPersistLocalStorage,
    },
    slice.reducer,
);

export const { updateRadarSettings } = slice.actions;
