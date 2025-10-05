import { SettingsBackupRestore as IconReset } from "@mui/icons-material";
import {
    Box,
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    IconButton,
    Slider,
    Switch,
    Typography,
    MenuItem,
    FormControl,
    Select,
    Tabs,
    Tab
} from "@mui/material";
import { MuiColorInput } from "mui-color-input";
import React, { useState } from "react";
import { useAppDispatch, useAppSelector } from "../../../../state";
import { kDefaultRadarSettings, RadarSettingsState, updateRadarSettings } from "../../../../state/radar-settings";

export default React.memo(() => {
    const isOpen = useAppSelector((state) => state.radarSettings.dialogOpen);
    const dispatch = useAppDispatch();
    const highlightBroadcaster = useAppSelector((state) => state.radarSettings.showDotOwn);
    const [currentTab, setCurrentTab] = useState(0);

    return (
        <Dialog open={isOpen} onClose={() => dispatch(updateRadarSettings({ dialogOpen: false }))}>
            <DialogTitle>Settings</DialogTitle>
            <DialogContent
                sx={{
                    minWidth: "15em",
                    width: "30em",

                    minHeight: "10em",
                    height: "30em",

                    overflow: "auto",
                }}
            >
                <Box sx={{ borderBottom: 1, borderColor: 'divider', width: "100%" }}>
                    <Tabs value={currentTab} onChange={(_event, value) => setCurrentTab(value)}>
                        <Tab label="Page Settings" />
                        <Tab label="Visuals" />
                        <Tab label="Map Style" />
                        <Tab label="Colors" />
                    </Tabs>
                </Box>

                <TabPanel index={0} value={currentTab}>
                    <SettingBoolean target="disablePadding" title="Disable page padding" />
                    <SettingBoolean target="hideMapTitle" title="Hide map title" />
                    <SettingBoolean target="displayStatistics" title="Display Statistics" />
                </TabPanel>

                <TabPanel index={1} value={currentTab}>
                    <SettingSlider target={"iconSize"} title={"Icon Size"} min={0.1} max={5.0} step={0.1} />
                    <SettingSlider target={"bombDetailsOpacity"} title={"Bomb Details Opacity"} min={0.1} max={1.0} step={0.1} />

                    <SettingBoolean target="displayBombDetails" title="Display Bomb Details" />
                    <SettingBoolean target="showAllLayers" title="Display all levels" />
                    <SettingBoolean target="showDotOwn" title="Highlight broadacster" />
                </TabPanel>

                <TabPanel index={2} value={currentTab}>
                    <SettingStyleSelector />
                    <SettingSlider target={"mapScale"} title={"Scale"} min={.1} max={4} step={.1} />

                    <SettingSlider target={"mapMarginLeft"} title={"Margin Left (%)"} min={-100} max={100} step={1} />
                    <SettingSlider target={"mapMarginRight"} title={"Margin Right (%)"} min={-100} max={100} step={1} />
                    <SettingSlider target={"mapMarginTop"} title={"Margin Top (%)"} min={-100} max={100} step={1} />
                    <SettingSlider target={"mapMarginBottom"} title={"Margin Bottom (%)"} min={-100} max={100} step={1} />
                </TabPanel>

                <TabPanel index={3} value={currentTab}>
                    <SettingDotColor target="colorDotCT" title="CT Color" />
                    <SettingDotColor target="colorDotT" title="T Color" />
                    {highlightBroadcaster && <SettingDotColor target="colorDotOwn" title="Own Color" />}
                </TabPanel>

            </DialogContent>
            <DialogActions>
                <Button onClick={() => dispatch(updateRadarSettings({ dialogOpen: false }))}>Close</Button>
            </DialogActions>
        </Dialog>
    );
});

const TabPanel = (props: {
    children?: React.ReactNode;
    index: number;
    value: number;
}) => {
    const { children, value, index } = props;

    return (
        <Box role="tabpanel" sx={{ p: 1, display: value == index ? "flex" : "none", flexDirection: "column", gap: 1 }}>
            {value === index && children}
        </ Box>
    );
}

const SettingSlider = React.memo((props: {
    title: string,
    target: KeysMatching<RadarSettingsState, number>,

    min: number,
    max: number,
    step: number
}) => {
    const value = useAppSelector((state) => state.radarSettings[props.target]);
    const dispatch = useAppDispatch();

    return (
        <Box>
            <Typography variant={"subtitle1"}>{props.title}</Typography>
            <Slider
                min={props.min}
                max={props.max}
                step={props.step}
                value={value}
                onChange={(_event, value) => dispatch(updateRadarSettings({ [props.target]: value }))}
                valueLabelDisplay={"auto"}
            />
        </Box>
    );
});

type KeysMatching<T extends object, V> = {
    [K in keyof T]-?: T[K] extends V ? K : never
}[keyof T];

const SettingBoolean = React.memo((props: {
    title: string,
    target: KeysMatching<RadarSettingsState, boolean>
}) => {
    const { target, title } = props;
    const value = useAppSelector(state => state.radarSettings[target]);
    const dispatch = useAppDispatch();

    return (
        <Box>
            <Typography variant={"subtitle1"}>{title}</Typography>
            <Switch
                checked={value}
                onChange={(_event, value) => dispatch(updateRadarSettings({ [target]: value }))}
            />
        </Box>
    );
},
);

const SettingDotColor = React.memo(
    (props: { title: string; target: keyof RadarSettingsState & ("colorDotCT" | "colorDotT" | "colorDotOwn") }) => {
        const value = useAppSelector((state) => state.radarSettings[props.target]);
        const dispatch = useAppDispatch();

        return (
            <Box>
                <Typography variant={"subtitle1"}>{props.title}</Typography>
                <Box
                    sx={{
                        display: "flex",
                        flexDirection: "row",
                        gap: "1em",
                    }}
                >
                    <MuiColorInput
                        fullWidth
                        sx={{ minWidth: "5em" }}
                        size="small"
                        format="hex"
                        value={value}
                        onChange={(event) =>
                            dispatch(
                                updateRadarSettings({
                                    [props.target]: event,
                                }),
                            )
                        }
                    />
                    <IconButton
                        onClick={() => {
                            dispatch(
                                updateRadarSettings({
                                    [props.target]: kDefaultRadarSettings[props.target],
                                }),
                            );
                        }}
                        title="Reset value"
                    >
                        <IconReset />
                    </IconButton>
                </Box>
            </Box>
        );
    },
);

const SettingStyleSelector = React.memo(() => {
    const value = useAppSelector((state) => state.radarSettings.mapStyle);
    const dispatch = useAppDispatch();

    return (
        <Box>
            <Typography variant={"subtitle1"}>Map Style</Typography>
            <FormControl fullWidth>
                <Select
                    value={value}
                    onChange={event => dispatch(updateRadarSettings({ mapStyle: event.target.value }))}
                >
                    <MenuItem value="Official">Official</MenuItem>
                    <MenuItem value="SimpleRadar">Simple Radar</MenuItem>
                </Select>
            </FormControl>
        </Box>
    );
});
