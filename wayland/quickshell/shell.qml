//@ pragma AppId snowhush-settings
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Quickshell.Wayland

ShellRoot {
    // =========================================================================
    // INLINE REUSABLE COMPONENTS (Directly styled after Serpantinum Rice)
    // =========================================================================

    id: root

    // Rice colors synchronized with serpantinum qs_colors.json
    property color colBase: "#050505"
    property color colMantle: "#000000"
    property color colCrust: "#000000"
    property color colSurface0: "#141414"
    property color colSurface1: "#222222"
    property color colSurface2: "#303030"
    property color colOverlay0: "#555555"
    property color colOverlay1: "#777777"
    property color colText: "#ffffff"
    property color colSubtext: "#9e9e9e"
    property color colAccent: "#89b4fa"
    property color colBlue: "#89b4fa"
    property color colGreen: "#a6e3a1"
    property color colRed: "#f38ba8"
    property color colPeach: "#fab387"
    property color colMauve: "#cba6f7"
    property color colTeal: "#94e2d5"
    // Dynamic Font System: Theme font with fallback to installed default
    property string themeFont: "Futura"
    property string effectiveFont: "sans-serif"
    // Settings state
    property string optSnowfall: "steady"
    property real optDensity: 1
    property real optSpeedFactor: 1.1
    property real optSwayFactor: 2.3
    property real optTopMargin: 44
    property bool optTrackLayers: true
    property bool optSweepGround: true
    property bool optMelt: true
    property real optMeltRate: 4
    property bool optWindAccel: true
    property real optWindFactor: 1
    property real optWindErosionThreshold: 55
    property real optWindErosionRate: 1
    property real optStaticWind: 0
    property real optWindowLeak: 0.5
    property real optMotionShed: 1
    property real optCursorForce: 1.5
    property real optCloudDensity: 1.0
    property real optCloudAmount: 1.0
    property real optCloudSize: 1
    property real optSprayDensity: 1.0
    property real optSprayAmount: 1
    property real optSpraySize: 1.0
    property real optSprayForce: 1.0
    property real optEjectionForce: 1.0
    property real optWindowWake: 1.0
    property real optMotionBlur: 1.0
    property bool optNoFullscreenSnow: true
    property real optGrainSpeed: 1.0
    property real optGrainGravity: 1.0
    property real optGrainRandom: 1.0
    property real optCloudRandom: 1.0
    property real optCloudAccel: 1.0
    property real optCloudFadeRate: 1.0
    property real optCloudDepthThreshold: 1
    property real optCloudDepthScale: 1.0
    property int optMaxSnow: 2500
    property int optMaxEffects: 5000
    property real optWorkspaceSweep: 1
    property real optShedThreshold: 450
    property bool optDynamicWeather: false
    property string optWeatherMode: "auto"
    property real optWeatherSpeed: 1.4
    property int activeTab: 0
    property bool isSnowhushRunning: false

    function updateFont() {
        let families = Qt.fontFamilies();
        let target = root.themeFont ? root.themeFont.trim() : "";
        if (target !== "") {
            for (let i = 0; i < families.length; i++) {
                if (families[i].toLowerCase() === target.toLowerCase()) {
                    root.effectiveFont = families[i];
                    return ;
                }
            }
        }
        // Fallback defaults if theme font is not found on system
        let defaults = ["Inter", "Noto Sans", "Cantarell", "DejaVu Sans", "Roboto", "sans-serif"];
        for (let j = 0; j < defaults.length; j++) {
            for (let i = 0; i < families.length; i++) {
                if (families[i].toLowerCase() === defaults[j].toLowerCase()) {
                    root.effectiveFont = families[i];
                    return ;
                }
            }
        }
        root.effectiveFont = "sans-serif";
    }

    function requestSave() {
        saveDebounce.restart();
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
            "melt": root.optMelt,
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
    }

    function resetToDefaults() {
        root.optSnowfall = "steady";
        root.optDensity = 1.0;
        root.optSpeedFactor = 1.1;
        root.optSwayFactor = 2.3;
        root.optTopMargin = 44.0;
        root.optTrackLayers = true;
        root.optSweepGround = true;
        root.optMelt = true;
        root.optMeltRate = 4.0;
        root.optWindAccel = true;
        root.optWindFactor = 1.0;
        root.optWindErosionThreshold = 55.0;
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
        root.optShedThreshold = 450.0;
        root.optDynamicWeather = false;
        root.optWeatherMode = "auto";
        root.optWeatherSpeed = 1.4;
        root.writeConfig();
    }

    Component.onCompleted: {
        root.updateFont();
    }

    // Watch serpantinum theme settings for font and colors
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

    // Watch serpantinum colors
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
                    if (c.base)
                        root.colBase = c.base;

                    if (c.mantle)
                        root.colMantle = c.mantle;

                    if (c.crust)
                        root.colCrust = c.crust;

                    if (c.surface0)
                        root.colSurface0 = c.surface0;

                    if (c.surface1)
                        root.colSurface1 = c.surface1;

                    if (c.surface2)
                        root.colSurface2 = c.surface2;

                    if (c.overlay0)
                        root.colOverlay0 = c.overlay0;

                    if (c.overlay1)
                        root.colOverlay1 = c.overlay1;

                    if (c.text)
                        root.colText = c.text;

                    if (c.subtext0)
                        root.colSubtext = c.subtext0;

                    if (c.blue)
                        root.colBlue = c.blue;

                    if (c.accent)
                        root.colAccent = c.accent;
                    else if (c.blue)
                        root.colAccent = c.blue;
                    if (c.green)
                        root.colGreen = c.green;

                    if (c.red)
                        root.colRed = c.red;

                    if (c.peach)
                        root.colPeach = c.peach;

                    if (c.mauve)
                        root.colMauve = c.mauve;

                    if (c.teal)
                        root.colTeal = c.teal;

                } catch (e) {
                }
            }
        }
    }

    // Watch snowhush config.json
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
                        if (cfg.snowfall === "off")
                            root.optDensity = 0;
                        else if (cfg.snowfall === "light")
                            root.optDensity = 0.4;
                        else if (cfg.snowfall === "steady")
                            root.optDensity = 1;
                        else if (cfg.snowfall === "heavy")
                            root.optDensity = 2;
                    }
                    if (cfg.speed_factor !== undefined)
                        root.optSpeedFactor = Number(cfg.speed_factor);

                    if (cfg.sway_factor !== undefined)
                        root.optSwayFactor = Number(cfg.sway_factor);

                    if (cfg.top_margin !== undefined)
                        root.optTopMargin = Number(cfg.top_margin);

                    if (cfg.track_layers !== undefined)
                        root.optTrackLayers = Boolean(cfg.track_layers);

                    if (cfg.sweep_ground !== undefined)
                        root.optSweepGround = Boolean(cfg.sweep_ground);

                    if (cfg.melt !== undefined)
                        root.optMelt = Boolean(cfg.melt);

                    if (cfg.melt_rate !== undefined)
                        root.optMeltRate = Number(cfg.melt_rate);

                    if (cfg.wind_accel !== undefined)
                        root.optWindAccel = Boolean(cfg.wind_accel);

                    if (cfg.wind_factor !== undefined)
                        root.optWindFactor = Number(cfg.wind_factor);

                    if (cfg.wind_erosion_threshold !== undefined)
                        root.optWindErosionThreshold = Number(cfg.wind_erosion_threshold);

                    if (cfg.wind_erosion_rate !== undefined)
                        root.optWindErosionRate = Number(cfg.wind_erosion_rate);

                    if (cfg.static_wind !== undefined)
                        root.optStaticWind = Number(cfg.static_wind);

                    if (cfg.window_leak !== undefined)
                        root.optWindowLeak = Number(cfg.window_leak);

                    if (cfg.motion_shed !== undefined)
                        root.optMotionShed = Number(cfg.motion_shed);

                    if (cfg.cursor_force !== undefined)
                        root.optCursorForce = Number(cfg.cursor_force);

                    if (cfg.cloud_density !== undefined)
                        root.optCloudDensity = Number(cfg.cloud_density);

                    if (cfg.cloud_amount !== undefined)
                        root.optCloudAmount = Number(cfg.cloud_amount);

                    if (cfg.cloud_size !== undefined)
                        root.optCloudSize = Number(cfg.cloud_size);

                    if (cfg.spray_density !== undefined)
                        root.optSprayDensity = Number(cfg.spray_density);

                    if (cfg.spray_amount !== undefined)
                        root.optSprayAmount = Number(cfg.spray_amount);

                    if (cfg.spray_size !== undefined)
                        root.optSpraySize = Number(cfg.spray_size);

                    if (cfg.spray_force !== undefined)
                        root.optSprayForce = Number(cfg.spray_force);

                    if (cfg.ejection_force !== undefined)
                        root.optEjectionForce = Number(cfg.ejection_force);

                    if (cfg.window_wake !== undefined)
                        root.optWindowWake = Number(cfg.window_wake);

                    if (cfg.motion_blur !== undefined)
                        root.optMotionBlur = Number(cfg.motion_blur);

                    if (cfg.no_fullscreen_snow !== undefined)
                        root.optNoFullscreenSnow = Boolean(cfg.no_fullscreen_snow);

                    if (cfg.grain_speed !== undefined)
                        root.optGrainSpeed = Number(cfg.grain_speed);

                    if (cfg.grain_gravity !== undefined)
                        root.optGrainGravity = Number(cfg.grain_gravity);

                    if (cfg.grain_random !== undefined)
                        root.optGrainRandom = Number(cfg.grain_random);

                    if (cfg.cloud_random !== undefined)
                        root.optCloudRandom = Number(cfg.cloud_random);

                    if (cfg.cloud_accel !== undefined)
                        root.optCloudAccel = Number(cfg.cloud_accel);

                    if (cfg.cloud_fade_rate !== undefined)
                        root.optCloudFadeRate = Number(cfg.cloud_fade_rate);

                    if (cfg.cloud_depth_threshold !== undefined)
                        root.optCloudDepthThreshold = Number(cfg.cloud_depth_threshold);

                    if (cfg.cloud_depth_scale !== undefined)
                        root.optCloudDepthScale = Number(cfg.cloud_depth_scale);

                    if (cfg.max_snow !== undefined)
                        root.optMaxSnow = Math.round(Number(cfg.max_snow));

                    if (cfg.max_effects !== undefined)
                        root.optMaxEffects = Math.round(Number(cfg.max_effects));

                    if (cfg.workspace_sweep !== undefined)
                        root.optWorkspaceSweep = Number(cfg.workspace_sweep);

                    if (cfg.shed_threshold !== undefined)
                        root.optShedThreshold = Number(cfg.shed_threshold);

                    if (cfg.dynamic_weather !== undefined)
                        root.optDynamicWeather = Boolean(cfg.dynamic_weather);

                    if (cfg.weather_mode !== undefined)
                        root.optWeatherMode = String(cfg.weather_mode);

                    if (cfg.weather_cycle_speed !== undefined)
                        root.optWeatherSpeed = Number(cfg.weather_cycle_speed);

                } catch (e) {
                }
            }
        }
    }

    // Process checker for snowhush-wayland
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

    // Debounced config saver
    Timer {
        id: saveDebounce

        interval: 120
        repeat: false
        onTriggered: root.writeConfig()
    }

    Process {
        id: configWriter

        property string payload: ""

        command: ["sh", "-c", "cat << 'EOF' > ~/.config/snowhush/config.json\n" + payload + "\nEOF"]
    }

    Process {
        id: starter

        command: ["/home/user/.local/bin/snowhush-toggle", "start"]
        onExited: {
            statusChecker.running = true;
        }
    }

    Process {
        id: killer

        command: ["/home/user/.local/bin/snowhush-toggle", "stop"]
        onExited: {
            statusChecker.running = true;
        }
    }

    Process {
        id: restarter

        command: ["/home/user/.local/bin/snowhush-toggle", "restart"]
        onExited: {
            statusChecker.running = true;
        }
    }

    // Floating Window Configuration
    FloatingWindow {
        id: win

        title: "Snowhush Settings"
        visible: true
        implicitWidth: 640
        implicitHeight: 680
        color: "transparent"

        Shortcut {
            sequence: "Escape"
            onActivated: Qt.quit()
        }

        Rectangle {
            id: mainCard

            anchors.fill: parent
            radius: 16
            color: root.colBase
            border.color: root.colSurface1
            border.width: 1

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 18
                spacing: 14

                // ==================== HEADER ROW ====================
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6

                    // Minimalist Accent Badge (No emojis)
                    Rectangle {
                        implicitWidth: 34
                        implicitHeight: 32
                        Layout.preferredWidth: 34
                        radius: 8
                        color: root.colSurface1
                        border.color: root.colSurface2
                        border.width: 1

                        Text {
                            anchors.centerIn: parent
                            text: "SH"
                            font.family: root.effectiveFont
                            font.pixelSize: 12
                            font.bold: true
                            color: root.colAccent
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        Layout.minimumWidth: 80
                        spacing: 1

                        Text {
                            text: "Snowhush"
                            font.family: root.effectiveFont
                            font.pixelSize: 15
                            font.bold: true
                            color: root.colText
                        }

                        Text {
                            text: "Desktop snowfall physics"
                            font.family: root.effectiveFont
                            font.pixelSize: 10
                            color: root.colSubtext
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }
                    }

                    // Status Indicator Badge (constant stable width)
                    Rectangle {
                        implicitHeight: 32
                        implicitWidth: 82
                        Layout.preferredWidth: 82
                        radius: 16
                        color: root.colSurface0
                        border.color: root.colSurface1
                        border.width: 1

                        RowLayout {
                            id: statusRow
                            anchors.centerIn: parent
                            spacing: 6

                            Rectangle {
                                implicitWidth: 8
                                implicitHeight: 8
                                radius: 4
                                color: root.isSnowhushRunning ? root.colGreen : root.colOverlay0
                            }

                            Text {
                                text: root.isSnowhushRunning ? "Running" : "Stopped"
                                font.family: root.effectiveFont
                                font.pixelSize: 11
                                font.bold: true
                                color: root.isSnowhushRunning ? root.colText : root.colSubtext
                            }
                        }
                    }

                    // Reset to Defaults Button (in header where restart was)
                    Rectangle {
                        implicitWidth: 72
                        implicitHeight: 32
                        Layout.preferredWidth: 72
                        radius: 8
                        color: resetBtnHeaderMouse.containsMouse ? root.colSurface2 : root.colSurface1
                        border.color: resetBtnHeaderMouse.containsMouse ? root.colMauve : root.colSurface2
                        border.width: 1

                        Row {
                            anchors.centerIn: parent
                            spacing: 4

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: "↺"
                                font.family: root.effectiveFont
                                font.pixelSize: 12
                                color: resetBtnHeaderMouse.containsMouse ? root.colMauve : root.colSubtext
                            }

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: "Reset"
                                font.family: root.effectiveFont
                                font.pixelSize: 11
                                font.bold: true
                                color: resetBtnHeaderMouse.containsMouse ? root.colMauve : root.colSubtext
                            }
                        }

                        MouseArea {
                            id: resetBtnHeaderMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                root.resetToDefaults();
                            }
                        }

                        Behavior on color {
                            ColorAnimation {
                                duration: 150
                            }
                        }
                    }

                    // Restart Daemon Button (closer to Start)
                    Rectangle {
                        implicitWidth: 76
                        implicitHeight: 32
                        Layout.preferredWidth: 76
                        radius: 8
                        color: restartBtnMouse.containsMouse ? root.colSurface2 : root.colSurface1
                        border.color: restartBtnMouse.containsMouse ? root.colPeach : root.colSurface2
                        border.width: 1

                        Row {
                            anchors.centerIn: parent
                            spacing: 4

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: "↻"
                                font.family: root.effectiveFont
                                font.pixelSize: 12
                                color: restartBtnMouse.containsMouse ? root.colPeach : root.colSubtext
                            }

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: "Restart"
                                font.family: root.effectiveFont
                                font.pixelSize: 11
                                font.bold: true
                                color: restartBtnMouse.containsMouse ? root.colPeach : root.colSubtext
                            }
                        }

                        MouseArea {
                            id: restartBtnMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                restarter.running = true;
                            }
                        }

                        Behavior on color {
                            ColorAnimation {
                                duration: 150
                            }
                        }
                    }

                    // Run / Toggle Button (Start / Stop)
                    Rectangle {
                        implicitWidth: 72
                        implicitHeight: 32
                        Layout.preferredWidth: 72
                        radius: 8
                        color: runBtnMouse.containsMouse ? root.colSurface2 : root.colSurface1
                        border.color: root.isSnowhushRunning ? root.colSurface2 : root.colAccent
                        border.width: 1

                        Text {
                            anchors.centerIn: parent
                            text: root.isSnowhushRunning ? "Stop" : "Start"
                            font.family: root.effectiveFont
                            font.pixelSize: 11
                            font.bold: true
                            color: root.isSnowhushRunning ? root.colRed : root.colAccent
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
                            ColorAnimation {
                                duration: 150
                            }
                        }
                    }

                    // Close Button
                    Rectangle {
                        implicitWidth: 30
                        implicitHeight: 32
                        Layout.preferredWidth: 30
                        radius: 8
                        color: closeBtnMouse.containsMouse ? root.colSurface2 : root.colSurface0
                        border.color: root.colSurface1
                        border.width: 1

                        Text {
                            anchors.centerIn: parent
                            text: "✕"
                            font.family: root.effectiveFont
                            font.pixelSize: 11
                            font.bold: true
                            color: root.colText
                        }

                        MouseArea {
                            id: closeBtnMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: Qt.quit()
                        }

                        Behavior on color {
                            ColorAnimation {
                                duration: 120
                            }
                        }
                    }
                }

                // ==================== SEGMENTED TABS (No emojis) ====================
                Rectangle {
                    property var tabs: [{
                        "id": 0,
                        "label": "Snowfall"
                    }, {
                        "id": 1,
                        "label": "Wind & Swirl"
                    }, {
                        "id": 2,
                        "label": "Windows & Layers"
                    }, {
                        "id": 3,
                        "label": "Cursor & Drift"
                    }, {
                        "id": 4,
                        "label": "Effects & Clouds"
                    }]

                    Layout.fillWidth: true
                    implicitHeight: 38
                    radius: 10
                    color: root.colSurface0
                    border.color: root.colSurface1
                    border.width: 1

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 3
                        spacing: 4

                        Repeater {
                            model: parent.parent.tabs

                            Rectangle {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                radius: 8
                                color: root.activeTab === modelData.id ? root.colSurface2 : (tabMouse.containsMouse ? root.colSurface1 : "transparent")

                                Text {
                                    anchors.centerIn: parent
                                    text: modelData.label
                                    font.family: root.effectiveFont
                                    font.pixelSize: 11
                                    font.bold: root.activeTab === modelData.id
                                    color: root.activeTab === modelData.id ? root.colText : root.colSubtext
                                }

                                MouseArea {
                                    id: tabMouse

                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: root.activeTab = modelData.id
                                }

                                Behavior on color {
                                    ColorAnimation {
                                        duration: 140
                                    }

                                }

                            }

                        }

                    }

                }

                // ==================== TAB CONTENT AREA ====================
                Flickable {
                    id: scrollArea
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    contentWidth: width
                    contentHeight: contentCol.implicitHeight
                    flickableDirection: Flickable.VerticalFlick
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds

                    ColumnLayout {
                        id: contentCol

                        anchors.left: parent.left
                        anchors.right: parent.right
                        spacing: 12

                        // ==================== TAB 0: SNOWFALL ====================
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 12
                            visible: root.activeTab === 0

                            SettingsCard {
                                title: "PRECIPITATION & FLAKE DYNAMICS"

                                // 1. Snowfall Intensity
                                RiceSliderRow {
                                    title: "Snowfall Intensity"
                                    description: root.optDensity <= 0.01 ? "Snowfall disabled (0%)" : (root.optDensity < 0.6 ? "Light flurries (" + Math.round(root.optDensity * 100) + "%)" : (root.optDensity < 1.4 ? "Steady snowfall (" + Math.round(root.optDensity * 100) + "%)" : (root.optDensity < 2.2 ? "Heavy snowfall (" + Math.round(root.optDensity * 100) + "%)" : "Blizzard conditions (" + Math.round(root.optDensity * 100) + "%)")))
                                    from: 0
                                    to: 3
                                    step: 0.05
                                    value: root.optDensity
                                    valueText: Math.round(root.optDensity * 100) + "%"
                                    onSliderMoved: (val) => {
                                        root.optDensity = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 2. Snow Fall Speed
                                RiceSliderRow {
                                    title: "Snow Fall Speed"
                                    description: root.optSpeedFactor < 0.7 ? "Gentle, weightless float" : (root.optSpeedFactor > 1.6 ? "Rapid downward descent" : "Natural snowfall velocity")
                                    from: 0.2
                                    to: 3
                                    step: 0.1
                                    value: root.optSpeedFactor
                                    valueText: root.optSpeedFactor.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optSpeedFactor = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 3. Swirl & Turbulence
                                RiceSliderRow {
                                    title: "Swirl & Turbulence"
                                    description: root.optSwayFactor < 0.1 ? "Flakes fall straight down without lateral sway" : (root.optSwayFactor > 1.8 ? "Chaotic swirling vortices and gusts" : "Gentle lateral swaying in the breeze")
                                    from: 0
                                    to: 5
                                    step: 0.1
                                    value: root.optSwayFactor
                                    valueText: root.optSwayFactor.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optSwayFactor = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 4. Window Ledge Leak
                                RiceSliderRow {
                                    title: "Window Ledge Leak"
                                    description: "Percentage of flakes that slip past window ledges to lower surfaces or ground"
                                    from: 0
                                    to: 1
                                    step: 0.05
                                    value: root.optWindowLeak
                                    valueText: Math.round(root.optWindowLeak * 100) + "%"
                                    onSliderMoved: (val) => {
                                        root.optWindowLeak = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 5. Motion Blur Streaks
                                RiceSliderRow {
                                    title: "Motion Blur Streaks"
                                    description: root.optMotionBlur <= 0.05 ? "Crisp sharp particles without velocity streak smear" : (root.optMotionBlur < 1.4 ? "Natural velocity streak blur (" + root.optMotionBlur.toFixed(2) + "x)" : "Cinematic motion streak blur (" + root.optMotionBlur.toFixed(2) + "x)")
                                    from: 0
                                    to: 3
                                    step: 0.05
                                    value: root.optMotionBlur
                                    valueText: root.optMotionBlur <= 0.05 ? "Off" : root.optMotionBlur.toFixed(2) + "x"
                                    onSliderMoved: (val) => {
                                        root.optMotionBlur = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 6. Max Snowflakes Cap
                                RiceSliderRow {
                                    title: "Max Snowflakes"
                                    description: "Hard cap on simultaneously active airborne snowflakes (" + root.optMaxSnow + " flakes)"
                                    from: 100
                                    to: 8000
                                    step: 100
                                    value: root.optMaxSnow
                                    valueText: root.optMaxSnow.toString()
                                    onSliderMoved: (val) => {
                                        root.optMaxSnow = Math.round(val);
                                        root.requestSave();
                                    }
                                }

                            }

                        }

                        // ==================== TAB 1: WIND & SWIRL ====================
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 12
                            visible: root.activeTab === 1

                            SettingsCard {
                                title: "AERODYNAMICS & AIR CURRENTS"

                                // 1. Ambient Base Wind
                                RiceSliderRow {
                                    title: "Ambient Base Wind"
                                    description: root.optStaticWind === 0 ? "Calm air: flakes fall vertically" : (root.optStaticWind > 0 ? "Breeze to the right (" + Math.round(root.optStaticWind) + " px/s)" : "Breeze to the left (" + Math.round(Math.abs(root.optStaticWind)) + " px/s)")
                                    from: -150
                                    to: 150
                                    step: 5
                                    value: root.optStaticWind
                                    valueText: (root.optStaticWind > 0 ? "+" : "") + Math.round(root.optStaticWind) + " px/s"
                                    onSliderMoved: (val) => {
                                        root.optStaticWind = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 2. Dynamic Wind Factor
                                RiceSliderRow {
                                    title: "Dynamic Wind Factor"
                                    description: "Multiplier for wind gusts and airflow generated by desktop actions"
                                    from: 0.2
                                    to: 4
                                    step: 0.1
                                    value: root.optWindFactor
                                    valueText: root.optWindFactor.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optWindFactor = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 3. Wind Snow Drift
                                RiceSwitchRow {
                                    title: "Wind Snow Drift"
                                    description: "Strong horizontal wind gusts dislodge settled snow into airborne clouds"
                                    checked: root.optWindAccel
                                    onToggled: (chk) => {
                                        root.optWindAccel = chk;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 4. Wind Erosion Speed
                                RiceSliderRow {
                                    title: "Wind Erosion Speed"
                                    description: "How rapidly wind sweeps off and erodes the fragile top crests of snowdrifts"
                                    from: 0
                                    to: 5
                                    step: 0.1
                                    value: root.optWindErosionRate
                                    valueText: root.optWindErosionRate.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optWindErosionRate = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 5. Wind Erosion Threshold
                                RiceSliderRow {
                                    title: "Wind Erosion Threshold"
                                    description: "Minimum wind speed required before the wind begins peeling snow off peaks and crests"
                                    from: 5
                                    to: 200
                                    step: 5
                                    value: root.optWindErosionThreshold
                                    valueText: Math.round(root.optWindErosionThreshold) + " px/s"
                                    onSliderMoved: (val) => {
                                        root.optWindErosionThreshold = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 4. Sweep Ground on Workspace Switch
                                RiceSwitchRow {
                                    title: "Sweep Ground on Workspace Switch"
                                    description: "Clears snow accumulation on both windows and ground during workspace transitions"
                                    checked: root.optSweepGround
                                    onToggled: (chk) => {
                                        root.optSweepGround = chk;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 5. Workspace Sweep Strength
                                RiceSliderRow {
                                    title: "Workspace Sweep Strength"
                                    description: root.optWorkspaceSweep <= 0.05 ? "Disabled: workspace changes do not blow snow away" : "Strength of wind wave and snowdrift sweep during workspace switch (" + root.optWorkspaceSweep.toFixed(2) + "x)"
                                    from: 0
                                    to: 3
                                    step: 0.1
                                    value: root.optWorkspaceSweep
                                    valueText: root.optWorkspaceSweep <= 0.05 ? "Off" : root.optWorkspaceSweep.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optWorkspaceSweep = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 6. Window Air Cushion
                                RiceSliderRow {
                                    title: "Window Air Cushion"
                                    description: root.optWindowWake <= 0.05 ? "Disabled: moving windows do not deflect airborne snowflakes" : "Aerodynamic wake deflecting falling snowflakes aside without disturbing resting snowdrifts (" + Math.round(root.optWindowWake * 100) + "%)"
                                    from: 0
                                    to: 3.0
                                    step: 0.05
                                    value: root.optWindowWake
                                    valueText: root.optWindowWake <= 0.05 ? "Off" : Math.round(root.optWindowWake * 100) + "%"
                                    onSliderMoved: (val) => {
                                        root.optWindowWake = val;
                                        root.requestSave();
                                    }
                                }

                            }

                            SettingsCard {
                                title: "DYNAMIC WEATHER SYSTEM"

                                // 1. Dynamic Weather Engine
                                RiceSwitchRow {
                                    title: "Dynamic Weather Cycles"
                                    description: root.optDynamicWeather ? "Weather evolves organically (wind gusts, intensity shifts, and speed changes)" : "Disabled: manual wind, speed, and precipitation settings apply"
                                    checked: root.optDynamicWeather
                                    onToggled: (chk) => {
                                        root.optDynamicWeather = chk;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 2. Weather Preset / Mode
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 8

                                    Text {
                                        text: "Weather Phase & Mode"
                                        font.family: root.effectiveFont
                                        font.pixelSize: 13
                                        font.weight: Font.DemiBold
                                        color: root.colText
                                    }

                                    Text {
                                        text: root.optWeatherMode === "auto" ? "Automatic organic transitions: Calm -> Breeze -> Blizzard -> Squall -> Thaw" : (root.optWeatherMode === "calm" ? "Quiet flurries: gentle floating snow with near-zero wind" : (root.optWeatherMode === "breeze" ? "Cold winter breeze: slanted snowfall and steady wind" : (root.optWeatherMode === "blizzard" ? "Snowstorm: heavy snowfall, fast descent, and swirling storm gusts" : (root.optWeatherMode === "squall" ? "Arctic squall: fast icy grains and shifting winds" : "Mild thaw: sparse lazy flakes and gentle drift"))))
                                        font.family: root.effectiveFont
                                        font.pixelSize: 11
                                        color: root.colSubtext
                                        wrapMode: Text.Wrap
                                        Layout.fillWidth: true
                                    }

                                    RowLayout {
                                        Layout.fillWidth: true
                                        spacing: 6

                                        Repeater {
                                            model: [{
                                                "id": "auto",
                                                "name": "Auto"
                                            }, {
                                                "id": "calm",
                                                "name": "Calm"
                                            }, {
                                                "id": "breeze",
                                                "name": "Breeze"
                                            }, {
                                                "id": "blizzard",
                                                "name": "Blizzard"
                                            }, {
                                                "id": "squall",
                                                "name": "Squall"
                                            }, {
                                                "id": "thaw",
                                                "name": "Thaw"
                                            }]

                                            Rectangle {
                                                Layout.fillWidth: true
                                                height: 28
                                                radius: 6
                                                color: root.optWeatherMode === modelData.id ? root.colAccent : root.colSurface0
                                                border.color: root.optWeatherMode === modelData.id ? root.colAccent : root.colSurface1
                                                border.width: 1

                                                Text {
                                                    anchors.centerIn: parent
                                                    text: modelData.name
                                                    font.family: root.effectiveFont
                                                    font.pixelSize: 11
                                                    font.weight: root.optWeatherMode === modelData.id ? Font.Bold : Font.Normal
                                                    color: root.optWeatherMode === modelData.id ? root.colCrust : root.colText
                                                }

                                                MouseArea {
                                                    anchors.fill: parent
                                                    cursorShape: Qt.PointingHandCursor
                                                    onClicked: {
                                                        root.optWeatherMode = modelData.id;
                                                        root.requestSave();
                                                    }
                                                }

                                            }

                                        }

                                    }

                                }

                                RowDivider {
                                }

                                // 3. Weather Cycle Speed
                                RiceSliderRow {
                                    title: "Weather Cycle Speed"
                                    description: "How frequently weather phases transition and evolve over time"
                                    from: 0.2
                                    to: 3
                                    step: 0.1
                                    value: root.optWeatherSpeed
                                    valueText: root.optWeatherSpeed.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optWeatherSpeed = val;
                                        root.requestSave();
                                    }
                                }

                            }

                        }

                        // ==================== TAB 2: WINDOWS & LAYERS ====================
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 12
                            visible: root.activeTab === 2

                            SettingsCard {
                                title: "SURFACE & COMPOSITOR INTERACTION"

                                // 1. Top Margin Clearance
                                RiceSliderRow {
                                    title: "Top Margin Clearance"
                                    description: "Minimum distance from screen top before snow is allowed to settle"
                                    from: 0
                                    to: 150
                                    step: 2
                                    value: root.optTopMargin
                                    valueText: Math.round(root.optTopMargin) + " px"
                                    onSliderMoved: (val) => {
                                        root.optTopMargin = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 2. Window Motion Shed
                                RiceSliderRow {
                                    title: "Window Motion Shed"
                                    description: "Intensity of snow scattering and dust puffs when moving or resizing windows"
                                    from: 0.1
                                    to: 3
                                    step: 0.1
                                    value: root.optMotionShed
                                    valueText: root.optMotionShed.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optMotionShed = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 3. Window Shed Speed Threshold
                                RiceSliderRow {
                                    title: "Window Shed Speed Threshold"
                                    description: "Minimum window movement speed required before windows sweep off and shed their snow"
                                    from: 50
                                    to: 2000
                                    step: 25
                                    value: root.optShedThreshold
                                    valueText: Math.round(root.optShedThreshold) + " px/s"
                                    onSliderMoved: (val) => {
                                        root.optShedThreshold = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 4. Track Quickshell Layers
                                RiceSwitchRow {
                                    title: "Track Quickshell Layers"
                                    description: "Allow snow to settle on Serpantinum bar, dock, and floating overlays"
                                    checked: root.optTrackLayers
                                    onToggled: (chk) => {
                                        root.optTrackLayers = chk;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 4. Disable Snow in Fullscreen
                                RiceSwitchRow {
                                    title: "Disable Snow in Fullscreen"
                                    description: "Completely prevents snow from accumulating or settling anywhere while any window is fullscreen"
                                    checked: root.optNoFullscreenSnow
                                    onToggled: (chk) => {
                                        root.optNoFullscreenSnow = chk;
                                        root.requestSave();
                                    }
                                }

                            }

                            // Notice Card about Lock Screen
                            Rectangle {
                                Layout.fillWidth: true
                                implicitHeight: lockInfoCol.implicitHeight + 24
                                radius: 10
                                color: root.colSurface0
                                border.color: root.colSurface1
                                border.width: 1

                                ColumnLayout {
                                    id: lockInfoCol

                                    anchors.fill: parent
                                    anchors.margins: 14
                                    spacing: 4

                                    Text {
                                        text: "Lockscreen Window Bypass"
                                        font.family: root.effectiveFont
                                        font.pixelSize: 12
                                        font.bold: true
                                        color: root.colText
                                    }

                                    Text {
                                        text: "When the session is locked, window barriers are automatically disabled and snowflakes fall through all windows directly to the ground."
                                        font.family: root.effectiveFont
                                        font.pixelSize: 11
                                        color: root.colSubtext
                                        wrapMode: Text.WordWrap
                                        Layout.fillWidth: true
                                    }

                                }

                            }

                        }

                        // ==================== TAB 3: CURSOR & DRIFT ====================
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 12
                            visible: root.activeTab === 3

                            SettingsCard {
                                title: "CURSOR IMPULSE & THAW PHYSICS"

                                // 1. Cursor Impact Force
                                RiceSliderRow {
                                    title: "Cursor Impact Force"
                                    description: root.optCursorForce < 0.1 ? "Cursor creates no airflow (snowflakes ignore mouse)" : "Mouse movement produces air currents and sweeps snow banks into clouds"
                                    from: 0
                                    to: 4
                                    step: 0.1
                                    value: root.optCursorForce
                                    valueText: root.optCursorForce.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optCursorForce = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 2. Natural Snow Melting
                                RiceSwitchRow {
                                    title: "Natural Snow Melting"
                                    description: "Gradually thaws settled snow deposits over time to maintain visual balance"
                                    checked: root.optMelt
                                    onToggled: (chk) => {
                                        root.optMelt = chk;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 3. Snow Thaw Rate
                                RiceSliderRow {
                                    title: "Snow Thaw Rate"
                                    description: "Speed multiplier for natural snow bank dissipation"
                                    from: 0.2
                                    to: 10
                                    step: 0.2
                                    value: root.optMeltRate
                                    valueText: root.optMeltRate.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optMeltRate = val;
                                        root.requestSave();
                                    }
                                }

                            }

                        }

                        // ==================== TAB 4: EFFECTS & CLOUDS ====================
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 12
                            visible: root.activeTab === 4

                            SettingsCard {
                                title: "CLOUD MOTES & POWDER DUST"

                                // 1. Cloud Density (Opacity)
                                RiceSliderRow {
                                    title: "Cloud Density"
                                    description: "Opacity and visual thickness of billowing snow dust clouds"
                                    from: 0
                                    to: 3.0
                                    step: 0.05
                                    value: root.optCloudDensity
                                    valueText: Math.round(root.optCloudDensity * 100) + "%"
                                    onSliderMoved: (val) => {
                                        root.optCloudDensity = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 2. Cloud Amount (Quantity)
                                RiceSliderRow {
                                    title: "Cloud Amount"
                                    description: root.optCloudAmount < 0.1 ? "Cloud motes disabled (0%)" : "Frequency and volume of cloud puffs kicked up by actions (" + Math.round(root.optCloudAmount * 100) + "%)"
                                    from: 0
                                    to: 4
                                    step: 0.1
                                    value: root.optCloudAmount
                                    valueText: root.optCloudAmount.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optCloudAmount = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 3. Cloud Size
                                RiceSliderRow {
                                    title: "Cloud Size"
                                    description: "Expansion radius and physical dimensions of volumetric puffs"
                                    from: 0.2
                                    to: 3
                                    step: 0.1
                                    value: root.optCloudSize
                                    valueText: root.optCloudSize.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optCloudSize = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 4. Cloud Mote Randomness
                                RiceSliderRow {
                                    title: "Cloud Mote Randomness"
                                    description: root.optCloudRandom < 0.1 ? "Uniform: cloud puffs billow symmetrically" : "Randomization of cloud mote trajectory swirl, curl, velocity, and buoyancy (" + root.optCloudRandom.toFixed(1) + "x)"
                                    from: 0
                                    to: 2.5
                                    step: 0.1
                                    value: root.optCloudRandom
                                    valueText: root.optCloudRandom < 0.1 ? "Off" : root.optCloudRandom.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optCloudRandom = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 5. Cloud Acceleration
                                RiceSliderRow {
                                    title: "Cloud Acceleration"
                                    description: "Acceleration and kinetic speed multiplier for billowing cloud motes (" + root.optCloudAccel.toFixed(2) + "x)"
                                    from: 0.05
                                    to: 2.0
                                    step: 0.05
                                    value: root.optCloudAccel
                                    valueText: root.optCloudAccel.toFixed(2) + "x"
                                    onSliderMoved: (val) => {
                                        root.optCloudAccel = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 6. Cloud Fade Rate
                                RiceSliderRow {
                                    title: "Cloud Fade Rate"
                                    description: "Dissipation and decay speed of cloud motes into the air (" + root.optCloudFadeRate.toFixed(2) + "x)"
                                    from: 0.1
                                    to: 2.5
                                    step: 0.05
                                    value: root.optCloudFadeRate
                                    valueText: root.optCloudFadeRate.toFixed(2) + "x"
                                    onSliderMoved: (val) => {
                                        root.optCloudFadeRate = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 7. Cloud Depth Threshold
                                RiceSliderRow {
                                    title: "Cloud Depth Threshold"
                                    description: root.optCloudDepthThreshold <= 0.05 ? "Smoke triggers on all snow (even microscopic points, 0 px)" : "Minimum snowdrift depth required to emit cloud smoke (" + root.optCloudDepthThreshold.toFixed(1) + " px)"
                                    from: 0.0
                                    to: 5.0
                                    step: 0.1
                                    value: root.optCloudDepthThreshold
                                    valueText: root.optCloudDepthThreshold <= 0.05 ? "Off (0 px)" : root.optCloudDepthThreshold.toFixed(1) + " px"
                                    onSliderMoved: (val) => {
                                        root.optCloudDepthThreshold = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 8. Cloud Depth Sensitivity
                                RiceSliderRow {
                                    title: "Cloud Depth Sensitivity"
                                    description: "Rate at which smoke puff density and size scale up with snowdrift depth (" + root.optCloudDepthScale.toFixed(1) + "x)"
                                    from: 0.1
                                    to: 4.0
                                    step: 0.1
                                    value: root.optCloudDepthScale
                                    valueText: root.optCloudDepthScale.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optCloudDepthScale = val;
                                        root.requestSave();
                                    }
                                }

                            }

                            SettingsCard {
                                title: "SNOW SPRAY & SCATTER PARTICLES"

                                // 1. Spray Particle Density
                                RiceSliderRow {
                                    title: "Spray Density"
                                    description: "Visual contrast and opacity of flying grains and tumbling snow clods"
                                    from: 0.1
                                    to: 5.0
                                    step: 0.1
                                    value: root.optSprayDensity
                                    valueText: Math.round(root.optSprayDensity * 100) + "%"
                                    onSliderMoved: (val) => {
                                        root.optSprayDensity = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 2. Spray Particle Amount (Count)
                                RiceSliderRow {
                                    title: "Spray Amount"
                                    description: "Particle count multiplier for snow blasted and scattered into the air"
                                    from: 0.2
                                    to: 4
                                    step: 0.1
                                    value: root.optSprayAmount
                                    valueText: root.optSprayAmount.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optSprayAmount = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 3. Spray Particle Size
                                RiceSliderRow {
                                    title: "Spray Particle Size"
                                    description: "Scale of individual snow crystals, physical grains, and flying chunks"
                                    from: 0.3
                                    to: 3.0
                                    step: 0.1
                                    value: root.optSpraySize
                                    valueText: root.optSpraySize.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optSpraySize = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 4. Spray Force / Velocity
                                RiceSliderRow {
                                    title: "Spray Impulse Force"
                                    description: "Ejection speed and explosive fountain height when windows crush or fling snow"
                                    from: 0.2
                                    to: 6.0
                                    step: 0.1
                                    value: root.optSprayForce
                                    valueText: root.optSprayForce.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optSprayForce = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 5. Grain Flight Speed
                                RiceSliderRow {
                                    title: "Grain Flight Speed"
                                    description: "Velocity multiplier governing how quickly physical grains travel through the air"
                                    from: 0.05
                                    to: 2.0
                                    step: 0.05
                                    value: root.optGrainSpeed
                                    valueText: root.optGrainSpeed.toFixed(2) + "x"
                                    onSliderMoved: (val) => {
                                        root.optGrainSpeed = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 6. Grain Gravity
                                RiceSliderRow {
                                    title: "Grain Gravity"
                                    description: root.optGrainGravity < 0.5 ? "Light airy grains float gently in the air before descending" : (root.optGrainGravity > 1.6 ? "Heavy dense grains drop rapidly to the ground" : "Natural gravitational pull on physical snow crystals")
                                    from: 0.1
                                    to: 3
                                    step: 0.1
                                    value: root.optGrainGravity
                                    valueText: root.optGrainGravity.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optGrainGravity = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 7. Grain Spray Randomness
                                RiceSliderRow {
                                    title: "Grain Spray Randomness"
                                    description: root.optGrainRandom < 0.1 ? "Uniform: grains follow exact parabolic arc" : "Randomization of powder grain trajectory jitter, speed spread, and gravity (" + root.optGrainRandom.toFixed(1) + "x)"
                                    from: 0
                                    to: 2.5
                                    step: 0.1
                                    value: root.optGrainRandom
                                    valueText: root.optGrainRandom < 0.1 ? "Off" : root.optGrainRandom.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optGrainRandom = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 7. Snow Ejection Force
                                RiceSliderRow {
                                    title: "Snow Ejection Force"
                                    description: "Kickback impulse and violent ejection speed when wind erodes snowdrifts or blasts snowbanks"
                                    from: 0.2
                                    to: 6.0
                                    step: 0.1
                                    value: root.optEjectionForce
                                    valueText: root.optEjectionForce.toFixed(1) + "x"
                                    onSliderMoved: (val) => {
                                        root.optEjectionForce = val;
                                        root.requestSave();
                                    }
                                }

                                RowDivider {
                                }

                                // 8. Max Effects Particles Cap
                                RiceSliderRow {
                                    title: "Max Effects Particles"
                                    description: "Hard cap on total physical spray grains and cloud motes (" + root.optMaxEffects + " particles)"
                                    from: 200
                                    to: 15000
                                    step: 200
                                    value: root.optMaxEffects
                                    valueText: root.optMaxEffects.toString()
                                    onSliderMoved: (val) => {
                                        root.optMaxEffects = Math.round(val);
                                        root.requestSave();
                                    }
                                }

                            }

                        }

                    }

                }

                // ==================== FOOTER ====================
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8

                    Text {
                        text: "Shortcuts: Alt+S (toggle), Ctrl+Alt+S (settings)"
                        font.family: root.effectiveFont
                        font.pixelSize: 10
                        color: root.colSubtext
                        Layout.fillWidth: true
                    }
                }

            }

        }

    }

    // Card Container (like SettingsBlob in Serpantinum)
    component SettingsCard: Rectangle {
        property string title: ""
        default property alias content: cardCol.data

        Layout.fillWidth: true
        implicitHeight: cardOuterCol.implicitHeight + 20
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
                visible: title !== ""
                text: title
                font.family: root.effectiveFont
                font.pixelSize: 10
                font.bold: true
                color: root.colSubtext
                Layout.fillWidth: true
            }

            ColumnLayout {
                id: cardCol

                Layout.fillWidth: true
                spacing: 0
            }

        }

    }

    // Thin row divider
    component RowDivider: Rectangle {
        Layout.fillWidth: true
        implicitHeight: 1
        color: root.colSurface1
        Layout.topMargin: 8
        Layout.bottomMargin: 8
    }

    // Slider Row (Styled after Serpantinum SliderProgress)
    component RiceSliderRow: RowLayout {
        id: sliderRow

        property string title: ""
        property string description: ""
        property real from: 0
        property real to: 1
        property real step: 0.05
        property real value: 0
        property string valueText: ""

        signal sliderMoved(real val)

        Layout.fillWidth: true
        spacing: 12

        // Title & Description
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Text {
                text: sliderRow.title
                font.family: root.effectiveFont
                font.pixelSize: 12
                font.bold: true
                color: root.colText
            }

            Text {
                text: sliderRow.description
                font.family: root.effectiveFont
                font.pixelSize: 11
                color: root.colSubtext
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

        }

        // Slider Control Area
        RowLayout {
            spacing: 8
            Layout.alignment: Qt.AlignVCenter

            // The Slider Track & Fill
            Item {
                id: trackArea

                implicitWidth: 160
                implicitHeight: 24

                Rectangle {
                    id: trackBg

                    anchors.fill: parent
                    radius: 12
                    color: root.colSurface1
                    border.color: sliderMouse.containsMouse ? root.colOverlay0 : root.colSurface2
                    border.width: 1
                    clip: true

                    // Progress Fill Bar
                    Rectangle {
                        anchors.left: parent.left
                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        radius: 12
                        width: Math.max(0, Math.min(trackArea.width, trackArea.width * ((sliderRow.value - sliderRow.from) / Math.max(0.0001, sliderRow.to - sliderRow.from))))
                        color: root.colAccent
                        opacity: 0.85

                        Behavior on width {
                            enabled: !sliderMouse.drag.active

                            NumberAnimation {
                                duration: 120
                                easing.type: Easing.OutCubic
                            }

                        }

                    }

                    Behavior on border.color {
                        ColorAnimation {
                            duration: 150
                        }

                    }

                }

                // Draggable Thumb Handle
                Rectangle {
                    id: thumb

                    width: 18
                    height: 18
                    radius: 9
                    color: root.colBase
                    border.color: root.colText
                    border.width: 2
                    anchors.verticalCenter: parent.verticalCenter
                    x: Math.max(0, Math.min(trackArea.width - width, (trackArea.width - width) * ((sliderRow.value - sliderRow.from) / Math.max(0.0001, sliderRow.to - sliderRow.from))))

                    Behavior on x {
                        enabled: !sliderMouse.drag.active

                        NumberAnimation {
                            duration: 120
                            easing.type: Easing.OutCubic
                        }

                    }

                }

                MouseArea {
                    id: sliderMouse

                    function updateFromPos(mouseX) {
                        let ratio = Math.max(0, Math.min(1, mouseX / trackArea.width));
                        let rawVal = sliderRow.from + ratio * (sliderRow.to - sliderRow.from);
                        let stepped = Math.round(rawVal / sliderRow.step) * sliderRow.step;
                        stepped = Math.max(sliderRow.from, Math.min(sliderRow.to, stepped));
                        sliderRow.sliderMoved(stepped);
                    }

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onPressed: (mouse) => {
                        return updateFromPos(mouse.x);
                    }
                    onPositionChanged: (mouse) => {
                        if (pressed)
                            updateFromPos(mouse.x);

                    }
                }

            }

            // Numerical Value Badge
            Rectangle {
                implicitWidth: 64
                implicitHeight: 24
                radius: 6
                color: root.colSurface1
                border.color: root.colSurface2
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    text: sliderRow.valueText
                    font.family: root.effectiveFont
                    font.pixelSize: 11
                    font.bold: true
                    color: root.colText
                }

            }

        }

    }

    // Switch Row (Styled after Serpantinum Toggle)
    component RiceSwitchRow: RowLayout {
        id: switchRow

        property string title: ""
        property string description: ""
        property bool checked: false

        signal toggled(bool chk)

        Layout.fillWidth: true
        spacing: 12

        // Title & Description
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Text {
                text: switchRow.title
                font.family: root.effectiveFont
                font.pixelSize: 12
                font.bold: true
                color: root.colText
            }

            Text {
                text: switchRow.description
                font.family: root.effectiveFont
                font.pixelSize: 11
                color: root.colSubtext
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

        }

        // Toggle Switch Track
        Rectangle {
            id: toggleTrack

            implicitWidth: 46
            implicitHeight: 24
            radius: 12
            color: switchRow.checked ? root.colAccent : root.colSurface1
            border.color: toggleMouse.containsMouse ? root.colOverlay0 : root.colSurface2
            border.width: 1

            // Sliding Thumb
            Rectangle {
                id: toggleThumb

                width: 18
                height: 18
                radius: 9
                anchors.verticalCenter: parent.verticalCenter
                x: switchRow.checked ? (parent.width - width - 3) : 3
                color: switchRow.checked ? root.colBase : root.colText

                Behavior on x {
                    NumberAnimation {
                        duration: 180
                        easing.type: Easing.OutCubic
                    }

                }

                Behavior on color {
                    ColorAnimation {
                        duration: 180
                    }

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
                ColorAnimation {
                    duration: 180
                }

            }

            Behavior on border.color {
                ColorAnimation {
                    duration: 140
                }

            }

        }

    }

}
