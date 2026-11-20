//@ pragma AppId snowhush-settings
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Quickshell.Wayland

ShellRoot {
    id: root

    // colors synchronized with serpantinum qs_colors.json
    property color colBase: "#050505"
    property color colMantle: "#0a0a0a"
    property color colCrust: "#000000"
    property color colSurface0: "#141414"
    property color colSurface1: "#222222"
    property color colSurface2: "#303030"
    property color colOverlay0: "#555555"
    property color colOverlay1: "#777777"
    property color colText: "#ffffff"
    property color colSubtext: "#9e9e9e"
    property color colAccent: "#ffffff"
    property color colBlue: "#89b4fa"
    property color colGreen: "#a6e3a1"
    property color colRed: "#f38ba8"
    property color colPeach: "#fab387"
    property color colMauve: "#cba6f7"
    property color colTeal: "#94e2d5"

    // font selection with system fallback
    property string themeFont: "Futura"
    property string effectiveFont: "sans-serif"

    // settings state
    property string optSnowfall: "steady"
    property real optDensity: 1.0
    property real optSpeedFactor: 1.1
    property real optSwayFactor: 2.3
    property real optTopMargin: 44.0
    property bool optTrackLayers: true
    property bool optSweepGround: true
    property real optMeltRate: 4.0
    property bool optWindAccel: true
    property real optWindFactor: 1.0
    property real optWindErosionThreshold: 110.0
    property real optWindErosionRate: 1.0
    property real optStaticWind: 0.0
    property real optWindowLeak: 0.5
    property real optMotionShed: 1.0
    property real optCursorForce: 1.5
    property real optCloudDensity: 1.0
    property real optCloudAmount: 1.0
    property real optCloudSize: 1.0
    property real optSprayDensity: 1.0
    property real optSprayAmount: 1.0
    property real optSpraySize: 1.0
    property real optSprayForce: 1.0
    property real optEjectionForce: 1.0
    property real optWindowWake: 1.0
    property real optWindowPlow: 1.0
    property real optMotionBlur: 1.0
    property bool optNoFullscreenSnow: true
    property real optGrainSpeed: 1.0
    property real optGrainGravity: 1.0
    property real optGrainRandom: 1.0
    property real optCloudRandom: 1.0
    property real optCloudAccel: 1.0
    property real optCloudFadeRate: 1.0
    property real optCloudDepthThreshold: 1.0
    property real optCloudDepthScale: 1.0
    property int optMaxSnow: 2500
    property int optMaxEffects: 5000
    property real optWorkspaceSweep: 1.0
    property real optShedThreshold: 1500.0
    property bool optDynamicWeather: false
    property string optWeatherMode: "auto"
    property real optWeatherSpeed: 1.4

    // ui state
    property int activeTab: 0
    property string searchQuery: ""
    property bool isSearching: searchQuery.trim().length > 0
    property bool isSnowhushRunning: false
    property bool confirmReset: false
    property bool savedFlash: false

    readonly property var tabMeta: [
        { "id": 0, "label": "Snowfall" },
        { "id": 1, "label": "Wind & Weather" },
        { "id": 2, "label": "Windows" },
        { "id": 3, "label": "Cursor & Melt" },
        { "id": 4, "label": "Effects" }
    ]

    function matchesSearch(title, description) {
        let q = root.searchQuery.trim().toLowerCase();
        if (q.length === 0)
            return true;
        return title.toLowerCase().indexOf(q) !== -1 || description.toLowerCase().indexOf(q) !== -1;
    }

    function updateFont() {
        let families = Qt.fontFamilies();
        let target = root.themeFont ? root.themeFont.trim() : "";
        if (target !== "") {
            for (let i = 0; i < families.length; i++) {
                if (families[i].toLowerCase() === target.toLowerCase()) {
                    root.effectiveFont = families[i];
                    return;
                }
            }
        }
        let defaults = ["Inter", "Noto Sans", "Cantarell", "DejaVu Sans", "Roboto", "sans-serif"];
        for (let j = 0; j < defaults.length; j++) {
            for (let i = 0; i < families.length; i++) {
                if (families[i].toLowerCase() === defaults[j].toLowerCase()) {
                    root.effectiveFont = families[i];
                    return;
                }
            }
        }
        root.effectiveFont = "sans-serif";
    }

    function requestSave() {
        saveDebounce.restart();
    }

    function applyPreset(name) {
        if (name === "flurries") {
            root.optDensity = 0.45;
            root.optSpeedFactor = 0.8;
            root.optSwayFactor = 1.6;
            root.optStaticWind = 0.0;
            root.optWindFactor = 0.8;
            root.optDynamicWeather = false;
        } else if (name === "steady") {
            root.optDensity = 1.0;
            root.optSpeedFactor = 1.1;
            root.optSwayFactor = 2.3;
            root.optStaticWind = 0.0;
            root.optWindFactor = 1.0;
            root.optDynamicWeather = false;
        } else if (name === "blizzard") {
            root.optDensity = 2.4;
            root.optSpeedFactor = 1.9;
            root.optSwayFactor = 3.6;
            root.optStaticWind = 140.0;
            root.optWindFactor = 2.0;
            root.optDynamicWeather = true;
            root.optWeatherMode = "blizzard";
        } else if (name === "zerog") {
            root.optDensity = 0.9;
            root.optSpeedFactor = 0.35;
            root.optSwayFactor = 3.8;
            root.optStaticWind = 0.0;
            root.optWindFactor = 1.2;
            root.optDynamicWeather = false;
        }
        root.writeConfig();
    }

    function writeConfig() {
        let snowfallPreset = "steady";
        if (root.optDensity <= 0.01)
            snowfallPreset = "off";
        else if (root.optDensity < 0.6)
            snowfallPreset = "light";
        else if (root.optDensity < 1.6)
            snowfallPreset = "steady";
        else
            snowfallPreset = "heavy";

        let obj = {
            "snowfall": snowfallPreset,
            "density": Number(root.optDensity.toFixed(2)),
            "speed_factor": Number(root.optSpeedFactor.toFixed(2)),
            "sway_factor": Number(root.optSwayFactor.toFixed(2)),
            "top_margin": Number(root.optTopMargin.toFixed(1)),
            "track_layers": root.optTrackLayers,
            "sweep_ground": root.optSweepGround,
            "melt": root.optMeltRate > 0.05,
            "melt_rate": Number(root.optMeltRate.toFixed(2)),
            "wind_accel": root.optWindAccel,
            "wind_factor": Number(root.optWindFactor.toFixed(2)),
            "wind_erosion_threshold": Number(root.optWindErosionThreshold.toFixed(1)),
            "wind_erosion_rate": Number(root.optWindErosionRate.toFixed(2)),
            "static_wind": Number(root.optStaticWind.toFixed(1)),
            "window_leak": Number(root.optWindowLeak.toFixed(2)),
            "motion_shed": Number(root.optMotionShed.toFixed(2)),
            "cursor_force": Number(root.optCursorForce.toFixed(2)),
            "cloud_density": Number(root.optCloudDensity.toFixed(2)),
            "cloud_amount": Number(root.optCloudAmount.toFixed(2)),
            "cloud_size": Number(root.optCloudSize.toFixed(2)),
            "spray_density": Number(root.optSprayDensity.toFixed(2)),
            "spray_amount": Number(root.optSprayAmount.toFixed(2)),
            "spray_size": Number(root.optSpraySize.toFixed(2)),
            "spray_force": Number(root.optSprayForce.toFixed(2)),
            "ejection_force": Number(root.optEjectionForce.toFixed(2)),
            "window_wake": Number(root.optWindowWake.toFixed(2)),
            "window_plow": Number(root.optWindowPlow.toFixed(2)),
            "motion_blur": Number(root.optMotionBlur.toFixed(2)),
            "no_fullscreen_snow": root.optNoFullscreenSnow,
            "grain_speed": Number(root.optGrainSpeed.toFixed(2)),
            "grain_gravity": Number(root.optGrainGravity.toFixed(2)),
            "grain_random": Number(root.optGrainRandom.toFixed(2)),
            "cloud_random": Number(root.optCloudRandom.toFixed(2)),
            "cloud_accel": Number(root.optCloudAccel.toFixed(2)),
            "cloud_fade_rate": Number(root.optCloudFadeRate.toFixed(2)),
            "cloud_depth_threshold": Number(root.optCloudDepthThreshold.toFixed(2)),
            "cloud_depth_scale": Number(root.optCloudDepthScale.toFixed(2)),
            "max_snow": root.optMaxSnow,
            "max_effects": root.optMaxEffects,
            "workspace_sweep": Number(root.optWorkspaceSweep.toFixed(2)),
            "shed_threshold": Number(root.optShedThreshold.toFixed(1)),
            "dynamic_weather": root.optDynamicWeather,
            "weather_mode": root.optWeatherMode,
            "weather_cycle_speed": Number(root.optWeatherSpeed.toFixed(2))
        };
        configWriter.payload = JSON.stringify(obj, null, 2);
        configWriter.running = true;
        root.savedFlash = true;
        savedFlashTimer.restart();
    }

    function resetToDefaults() {
        root.optSnowfall = "steady";
        root.optDensity = 1.0;
        root.optSpeedFactor = 1.1;
        root.optSwayFactor = 2.3;
        root.optTopMargin = 44.0;
        root.optTrackLayers = true;
        root.optSweepGround = true;
        root.optMeltRate = 4.0;
        root.optWindAccel = true;
        root.optWindFactor = 1.0;
        root.optWindErosionThreshold = 110.0;
        root.optWindErosionRate = 1.0;
        root.optStaticWind = 0.0;
        root.optWindowLeak = 0.5;
        root.optMotionShed = 1.0;
        root.optCursorForce = 1.5;
        root.optCloudDensity = 1.0;
        root.optCloudAmount = 1.0;
        root.optCloudSize = 1.0;
        root.optSprayDensity = 1.0;
        root.optSprayAmount = 1.0;
        root.optSpraySize = 1.0;
        root.optSprayForce = 1.0;
        root.optEjectionForce = 1.0;
        root.optWindowWake = 1.0;
        root.optWindowPlow = 1.0;
        root.optMotionBlur = 1.0;
        root.optNoFullscreenSnow = true;
        root.optGrainSpeed = 1.0;
        root.optGrainGravity = 1.0;
        root.optGrainRandom = 1.0;
        root.optCloudRandom = 1.0;
        root.optCloudAccel = 1.0;
        root.optCloudFadeRate = 1.0;
        root.optCloudDepthThreshold = 1.0;
        root.optCloudDepthScale = 1.0;
        root.optMaxSnow = 2500;
        root.optMaxEffects = 5000;
        root.optWorkspaceSweep = 1.0;
        root.optShedThreshold = 1500.0;
        root.optDynamicWeather = false;
        root.optWeatherMode = "auto";
        root.optWeatherSpeed = 1.4;
        root.confirmReset = false;
        root.writeConfig();
    }

    Component.onCompleted: {
        root.updateFont();
    }

    FileView {
        id: themeSettingsWatcher

        path: Quickshell.env("HOME") + "/.config/serpantinum/settings.json"
        watchChanges: true
        onFileChanged: reload()
        onLoaded: {
            let txt = text().trim();
            if (txt !== "") {
                try {
                    let s = JSON.parse(txt);
                    if (s.theme && s.theme.fontFamily)
                        root.themeFont = s.theme.fontFamily;
                } catch (e) {
                }
            }
            root.updateFont();
        }
    }

    FileView {
        id: colorsWatcher

        path: Quickshell.env("HOME") + "/.local/state/serpantinum/qs_colors.json"
        watchChanges: true
        onFileChanged: reload()
        onLoaded: {
            let txt = text().trim();
            if (txt !== "") {
                try {
                    let c = JSON.parse(txt);
                    if (c.base) root.colBase = c.base;
                    if (c.mantle) root.colMantle = c.mantle;
                    if (c.crust) root.colCrust = c.crust;
                    if (c.surface0) root.colSurface0 = c.surface0;
                    if (c.surface1) root.colSurface1 = c.surface1;
                    if (c.surface2) root.colSurface2 = c.surface2;
                    if (c.overlay0) root.colOverlay0 = c.overlay0;
                    if (c.overlay1) root.colOverlay1 = c.overlay1;
                    if (c.text) root.colText = c.text;
                    if (c.subtext0) root.colSubtext = c.subtext0;
                    if (c.blue) root.colBlue = c.blue;
                    if (c.accent) root.colAccent = c.accent;
                    else if (c.blue) root.colAccent = c.blue;
                    if (c.green) root.colGreen = c.green;
                    if (c.red) root.colRed = c.red;
                    if (c.peach) root.colPeach = c.peach;
                    if (c.mauve) root.colMauve = c.mauve;
                    if (c.teal) root.colTeal = c.teal;
                } catch (e) {
                }
            }
        }
    }

    FileView {
        id: configWatcher

        path: Quickshell.env("HOME") + "/.config/snowhush/config.json"
        watchChanges: true
        onFileChanged: reload()
        onLoaded: {
            let txt = text().trim();
            if (txt !== "") {
                try {
                    let cfg = JSON.parse(txt);
                    if (cfg.snowfall !== undefined)
                        root.optSnowfall = cfg.snowfall;
                    if (cfg.density !== undefined) {
                        root.optDensity = Number(cfg.density);
                    } else {
                        if (cfg.snowfall === "off") root.optDensity = 0;
                        else if (cfg.snowfall === "light") root.optDensity = 0.4;
                        else if (cfg.snowfall === "steady") root.optDensity = 1;
                        else if (cfg.snowfall === "heavy") root.optDensity = 2;
                    }
                    if (cfg.speed_factor !== undefined) root.optSpeedFactor = Number(cfg.speed_factor);
                    if (cfg.sway_factor !== undefined) root.optSwayFactor = Number(cfg.sway_factor);
                    if (cfg.top_margin !== undefined) root.optTopMargin = Number(cfg.top_margin);
                    if (cfg.track_layers !== undefined) root.optTrackLayers = Boolean(cfg.track_layers);
                    if (cfg.sweep_ground !== undefined) root.optSweepGround = Boolean(cfg.sweep_ground);
                    if (cfg.melt === false)
                        root.optMeltRate = 0.0;
                    else if (cfg.melt_rate !== undefined)
                        root.optMeltRate = Number(cfg.melt_rate);
                    if (cfg.wind_accel !== undefined) root.optWindAccel = Boolean(cfg.wind_accel);
                    if (cfg.wind_factor !== undefined) root.optWindFactor = Number(cfg.wind_factor);
                    if (cfg.wind_erosion_threshold !== undefined) root.optWindErosionThreshold = Number(cfg.wind_erosion_threshold);
                    if (cfg.wind_erosion_rate !== undefined) root.optWindErosionRate = Number(cfg.wind_erosion_rate);
                    if (cfg.static_wind !== undefined) root.optStaticWind = Number(cfg.static_wind);
                    if (cfg.window_leak !== undefined) root.optWindowLeak = Number(cfg.window_leak);
                    if (cfg.motion_shed !== undefined) root.optMotionShed = Number(cfg.motion_shed);
                    if (cfg.cursor_force !== undefined) root.optCursorForce = Number(cfg.cursor_force);
                    if (cfg.cloud_density !== undefined) root.optCloudDensity = Number(cfg.cloud_density);
                    if (cfg.cloud_amount !== undefined) root.optCloudAmount = Number(cfg.cloud_amount);
                    if (cfg.cloud_size !== undefined) root.optCloudSize = Number(cfg.cloud_size);
                    if (cfg.spray_density !== undefined) root.optSprayDensity = Number(cfg.spray_density);
                    if (cfg.spray_amount !== undefined) root.optSprayAmount = Number(cfg.spray_amount);
                    if (cfg.spray_size !== undefined) root.optSpraySize = Number(cfg.spray_size);
                    if (cfg.spray_force !== undefined) root.optSprayForce = Number(cfg.spray_force);
                    if (cfg.ejection_force !== undefined) root.optEjectionForce = Number(cfg.ejection_force);
                    if (cfg.window_wake !== undefined) root.optWindowWake = Number(cfg.window_wake);
                    if (cfg.window_plow !== undefined) root.optWindowPlow = Number(cfg.window_plow);
                    if (cfg.motion_blur !== undefined) root.optMotionBlur = Number(cfg.motion_blur);
                    if (cfg.no_fullscreen_snow !== undefined) root.optNoFullscreenSnow = Boolean(cfg.no_fullscreen_snow);
                    if (cfg.grain_speed !== undefined) root.optGrainSpeed = Number(cfg.grain_speed);
                    if (cfg.grain_gravity !== undefined) root.optGrainGravity = Number(cfg.grain_gravity);
                    if (cfg.grain_random !== undefined) root.optGrainRandom = Number(cfg.grain_random);
                    if (cfg.cloud_random !== undefined) root.optCloudRandom = Number(cfg.cloud_random);
                    if (cfg.cloud_accel !== undefined) root.optCloudAccel = Number(cfg.cloud_accel);
                    if (cfg.cloud_fade_rate !== undefined) root.optCloudFadeRate = Number(cfg.cloud_fade_rate);
                    if (cfg.cloud_depth_threshold !== undefined) root.optCloudDepthThreshold = Number(cfg.cloud_depth_threshold);
                    if (cfg.cloud_depth_scale !== undefined) root.optCloudDepthScale = Number(cfg.cloud_depth_scale);
                    if (cfg.max_snow !== undefined) root.optMaxSnow = Math.round(Number(cfg.max_snow));
                    if (cfg.max_effects !== undefined) root.optMaxEffects = Math.round(Number(cfg.max_effects));
                    if (cfg.workspace_sweep !== undefined) root.optWorkspaceSweep = Number(cfg.workspace_sweep);
                    if (cfg.shed_threshold !== undefined) root.optShedThreshold = Number(cfg.shed_threshold);
                    if (cfg.dynamic_weather !== undefined) root.optDynamicWeather = Boolean(cfg.dynamic_weather);
                    if (cfg.weather_mode !== undefined) root.optWeatherMode = String(cfg.weather_mode);
                    if (cfg.weather_cycle_speed !== undefined) root.optWeatherSpeed = Number(cfg.weather_cycle_speed);
                } catch (e) {
                }
            }
        }
    }

    Process {
        id: statusChecker

        command: ["pidof", "snowhush-wayland"]
        running: false

        stdout: StdioCollector {
            onStreamFinished: {
                root.isSnowhushRunning = this.text.trim().length > 0;
            }
        }
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            if (!statusChecker.running)
                statusChecker.running = true;
        }
    }

    Timer {
        id: saveDebounce
        interval: 120
        repeat: false
        onTriggered: root.writeConfig()
    }

    Timer {
        id: savedFlashTimer
        interval: 1400
        repeat: false
        onTriggered: root.savedFlash = false
    }

    Timer {
        id: resetConfirmTimer
        interval: 2600
        repeat: false
        onTriggered: root.confirmReset = false
    }

    Process {
        id: configWriter
        property string payload: ""
        command: ["sh", "-c", "mkdir -p \"$HOME/.config/snowhush\" && cat << 'EOF' > \"$HOME/.config/snowhush/config.json\"\n" + payload + "\nEOF"]
    }

    Process {
        id: starter
        command: [Quickshell.env("HOME") + "/.local/bin/snowhush-toggle", "start"]
        onExited: statusChecker.running = true
    }

    Process {
        id: killer
        command: [Quickshell.env("HOME") + "/.local/bin/snowhush-toggle", "stop"]
        onExited: statusChecker.running = true
    }

    Process {
        id: restarter
        command: [Quickshell.env("HOME") + "/.local/bin/snowhush-toggle", "restart"]
        onExited: statusChecker.running = true
    }

    FloatingWindow {
        id: win

        title: "Snowhush Settings"
        visible: true
        implicitWidth: 780
        implicitHeight: 620
        color: "transparent"

        Shortcut {
            sequence: "Escape"
            onActivated: {
                if (root.isSearching) {
                    searchInput.text = "";
                    searchInput.focus = false;
                } else {
                    Qt.quit();
                }
            }
        }

        Shortcut {
            sequence: "Ctrl+F"
            onActivated: {
                searchInput.forceActiveFocus();
                searchInput.selectAll();
            }
        }

        Shortcut {
            sequence: "Ctrl+1"
            onActivated: { root.activeTab = 0; scrollArea.contentY = 0; }
        }
        Shortcut {
            sequence: "Ctrl+2"
            onActivated: { root.activeTab = 1; scrollArea.contentY = 0; }
        }
        Shortcut {
            sequence: "Ctrl+3"
            onActivated: { root.activeTab = 2; scrollArea.contentY = 0; }
        }
        Shortcut {
            sequence: "Ctrl+4"
            onActivated: { root.activeTab = 3; scrollArea.contentY = 0; }
        }
        Shortcut {
            sequence: "Ctrl+5"
            onActivated: { root.activeTab = 4; scrollArea.contentY = 0; }
        }

        Rectangle {
            id: mainCard

            anchors.fill: parent
            radius: 16
            color: root.colBase
            border.color: root.colSurface1
            border.width: 1
            clip: true

            RowLayout {
                anchors.fill: parent
                spacing: 0

                // ==================== LEFT SIDEBAR ====================
                Rectangle {
                    id: sidebar
                    Layout.preferredWidth: 184
                    Layout.fillHeight: true
                    color: root.colMantle

                    Rectangle {
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        width: 1
                        color: root.colSurface1
                    }

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 14
                        spacing: 12

                        Text {
                            text: "Snowhush"
                            font.family: root.effectiveFont
                            font.pixelSize: 15
                            font.bold: true
                            color: root.colText
                            Layout.leftMargin: 6
                            Layout.topMargin: 2
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            implicitHeight: 1
                            color: root.colSurface1
                        }

                        // navigation rail
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            Repeater {
                                model: root.tabMeta

                                Rectangle {
                                    id: navItem
                                    required property var modelData
                                    readonly property bool isActive: !root.isSearching && root.activeTab === modelData.id

                                    Layout.fillWidth: true
                                    implicitHeight: 34
                                    radius: 8
                                    color: isActive ? root.colSurface1 : (navMouse.containsMouse ? root.colSurface0 : "transparent")
                                    border.color: isActive ? root.colSurface2 : "transparent"
                                    border.width: 1

                                    Rectangle {
                                        anchors.left: parent.left
                                        anchors.leftMargin: 4
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: 3
                                        height: navItem.isActive ? 14 : 0
                                        radius: 2
                                        color: root.colAccent

                                        Behavior on height {
                                            NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                                        }
                                    }

                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        anchors.left: parent.left
                                        anchors.leftMargin: 14
                                        anchors.right: parent.right
                                        anchors.rightMargin: 10
                                        text: navItem.modelData.label
                                        font.family: root.effectiveFont
                                        font.pixelSize: 12
                                        font.weight: navItem.isActive ? Font.DemiBold : Font.Normal
                                        color: navItem.isActive ? root.colText : root.colSubtext
                                        elide: Text.ElideRight
                                    }

                                    MouseArea {
                                        id: navMouse
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (root.isSearching)
                                                searchInput.text = "";
                                            root.activeTab = navItem.modelData.id;
                                            scrollArea.contentY = 0;
                                        }
                                    }

                                    Behavior on color {
                                        ColorAnimation { duration: 120 }
                                    }
                                }
                            }
                        }

                        Item {
                            Layout.fillHeight: true
                        }

                        // daemon control card
                        Rectangle {
                            Layout.fillWidth: true
                            implicitHeight: daemonCol.implicitHeight + 20
                            radius: 11
                            color: root.colSurface0
                            border.color: root.colSurface1
                            border.width: 1

                            ColumnLayout {
                                id: daemonCol
                                anchors.fill: parent
                                anchors.margins: 10
                                spacing: 8

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 7

                                    Rectangle {
                                        implicitWidth: 7
                                        implicitHeight: 7
                                        radius: 4
                                        color: root.isSnowhushRunning ? root.colGreen : root.colOverlay0
                                    }

                                    Text {
                                        text: root.isSnowhushRunning ? "Running" : "Stopped"
                                        font.family: root.effectiveFont
                                        font.pixelSize: 11
                                        font.weight: Font.DemiBold
                                        color: root.isSnowhushRunning ? root.colText : root.colSubtext
                                        Layout.fillWidth: true
                                    }
                                }

                                Rectangle {
                                    Layout.fillWidth: true
                                    implicitHeight: 32
                                    radius: 8
                                    color: root.isSnowhushRunning
                                        ? (runBtnMouse.containsMouse ? root.colSurface2 : root.colSurface1)
                                        : (runBtnMouse.containsMouse ? Qt.lighter(root.colAccent, 1.08) : root.colAccent)
                                    border.color: root.isSnowhushRunning
                                        ? (runBtnMouse.containsMouse ? root.colRed : root.colSurface2)
                                        : root.colAccent
                                    border.width: 1

                                    Text {
                                        anchors.centerIn: parent
                                        text: root.isSnowhushRunning ? "Stop" : "Start"
                                        font.family: root.effectiveFont
                                        font.pixelSize: 11
                                        font.bold: true
                                        color: root.isSnowhushRunning ? root.colRed : root.colCrust
                                    }

                                    MouseArea {
                                        id: runBtnMouse
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (root.isSnowhushRunning) {
                                                killer.running = true;
                                                root.isSnowhushRunning = false;
                                            } else {
                                                starter.running = true;
                                                root.isSnowhushRunning = true;
                                            }
                                        }
                                    }

                                    Behavior on color {
                                        ColorAnimation { duration: 130 }
                                    }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 6

                                    Rectangle {
                                        Layout.fillWidth: true
                                        implicitHeight: 28
                                        radius: 7
                                        color: restartMouse.containsMouse ? root.colSurface2 : root.colSurface1
                                        border.color: restartMouse.containsMouse ? root.colOverlay0 : root.colSurface2
                                        border.width: 1

                                        Row {
                                            anchors.centerIn: parent
                                            spacing: 5

                                            Text {
                                                anchors.verticalCenter: parent.verticalCenter
                                                text: "↻"
                                                font.family: root.effectiveFont
                                                font.pixelSize: 11
                                                color: restartMouse.containsMouse ? root.colText : root.colSubtext
                                            }

                                            Text {
                                                anchors.verticalCenter: parent.verticalCenter
                                                text: "Restart"
                                                font.family: root.effectiveFont
                                                font.pixelSize: 10
                                                font.weight: Font.DemiBold
                                                color: restartMouse.containsMouse ? root.colText : root.colSubtext
                                            }
                                        }

                                        MouseArea {
                                            id: restartMouse
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: restarter.running = true
                                        }

                                        Behavior on color {
                                            ColorAnimation { duration: 120 }
                                        }
                                    }

                                    Rectangle {
                                        Layout.fillWidth: true
                                        implicitHeight: 28
                                        radius: 7
                                        color: root.confirmReset
                                            ? root.colSurface2
                                            : (resetMouse.containsMouse ? root.colSurface2 : root.colSurface1)
                                        border.color: root.confirmReset
                                            ? root.colRed
                                            : (resetMouse.containsMouse ? root.colOverlay0 : root.colSurface2)
                                        border.width: 1

                                        Row {
                                            anchors.centerIn: parent
                                            spacing: 5

                                            Text {
                                                visible: !root.confirmReset
                                                anchors.verticalCenter: parent.verticalCenter
                                                text: "↺"
                                                font.family: root.effectiveFont
                                                font.pixelSize: 11
                                                color: resetMouse.containsMouse ? root.colText : root.colSubtext
                                            }

                                            Text {
                                                anchors.verticalCenter: parent.verticalCenter
                                                text: root.confirmReset ? "Confirm?" : "Reset"
                                                font.family: root.effectiveFont
                                                font.pixelSize: 10
                                                font.weight: Font.DemiBold
                                                color: root.confirmReset ? root.colRed : (resetMouse.containsMouse ? root.colText : root.colSubtext)
                                            }
                                        }

                                        MouseArea {
                                            id: resetMouse
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: {
                                                if (root.confirmReset) {
                                                    resetConfirmTimer.stop();
                                                    root.resetToDefaults();
                                                } else {
                                                    root.confirmReset = true;
                                                    resetConfirmTimer.restart();
                                                }
                                            }
                                        }

                                        Behavior on color {
                                            ColorAnimation { duration: 120 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ==================== RIGHT CONTENT PANE ====================
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: 0

                    // top header bar
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: 54
                        color: root.colBase

                        Rectangle {
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.bottom: parent.bottom
                            height: 1
                            color: root.colSurface1
                        }

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 18
                            anchors.rightMargin: 16
                            spacing: 10

                            Text {
                                text: root.isSearching
                                    ? ("Search: \"" + root.searchQuery.trim() + "\"")
                                    : root.tabMeta[root.activeTab].label
                                font.family: root.effectiveFont
                                font.pixelSize: 15
                                font.bold: true
                                color: root.colText
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            // search box
                            Rectangle {
                                Layout.preferredWidth: 185
                                implicitHeight: 30
                                radius: 8
                                color: root.colSurface0
                                border.color: searchInput.activeFocus ? root.colOverlay1 : root.colSurface1
                                border.width: 1

                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: 10
                                    anchors.rightMargin: 8
                                    spacing: 6

                                    Text {
                                        text: "⌕"
                                        font.family: root.effectiveFont
                                        font.pixelSize: 13
                                        color: root.colSubtext
                                    }

                                    TextInput {
                                        id: searchInput
                                        Layout.fillWidth: true
                                        font.family: root.effectiveFont
                                        font.pixelSize: 11
                                        color: root.colText
                                        selectionColor: root.colSurface2
                                        selectedTextColor: root.colText
                                        clip: true
                                        onTextChanged: {
                                            root.searchQuery = text;
                                            scrollArea.contentY = 0;
                                        }

                                        Text {
                                            anchors.verticalCenter: parent.verticalCenter
                                            text: "Search (Ctrl+F)"
                                            font.family: root.effectiveFont
                                            font.pixelSize: 11
                                            color: root.colOverlay1
                                            visible: searchInput.text.length === 0
                                        }
                                    }

                                    Text {
                                        visible: searchInput.text.length > 0
                                        text: "✕"
                                        font.family: root.effectiveFont
                                        font.pixelSize: 10
                                        color: clearSearchMouse.containsMouse ? root.colText : root.colSubtext

                                        MouseArea {
                                            id: clearSearchMouse
                                            anchors.fill: parent
                                            anchors.margins: -4
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: searchInput.text = ""
                                        }
                                    }
                                }

                                Behavior on border.color {
                                    ColorAnimation { duration: 120 }
                                }
                            }

                            // close button
                            Rectangle {
                                implicitWidth: 30
                                implicitHeight: 30
                                radius: 8
                                color: closeBtnMouse.containsMouse ? root.colSurface1 : root.colSurface0
                                border.color: closeBtnMouse.containsMouse ? root.colSurface2 : root.colSurface1
                                border.width: 1

                                Text {
                                    anchors.centerIn: parent
                                    text: "✕"
                                    font.family: root.effectiveFont
                                    font.pixelSize: 11
                                    font.bold: true
                                    color: closeBtnMouse.containsMouse ? root.colText : root.colSubtext
                                }

                                MouseArea {
                                    id: closeBtnMouse
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: Qt.quit()
                                }

                                Behavior on color {
                                    ColorAnimation { duration: 120 }
                                }
                            }
                        }
                    }

                    // scrollable body
                    Flickable {
                        id: scrollArea
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        contentWidth: width
                        contentHeight: contentCol.implicitHeight + 28
                        flickableDirection: Flickable.VerticalFlick
                        clip: true
                        boundsBehavior: Flickable.StopAtBounds

                        ScrollBar.vertical: ScrollBar {
                            id: vScroll
                            policy: ScrollBar.AsNeeded
                            width: 6
                            anchors.right: parent.right
                            anchors.rightMargin: 3
                            anchors.top: parent.top
                            anchors.bottom: parent.bottom
                            anchors.topMargin: 8
                            anchors.bottomMargin: 8

                            contentItem: Rectangle {
                                radius: 3
                                color: vScroll.pressed ? root.colOverlay1 : root.colSurface2
                                opacity: vScroll.active || scrollArea.moving ? 0.9 : 0.35

                                Behavior on opacity {
                                    NumberAnimation { duration: 180 }
                                }
                            }

                            background: Item {}
                        }

                        ColumnLayout {
                            id: contentCol
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.leftMargin: 18
                            anchors.rightMargin: 18
                            anchors.topMargin: 14
                            spacing: 12

                            // ==================== TAB 0: SNOWFALL ====================
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 12
                                visible: root.isSearching || root.activeTab === 0

                                SettingsCard {
                                    title: "PRESETS"
                                    visible: !root.isSearching

                                    RowLayout {
                                        Layout.fillWidth: true
                                        spacing: 8

                                        Repeater {
                                            model: [
                                                { "id": "flurries", "name": "Flurries" },
                                                { "id": "steady", "name": "Steady" },
                                                { "id": "blizzard", "name": "Blizzard" },
                                                { "id": "zerog", "name": "Zero-G" }
                                            ]

                                            Rectangle {
                                                id: presetTile
                                                required property var modelData
                                                Layout.fillWidth: true
                                                implicitHeight: 32
                                                radius: 8
                                                color: presetMouse.containsMouse ? root.colSurface2 : root.colSurface1
                                                border.color: presetMouse.containsMouse ? root.colOverlay0 : root.colSurface2
                                                border.width: 1

                                                Text {
                                                    anchors.centerIn: parent
                                                    text: presetTile.modelData.name
                                                    font.family: root.effectiveFont
                                                    font.pixelSize: 11
                                                    font.weight: Font.DemiBold
                                                    color: root.colText
                                                }

                                                MouseArea {
                                                    id: presetMouse
                                                    anchors.fill: parent
                                                    hoverEnabled: true
                                                    cursorShape: Qt.PointingHandCursor
                                                    onClicked: root.applyPreset(presetTile.modelData.id)
                                                }

                                                Behavior on color {
                                                    ColorAnimation { duration: 120 }
                                                }
                                            }
                                        }
                                    }
                                }

                                SettingsCard {
                                    title: "PRECIPITATION"

                                    RiceSliderRow {
                                        title: "Snowfall Intensity"
                                        from: 0
                                        to: 3
                                        step: 0.05
                                        value: root.optDensity
                                        defaultValue: 1.0
                                        valueText: root.optDensity <= 0.01 ? "Off" : Math.round(root.optDensity * 100) + "%"
                                        onSliderMoved: (val) => { root.optDensity = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Snow Fall Speed"
                                        from: 0.2
                                        to: 3
                                        step: 0.1
                                        value: root.optSpeedFactor
                                        defaultValue: 1.1
                                        valueText: root.optSpeedFactor.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optSpeedFactor = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Swirl & Turbulence"
                                        from: 0
                                        to: 5
                                        step: 0.1
                                        value: root.optSwayFactor
                                        defaultValue: 2.3
                                        valueText: root.optSwayFactor <= 0.05 ? "Off" : root.optSwayFactor.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optSwayFactor = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Window Ledge Leak"
                                        description: "Share of flakes slipping past window ledges"
                                        from: 0
                                        to: 1
                                        step: 0.05
                                        value: root.optWindowLeak
                                        defaultValue: 0.5
                                        valueText: Math.round(root.optWindowLeak * 100) + "%"
                                        onSliderMoved: (val) => { root.optWindowLeak = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Motion Blur Streaks"
                                        from: 0
                                        to: 3
                                        step: 0.05
                                        value: root.optMotionBlur
                                        defaultValue: 1.0
                                        valueText: root.optMotionBlur <= 0.05 ? "Off" : root.optMotionBlur.toFixed(2) + "x"
                                        onSliderMoved: (val) => { root.optMotionBlur = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Max Snowflakes"
                                        from: 100
                                        to: 8000
                                        step: 100
                                        value: root.optMaxSnow
                                        defaultValue: 2500
                                        valueText: root.optMaxSnow.toString()
                                        onSliderMoved: (val) => { root.optMaxSnow = Math.round(val); root.requestSave(); }
                                    }
                                }
                            }

                            // ==================== TAB 1: WIND & WEATHER ====================
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 12
                                visible: root.isSearching || root.activeTab === 1

                                SettingsCard {
                                    title: "WIND"

                                    RiceSliderRow {
                                        title: "Ambient Base Wind"
                                        description: root.optStaticWind === 0 ? "Calm" : (root.optStaticWind > 0 ? "Right" : "Left")
                                        from: -300
                                        to: 300
                                        step: 5
                                        value: root.optStaticWind
                                        defaultValue: 0.0
                                        valueText: (root.optStaticWind > 0 ? "+" : "") + Math.round(root.optStaticWind) + " px/s"
                                        onSliderMoved: (val) => { root.optStaticWind = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Dynamic Wind Factor"
                                        description: "Natural breeze gusts and speed fluctuations"
                                        from: 0
                                        to: 4
                                        step: 0.1
                                        value: root.optWindFactor
                                        defaultValue: 1.0
                                        valueText: root.optWindFactor <= 0.05 ? "Off" : root.optWindFactor.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optWindFactor = val; root.requestSave(); }
                                    }

                                    RiceSwitchRow {
                                        title: "Wind Snow Drift"
                                        description: "Levels drifts on light wind and scours peaks on strong wind"
                                        checked: root.optWindAccel
                                        defaultValue: true
                                        onToggled: (chk) => { root.optWindAccel = chk; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Wind Erosion Speed"
                                        from: 0
                                        to: 5
                                        step: 0.1
                                        value: root.optWindErosionRate
                                        defaultValue: 1.0
                                        valueText: root.optWindErosionRate <= 0.05 ? "Off" : root.optWindErosionRate.toFixed(1) + "x"
                                        controlEnabled: root.optWindAccel
                                        onSliderMoved: (val) => { root.optWindErosionRate = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Wind Erosion Threshold"
                                        description: "Wind speed above which peaks start scouring"
                                        from: 5
                                        to: 300
                                        step: 5
                                        value: root.optWindErosionThreshold
                                        defaultValue: 110.0
                                        valueText: Math.round(root.optWindErosionThreshold) + " px/s"
                                        controlEnabled: root.optWindAccel && root.optWindErosionRate > 0.05
                                        onSliderMoved: (val) => { root.optWindErosionThreshold = val; root.requestSave(); }
                                    }

                                    RiceSwitchRow {
                                        title: "Sweep Ground on Workspace Switch"
                                        checked: root.optSweepGround
                                        defaultValue: true
                                        onToggled: (chk) => { root.optSweepGround = chk; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Workspace Sweep Strength"
                                        from: 0
                                        to: 3
                                        step: 0.1
                                        value: root.optWorkspaceSweep
                                        defaultValue: 1.0
                                        valueText: root.optWorkspaceSweep <= 0.05 ? "Off" : root.optWorkspaceSweep.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optWorkspaceSweep = val; root.requestSave(); }
                                    }
                                }

                                SettingsCard {
                                    title: "DYNAMIC WEATHER"

                                    RiceSwitchRow {
                                        title: "Dynamic Weather Cycles"
                                        description: "Overrides manual wind and precipitation over time"
                                        checked: root.optDynamicWeather
                                        defaultValue: false
                                        onToggled: (chk) => { root.optDynamicWeather = chk; root.requestSave(); }
                                    }

                                    ColumnLayout {
                                        property string title: "Weather Phase & Mode"
                                        property string description: "Auto Calm Breeze Blizzard Squall Thaw"
                                        Layout.fillWidth: true
                                        spacing: 8
                                        visible: root.matchesSearch(title, description)
                                        enabled: root.optDynamicWeather
                                        opacity: root.optDynamicWeather ? 1.0 : 0.42

                                        Behavior on opacity {
                                            NumberAnimation { duration: 140 }
                                        }

                                        Text {
                                            text: "Weather Mode"
                                            font.family: root.effectiveFont
                                            font.pixelSize: 12
                                            font.weight: Font.DemiBold
                                            color: root.colText
                                        }

                                        RowLayout {
                                            Layout.fillWidth: true
                                            spacing: 6

                                            Repeater {
                                                model: [
                                                    { "id": "auto", "name": "Auto" },
                                                    { "id": "calm", "name": "Calm" },
                                                    { "id": "breeze", "name": "Breeze" },
                                                    { "id": "blizzard", "name": "Blizzard" },
                                                    { "id": "squall", "name": "Squall" },
                                                    { "id": "thaw", "name": "Thaw" }
                                                ]

                                                Rectangle {
                                                    id: modePill
                                                    required property var modelData
                                                    readonly property bool selected: root.optWeatherMode === modelData.id

                                                    Layout.fillWidth: true
                                                    implicitHeight: 28
                                                    radius: 7
                                                    color: selected ? root.colAccent : (modeMouse.containsMouse ? root.colSurface2 : root.colSurface1)
                                                    border.color: selected ? root.colAccent : root.colSurface2
                                                    border.width: 1

                                                    Text {
                                                        anchors.centerIn: parent
                                                        text: modePill.modelData.name
                                                        font.family: root.effectiveFont
                                                        font.pixelSize: 11
                                                        font.weight: modePill.selected ? Font.Bold : Font.Normal
                                                        color: modePill.selected ? root.colCrust : root.colText
                                                    }

                                                    MouseArea {
                                                        id: modeMouse
                                                        anchors.fill: parent
                                                        hoverEnabled: true
                                                        cursorShape: Qt.PointingHandCursor
                                                        onClicked: {
                                                            root.optWeatherMode = modePill.modelData.id;
                                                            root.requestSave();
                                                        }
                                                    }

                                                    Behavior on color {
                                                        ColorAnimation { duration: 120 }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    RiceSliderRow {
                                        title: "Weather Cycle Speed"
                                        from: 0.2
                                        to: 3
                                        step: 0.1
                                        value: root.optWeatherSpeed
                                        defaultValue: 1.4
                                        valueText: root.optWeatherSpeed.toFixed(1) + "x"
                                        controlEnabled: root.optDynamicWeather
                                        onSliderMoved: (val) => { root.optWeatherSpeed = val; root.requestSave(); }
                                    }
                                }
                            }

                            // ==================== TAB 2: WINDOWS ====================
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 12
                                visible: root.isSearching || root.activeTab === 2

                                SettingsCard {
                                    title: "LEDGE SHEDDING"

                                    RiceSliderRow {
                                        title: "Ledge Shed Force"
                                        description: "Sheds snow from a moving window's own roof"
                                        from: 0
                                        to: 3
                                        step: 0.1
                                        value: root.optMotionShed
                                        defaultValue: 1.0
                                        valueText: root.optMotionShed <= 0.05 ? "Off" : root.optMotionShed.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optMotionShed = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Ledge Shed Speed Threshold"
                                        from: 50
                                        to: 2950
                                        step: 50
                                        value: root.optShedThreshold
                                        defaultValue: 1500.0
                                        valueText: Math.round(root.optShedThreshold) + " px/s"
                                        controlEnabled: root.optMotionShed > 0.05
                                        onSliderMoved: (val) => { root.optShedThreshold = val; root.requestSave(); }
                                    }
                                }

                                SettingsCard {
                                    title: "DRIFT PLOW & SCATTER"

                                    RiceSliderRow {
                                        title: "Drift Plow Force"
                                        description: "Pushes settled snowdrifts on ground and other windows"
                                        from: 0
                                        to: 3.0
                                        step: 0.1
                                        value: root.optWindowPlow
                                        defaultValue: 1.0
                                        valueText: root.optWindowPlow <= 0.05 ? "Off" : root.optWindowPlow.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optWindowPlow = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Airborne Snow Scatter"
                                        description: "Pushes falling snowflakes around moving and opening windows"
                                        from: 0
                                        to: 3.0
                                        step: 0.05
                                        value: root.optWindowWake
                                        defaultValue: 1.0
                                        valueText: root.optWindowWake <= 0.05 ? "Off" : Math.round(root.optWindowWake * 100) + "%"
                                        onSliderMoved: (val) => { root.optWindowWake = val; root.requestSave(); }
                                    }
                                }

                                SettingsCard {
                                    title: "COMPOSITOR"

                                    RiceSliderRow {
                                        title: "Top Margin Clearance"
                                        description: "No-settle zone from screen top"
                                        from: 0
                                        to: 150
                                        step: 2
                                        value: root.optTopMargin
                                        defaultValue: 44.0
                                        valueText: Math.round(root.optTopMargin) + " px"
                                        onSliderMoved: (val) => { root.optTopMargin = val; root.requestSave(); }
                                    }

                                    RiceSwitchRow {
                                        title: "Track Quickshell Layers"
                                        description: "Settle and plow on bar, dock, and overlays"
                                        checked: root.optTrackLayers
                                        defaultValue: true
                                        onToggled: (chk) => { root.optTrackLayers = chk; root.requestSave(); }
                                    }

                                    RiceSwitchRow {
                                        title: "Disable Snow in Fullscreen"
                                        checked: root.optNoFullscreenSnow
                                        defaultValue: true
                                        onToggled: (chk) => { root.optNoFullscreenSnow = chk; root.requestSave(); }
                                    }
                                }
                            }

                            // ==================== TAB 3: CURSOR & MELT ====================
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 12
                                visible: root.isSearching || root.activeTab === 3

                                SettingsCard {
                                    title: "CURSOR & THAW"

                                    RiceSliderRow {
                                        title: "Cursor Impact Force"
                                        from: 0
                                        to: 4
                                        step: 0.1
                                        value: root.optCursorForce
                                        defaultValue: 1.5
                                        valueText: root.optCursorForce <= 0.05 ? "Off" : root.optCursorForce.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optCursorForce = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Snow Thaw Rate"
                                        description: "Natural melting speed of settled drifts"
                                        from: 0
                                        to: 10
                                        step: 0.2
                                        value: root.optMeltRate
                                        defaultValue: 4.0
                                        valueText: root.optMeltRate <= 0.05 ? "Off" : root.optMeltRate.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optMeltRate = val; root.requestSave(); }
                                    }
                                }
                            }

                            // ==================== TAB 4: EFFECTS ====================
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 12
                                visible: root.isSearching || root.activeTab === 4

                                SettingsCard {
                                    title: "CLOUDS & POWDER"

                                    RiceSliderRow {
                                        title: "Cloud Density"
                                        from: 0
                                        to: 3.0
                                        step: 0.05
                                        value: root.optCloudDensity
                                        defaultValue: 1.0
                                        valueText: root.optCloudDensity <= 0.01 ? "Off" : Math.round(root.optCloudDensity * 100) + "%"
                                        onSliderMoved: (val) => { root.optCloudDensity = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Cloud Amount"
                                        from: 0
                                        to: 4
                                        step: 0.1
                                        value: root.optCloudAmount
                                        defaultValue: 1.0
                                        valueText: root.optCloudAmount <= 0.05 ? "Off" : root.optCloudAmount.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optCloudAmount = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Cloud Size"
                                        from: 0.2
                                        to: 3
                                        step: 0.1
                                        value: root.optCloudSize
                                        defaultValue: 1.0
                                        valueText: root.optCloudSize.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optCloudSize = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Cloud Mote Randomness"
                                        from: 0
                                        to: 2.5
                                        step: 0.1
                                        value: root.optCloudRandom
                                        defaultValue: 1.0
                                        valueText: root.optCloudRandom < 0.1 ? "Off" : root.optCloudRandom.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optCloudRandom = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Cloud Acceleration"
                                        from: 0.05
                                        to: 2.0
                                        step: 0.05
                                        value: root.optCloudAccel
                                        defaultValue: 1.0
                                        valueText: root.optCloudAccel.toFixed(2) + "x"
                                        onSliderMoved: (val) => { root.optCloudAccel = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Cloud Fade Rate"
                                        from: 0.1
                                        to: 2.5
                                        step: 0.05
                                        value: root.optCloudFadeRate
                                        defaultValue: 1.0
                                        valueText: root.optCloudFadeRate.toFixed(2) + "x"
                                        onSliderMoved: (val) => { root.optCloudFadeRate = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Cloud Depth Threshold"
                                        description: "Min drift depth to emit powder clouds"
                                        from: 0.0
                                        to: 5.0
                                        step: 0.1
                                        value: root.optCloudDepthThreshold
                                        defaultValue: 1.0
                                        valueText: root.optCloudDepthThreshold <= 0.05 ? "Off" : root.optCloudDepthThreshold.toFixed(1) + " px"
                                        onSliderMoved: (val) => { root.optCloudDepthThreshold = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Cloud Depth Sensitivity"
                                        from: 0.1
                                        to: 4.0
                                        step: 0.1
                                        value: root.optCloudDepthScale
                                        defaultValue: 1.0
                                        valueText: root.optCloudDepthScale.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optCloudDepthScale = val; root.requestSave(); }
                                    }
                                }

                                SettingsCard {
                                    title: "SPRAY & GRAINS"

                                    RiceSliderRow {
                                        title: "Spray Density"
                                        from: 0.1
                                        to: 5.0
                                        step: 0.1
                                        value: root.optSprayDensity
                                        defaultValue: 1.0
                                        valueText: Math.round(root.optSprayDensity * 100) + "%"
                                        onSliderMoved: (val) => { root.optSprayDensity = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Spray Amount"
                                        from: 0.2
                                        to: 4
                                        step: 0.1
                                        value: root.optSprayAmount
                                        defaultValue: 1.0
                                        valueText: root.optSprayAmount.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optSprayAmount = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Spray Particle Size"
                                        from: 0.3
                                        to: 3.0
                                        step: 0.1
                                        value: root.optSpraySize
                                        defaultValue: 1.0
                                        valueText: root.optSpraySize.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optSpraySize = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Spray Impulse Force"
                                        from: 0.2
                                        to: 6.0
                                        step: 0.1
                                        value: root.optSprayForce
                                        defaultValue: 1.0
                                        valueText: root.optSprayForce.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optSprayForce = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Grain Flight Speed"
                                        from: 0.05
                                        to: 2.0
                                        step: 0.05
                                        value: root.optGrainSpeed
                                        defaultValue: 1.0
                                        valueText: root.optGrainSpeed.toFixed(2) + "x"
                                        onSliderMoved: (val) => { root.optGrainSpeed = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Grain Gravity"
                                        from: 0.1
                                        to: 3
                                        step: 0.1
                                        value: root.optGrainGravity
                                        defaultValue: 1.0
                                        valueText: root.optGrainGravity.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optGrainGravity = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Grain Spray Randomness"
                                        from: 0
                                        to: 2.5
                                        step: 0.1
                                        value: root.optGrainRandom
                                        defaultValue: 1.0
                                        valueText: root.optGrainRandom < 0.1 ? "Off" : root.optGrainRandom.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optGrainRandom = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Snow Ejection Force"
                                        from: 0.2
                                        to: 6.0
                                        step: 0.1
                                        value: root.optEjectionForce
                                        defaultValue: 1.0
                                        valueText: root.optEjectionForce.toFixed(1) + "x"
                                        onSliderMoved: (val) => { root.optEjectionForce = val; root.requestSave(); }
                                    }

                                    RiceSliderRow {
                                        title: "Max Effects Particles"
                                        from: 200
                                        to: 15000
                                        step: 200
                                        value: root.optMaxEffects
                                        defaultValue: 5000
                                        valueText: root.optMaxEffects.toString()
                                        onSliderMoved: (val) => { root.optMaxEffects = Math.round(val); root.requestSave(); }
                                    }
                                }
                            }
                        }
                    }

                    // compact footer bar
                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: 32
                        color: root.colBase

                        Rectangle {
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.top: parent.top
                            height: 1
                            color: root.colSurface1
                        }

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 18
                            anchors.rightMargin: 18
                            spacing: 8

                            Text {
                                text: "Alt+S toggle · Ctrl+Alt+S settings · Double-click to reset"
                                font.family: root.effectiveFont
                                font.pixelSize: 10
                                color: root.colSubtext
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            Text {
                                text: "Saved"
                                font.family: root.effectiveFont
                                font.pixelSize: 10
                                font.weight: Font.DemiBold
                                color: root.colGreen
                                opacity: root.savedFlash ? 1.0 : 0.0

                                Behavior on opacity {
                                    NumberAnimation { duration: 220 }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ==================== REUSABLE COMPONENTS ====================

    component SettingsCard: Rectangle {
        id: cardRoot
        property string title: ""
        default property alias content: cardCol.data

        property bool hasVisibleContent: {
            if (!root.isSearching)
                return true;
            for (let i = 0; i < cardCol.children.length; i++) {
                let c = cardCol.children[i];
                if (c.title !== undefined && root.matchesSearch(c.title, c.description || ""))
                    return true;
            }
            return false;
        }

        visible: hasVisibleContent
        Layout.fillWidth: true
        implicitHeight: cardOuterCol.implicitHeight + 28
        radius: 12
        color: root.colSurface0
        border.color: root.colSurface1
        border.width: 1

        ColumnLayout {
            id: cardOuterCol
            anchors.fill: parent
            anchors.margins: 14
            spacing: 10

            Text {
                visible: cardRoot.title !== ""
                text: cardRoot.title
                font.family: root.effectiveFont
                font.pixelSize: 10
                font.bold: true
                font.letterSpacing: 0.7
                color: root.colSubtext
                Layout.fillWidth: true
            }

            ColumnLayout {
                id: cardCol
                Layout.fillWidth: true
                spacing: 10
            }
        }
    }

    component RiceSliderRow: RowLayout {
        id: sliderRow

        property string title: ""
        property string description: ""
        property real from: 0
        property real to: 1
        property real step: 0.05
        property real value: 0
        property real defaultValue: 0
        property string valueText: ""
        property bool controlEnabled: true
        readonly property bool isModified: Math.abs(value - defaultValue) > step * 0.4
        readonly property real normalized: Math.max(0, Math.min(1, (value - from) / Math.max(0.0001, to - from)))

        signal sliderMoved(real val)

        visible: root.matchesSearch(title, description)
        enabled: controlEnabled
        opacity: controlEnabled ? 1.0 : 0.42
        Layout.fillWidth: true
        spacing: 14

        Behavior on opacity {
            NumberAnimation { duration: 140 }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            RowLayout {
                Layout.fillWidth: true
                spacing: 6

                Text {
                    text: sliderRow.title
                    font.family: root.effectiveFont
                    font.pixelSize: 12
                    font.weight: Font.DemiBold
                    color: root.colText
                }

                Rectangle {
                    visible: sliderRow.isModified
                    implicitWidth: 5
                    implicitHeight: 5
                    radius: 3
                    color: root.colAccent
                    opacity: 0.75
                }

                Item {
                    Layout.fillWidth: true
                }
            }

            Text {
                visible: sliderRow.description !== ""
                text: sliderRow.description
                font.family: root.effectiveFont
                font.pixelSize: 11
                color: root.colSubtext
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        RowLayout {
            spacing: 10
            Layout.alignment: Qt.AlignVCenter

            Item {
                id: trackArea
                implicitWidth: 150
                implicitHeight: 24

                Rectangle {
                    id: trackBg
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    height: 6
                    radius: 3
                    color: root.colSurface1
                    border.color: sliderMouse.containsMouse ? root.colOverlay0 : root.colSurface2
                    border.width: 1
                    clip: true

                    Rectangle {
                        anchors.left: parent.left
                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        radius: 3
                        width: thumb.x + thumb.width / 2
                        color: root.colAccent
                    }

                    Behavior on border.color {
                        ColorAnimation { duration: 120 }
                    }
                }

                Rectangle {
                    id: thumb
                    width: sliderMouse.pressed ? 16 : 14
                    height: sliderMouse.pressed ? 16 : 14
                    radius: width / 2
                    color: root.colText
                    border.color: root.colBase
                    border.width: 2
                    anchors.verticalCenter: parent.verticalCenter
                    x: Math.max(0, Math.min(trackArea.width - width, (trackArea.width - width) * sliderRow.normalized))

                    Behavior on x {
                        enabled: !sliderMouse.pressed
                        NumberAnimation { duration: 100; easing.type: Easing.OutCubic }
                    }
                    Behavior on width {
                        NumberAnimation { duration: 90 }
                    }
                    Behavior on height {
                        NumberAnimation { duration: 90 }
                    }
                }

                MouseArea {
                    id: sliderMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    cursorShape: Qt.PointingHandCursor

                    function updateFromPos(mouseX) {
                        let usable = Math.max(1, trackArea.width - thumb.width);
                        let ratio = Math.max(0, Math.min(1, (mouseX - thumb.width / 2) / usable));
                        let rawVal = sliderRow.from + ratio * (sliderRow.to - sliderRow.from);
                        let stepped = Math.round(rawVal / sliderRow.step) * sliderRow.step;
                        stepped = Math.max(sliderRow.from, Math.min(sliderRow.to, stepped));
                        sliderRow.sliderMoved(stepped);
                    }

                    onPressed: (mouse) => {
                        if (mouse.button === Qt.RightButton)
                            sliderRow.sliderMoved(sliderRow.defaultValue);
                        else
                            updateFromPos(mouse.x);
                    }
                    onPositionChanged: (mouse) => {
                        if (pressed && (mouse.buttons & Qt.LeftButton))
                            updateFromPos(mouse.x);
                    }
                    onDoubleClicked: sliderRow.sliderMoved(sliderRow.defaultValue)
                    onWheel: (wheel) => {
                        let delta = wheel.angleDelta.y > 0 ? sliderRow.step : -sliderRow.step;
                        let next = Math.round((sliderRow.value + delta) / sliderRow.step) * sliderRow.step;
                        next = Math.max(sliderRow.from, Math.min(sliderRow.to, next));
                        sliderRow.sliderMoved(next);
                    }
                }
            }

            Rectangle {
                implicitWidth: 64
                implicitHeight: 24
                radius: 6
                color: badgeMouse.containsMouse && sliderRow.isModified ? root.colSurface2 : root.colSurface1
                border.color: sliderRow.isModified ? root.colOverlay0 : root.colSurface2
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    text: sliderRow.valueText
                    font.family: root.effectiveFont
                    font.pixelSize: 11
                    font.weight: Font.DemiBold
                    color: root.colText
                }

                MouseArea {
                    id: badgeMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    cursorShape: sliderRow.isModified ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onDoubleClicked: sliderRow.sliderMoved(sliderRow.defaultValue)
                    onClicked: (mouse) => {
                        if (mouse.button === Qt.RightButton)
                            sliderRow.sliderMoved(sliderRow.defaultValue);
                    }
                }
            }
        }
    }

    component RiceSwitchRow: RowLayout {
        id: switchRow

        property string title: ""
        property string description: ""
        property bool checked: false
        property bool defaultValue: false
        property bool controlEnabled: true
        readonly property bool isModified: checked !== defaultValue

        signal toggled(bool chk)

        visible: root.matchesSearch(title, description)
        enabled: controlEnabled
        opacity: controlEnabled ? 1.0 : 0.42
        Layout.fillWidth: true
        spacing: 14

        Behavior on opacity {
            NumberAnimation { duration: 140 }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            RowLayout {
                Layout.fillWidth: true
                spacing: 6

                Text {
                    text: switchRow.title
                    font.family: root.effectiveFont
                    font.pixelSize: 12
                    font.weight: Font.DemiBold
                    color: root.colText
                }

                Rectangle {
                    visible: switchRow.isModified
                    implicitWidth: 5
                    implicitHeight: 5
                    radius: 3
                    color: root.colAccent
                    opacity: 0.75
                }

                Item {
                    Layout.fillWidth: true
                }
            }

            Text {
                visible: switchRow.description !== ""
                text: switchRow.description
                font.family: root.effectiveFont
                font.pixelSize: 11
                color: root.colSubtext
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        Rectangle {
            id: toggleTrack
            implicitWidth: 44
            implicitHeight: 24
            radius: 12
            color: switchRow.checked ? root.colAccent : root.colSurface1
            border.color: toggleMouse.containsMouse ? root.colOverlay0 : root.colSurface2
            border.width: 1

            Rectangle {
                id: toggleThumb
                width: 18
                height: 18
                radius: 9
                anchors.verticalCenter: parent.verticalCenter
                x: switchRow.checked ? (parent.width - width - 3) : 3
                color: switchRow.checked ? root.colBase : root.colText

                Behavior on x {
                    NumberAnimation { duration: 160; easing.type: Easing.OutCubic }
                }
                Behavior on color {
                    ColorAnimation { duration: 160 }
                }
            }

            MouseArea {
                id: toggleMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: switchRow.toggled(!switchRow.checked)
            }

            Behavior on color {
                ColorAnimation { duration: 160 }
            }
            Behavior on border.color {
                ColorAnimation { duration: 120 }
            }
        }
    }
}
