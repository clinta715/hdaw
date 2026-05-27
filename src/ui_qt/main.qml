import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import com.hdaw 1.0

// ---- Single Root ApplicationWindow ----
ApplicationWindow {
    id: mainWindow
    visible: true
    width: 1280
    height: 720
    title: "HDAW"
    minimumWidth: 800
    minimumHeight: 500

    // Bridge instances
    TransportBar { id: transport }
    PoolModel { id: poolModel }
    EffectEditor { id: fx }
    TimelineModel { id: tl }
    MixerModel { id: mixer }
    StateBridge { id: state }
    ShortcutHandler { id: sc }

    // ---- 60fps Timer ----
    Timer {
        interval: 16
        running: true
        repeat: true
        onTriggered: {
            transport.sync_state()
            tl.sync_playhead()
            mixer.sync_peaks()
            fx.sync_gr()
            state.sync_state()
            sc.sync_state()

            // auto-scroll during playback
            if (transport.playing) {
                var vw = tlFlick.width
                var sx = tlFlick.contentX
                var px = tl.playhead_x
                if (px > sx + vw * 0.8) {
                    tlFlick.contentX = px - vw * 0.5
                }
            }
        }
    }

    // ---- 300ms refresh Timer ----
    Timer {
        interval: 300
        running: true
        repeat: true
        onTriggered: {
            poolModel.refresh()
            tl.refresh()
            mixer.refresh()
        }
    }

    // ---- Keyboard Shortcuts ----
    Shortcut { sequence: "Space";     onActivated: transport.toggle_play_stop() }
    Shortcut { sequence: "Ctrl+Z";    onActivated: state.undo() }
    Shortcut { sequence: "Ctrl+Y";    onActivated: state.redo() }
    Shortcut { sequence: "L";         onActivated: state.toggle_loop() }
    Shortcut { sequence: "=";         onActivated: tl.zoom_in() }
    Shortcut { sequence: "-";         onActivated: tl.zoom_out() }
    Shortcut { sequence: "P";         onActivated: transport.toggle_pool() }
    Shortcut { sequence: "Ctrl+M";    onActivated: mixerVisible = !mixerVisible }
    Shortcut { sequence: "Ctrl+T";    onActivated: sc.add_track() }
    Shortcut { sequence: "Delete";    onActivated: sc.delete_selected() }
    Shortcut { sequence: "Ctrl+C";    onActivated: sc.copy_selected() }
    Shortcut { sequence: "Ctrl+V";    onActivated: sc.paste() }
    Shortcut { sequence: "Ctrl+A";    onActivated: sc.select_all() }
    Shortcut { sequence: "M";         onActivated: sc.toggle_mute_selected_track() }
    Shortcut { sequence: "Home";      onActivated: sc.go_to_start() }
    Shortcut { sequence: "End";       onActivated: sc.go_to_end() }
    Shortcut { sequence: "S";         onActivated: sc.toggle_tool_mode() }
    Shortcut { sequence: "N";         onActivated: sc.toggle_snap() }
    Shortcut { sequence: "Escape";    onActivated: sc.reset_tool_mode() }
    Shortcut { sequence: "Left";      onActivated: sc.nudge_left() }
    Shortcut { sequence: "Right";     onActivated: sc.nudge_right() }

    // ---- Visibility toggles ----
    property bool poolVisible: transport.pool_visible
    property bool fxVisible: false
    property bool mixerVisible: true

    // ---- Main Layout ----
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // ---- Transport Toolbar ----
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 40
            color: "#1a1a1a"

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 4
                anchors.rightMargin: 4
                spacing: 4

                Rectangle {
                    Layout.preferredWidth: 36
                    Layout.preferredHeight: 28
                    radius: 3
                    color: transport.playing ? "#1a5c1a" : "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: "\u25B6"
                        color: "#cccccc"
                        font.pixelSize: 14
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: transport.play()
                    }
                }

                Button {
                    text: "\u25A0"
                    onClicked: transport.stop()
                    implicitWidth: 36
                    implicitHeight: 28
                    flat: true
                }

                Rectangle {
                    Layout.preferredWidth: 36
                    Layout.preferredHeight: 28
                    radius: 3
                    color: transport.recording ? "#5c1a1a" : "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: "\u25CF"
                        color: "#ff8888"
                        font.pixelSize: 14
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: transport.toggle_record()
                    }
                }

                Rectangle { width: 1; height: parent.height - 4; color: "#444444" }

                Label {
                    text: "" + state.time_display
                    color: "#cccccc"
                    font.pixelSize: 13
                    font.bold: true
                    Layout.preferredWidth: 80
                    horizontalAlignment: Text.AlignHCenter
                }

                Label {
                    text: "" + state.bpm_display
                    color: "#888888"
                    font.pixelSize: 11
                    Layout.preferredWidth: 80
                    horizontalAlignment: Text.AlignHCenter
                }

                Label {
                    text: "" + state.time_sig_display
                    color: "#888888"
                    font.pixelSize: 11
                    Layout.preferredWidth: 40
                    horizontalAlignment: Text.AlignHCenter
                }

                Rectangle { width: 1; height: parent.height - 4; color: "#444444" }

                Button {
                    text: "\u21B6"
                    onClicked: state.undo()
                    implicitWidth: 32
                    implicitHeight: 28
                    flat: true
                    enabled: state.undo_available
                    opacity: enabled ? 1.0 : 0.3
                }

                Button {
                    text: "\u21B7"
                    onClicked: state.redo()
                    implicitWidth: 32
                    implicitHeight: 28
                    flat: true
                    enabled: state.redo_available
                    opacity: enabled ? 1.0 : 0.3
                }

                Rectangle { width: 1; height: parent.height - 4; color: "#444444" }

                Rectangle {
                    Layout.preferredWidth: 50
                    Layout.preferredHeight: 28
                    radius: 3
                    color: state.loop_enabled ? "#3a3a5c" : "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: "Loop"
                        color: "#cccccc"
                        font.pixelSize: 12
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: state.toggle_loop()
                    }
                }

                Item { Layout.fillWidth: true }

                Button {
                    text: "Import"
                    onClicked: transport.import_file()
                    implicitWidth: 60
                    implicitHeight: 28
                    flat: true
                }

                Rectangle {
                    Layout.preferredWidth: 50
                    Layout.preferredHeight: 28
                    radius: 3
                    color: poolVisible ? "#1a3a5c" : "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: "Pool"
                        color: "#cccccc"
                        font.pixelSize: 12
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: transport.toggle_pool()
                    }
                }

                Rectangle {
                    Layout.preferredWidth: 40
                    Layout.preferredHeight: 28
                    radius: 3
                    color: fxVisible ? "#3a5c3a" : "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: "FX"
                        color: "#cccccc"
                        font.pixelSize: 12
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: { fxVisible = !fxVisible }
                    }
                }

                Rectangle {
                    Layout.preferredWidth: 50
                    Layout.preferredHeight: 28
                    radius: 3
                    color: mixerVisible ? "#3a3a1a" : "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: "Mixer"
                        color: "#cccccc"
                        font.pixelSize: 12
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: { mixerVisible = !mixerVisible }
                    }
                }
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#333333" }

        // ---- Main Area: Pool | Timeline | FX Editor ----
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            // ---- Pool Panel (left) ----
            Rectangle {
                id: poolArea
                Layout.preferredWidth: poolVisible ? 280 : 0
                Layout.fillHeight: true
                visible: poolVisible
                clip: true
                color: "#121212"

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 24
                        color: "#1a1a1a"
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 8
                            anchors.rightMargin: 4
                            Text {
                                text: "AUDIO POOL"
                                color: "#888888"
                                font.pixelSize: 11
                                verticalAlignment: Text.AlignVCenter
                            }
                            Item { Layout.fillWidth: true }
                        }
                    }

                    ListView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        spacing: 1
                        model: ListModel { id: poolListModel }

                        delegate: Rectangle {
                            width: parent.width
                            height: 36
                            color: mouseArea.containsMouse ? "#2a3a5a" : "transparent"
                            radius: 3

                            MouseArea {
                                id: mouseArea
                                anchors.fill: parent
                                hoverEnabled: true
                                onClicked: poolModel.insert_pool_audio(model.path)
                            }

                            ColumnLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 4
                                anchors.rightMargin: 4
                                spacing: 0

                                Text {
                                    text: model.name
                                    color: "#cccccc"
                                    font.pixelSize: 10
                                    elide: Text.ElideRight
                                }

                                Text {
                                    text: model.info
                                    color: "#777777"
                                    font.pixelSize: 8
                                }
                            }
                        }
                    }
                }

                function updatePool(jsonStr) {
                    var data
                    try { data = JSON.parse(jsonStr) } catch (e) { return }
                    if (!Array.isArray(data)) return
                    poolListModel.clear()
                    for (var i = 0; i < data.length; i++) {
                        poolListModel.append({
                            name: data[i].name,
                            info: data[i].info,
                            usage: data[i].usage,
                            path: data[i].path
                        })
                    }
                }

                Connections {
                    target: poolModel
                    function onPool_jsonChanged() { poolArea.updatePool(poolModel.pool_json) }
                }

                Component.onCompleted: poolModel.refresh()
            }

            Rectangle {
                width: 1
                Layout.fillHeight: true
                color: "#333333"
                visible: poolVisible || fxVisible
            }

            // ---- Timeline (center, dominant) ----
            Rectangle {
                id: timelineArea
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: "#111111"
                clip: true

                // Track header column
                Rectangle {
                    id: headerBg
                    width: 56
                    anchors.top: parent.top
                    anchors.topMargin: 20
                    anchors.bottom: parent.bottom
                    z: 10
                    color: "#1a1a1a"

                    Flickable {
                        id: headerFlick
                        anchors.fill: parent
                        contentHeight: tlFlick.contentHeight
                        contentY: tlFlick.contentY
                        interactive: false
                        clip: true

                        Column {
                            Repeater {
                                model: trackModel
                                Rectangle {
                                    width: 56
                                    height: model.height
                                    color: "#222222"
                                    border.color: "#333333"
                                    border.width: 1
                                    Text {
                                        anchors.centerIn: parent
                                        text: model.name
                                        color: model.color || "#88cc88"
                                        font.pixelSize: 9
                                        elide: Text.ElideRight
                                        width: parent.width - 4
                                        horizontalAlignment: Text.AlignHCenter
                                    }
                                }
                            }
                        }
                    }
                }

                // Main scrollable timeline area
                Flickable {
                    id: tlFlick
                    anchors.left: headerBg.right
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    anchors.topMargin: 20
                    contentWidth: Math.max(tlContent.width, tlFlick.width)
                    contentHeight: Math.max(tlContent.height, tlFlick.height)
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds
                    interactive: true

                    Item {
                        id: tlContent
                        width: timelineArea.totalWidth
                        height: timelineArea.totalHeight

                        // Ruler ticks
                        Repeater {
                            model: rulerModel
                            Rectangle {
                                x: model.x
                                y: -20
                                width: 1
                                height: 20
                                color: model.major ? "#888888" : "#444444"
                                Text {
                                    x: 3
                                    y: 2
                                    text: model.major ? model.label : ""
                                    color: "#aaaaaa"
                                    font.pixelSize: 9
                                }
                            }
                        }

                        Rectangle {
                            y: -1
                            width: parent.width
                            height: 1
                            color: "#555555"
                        }

                        // Tracks and Clips
                        Repeater {
                            model: trackModel
                            Item {
                                x: 0
                                y: model.y

                                Repeater {
                                    model: model.clips || []
                                    Item {
                                        x: modelData.x
                                        y: 2
                                        width: modelData.width
                                        height: model.height - 4

                                        Rectangle {
                                            anchors.fill: parent
                                            radius: 3
                                            color: "#2a3a4a"
                                            visible: modelData.waveform_url === ""
                                        }
                                        Image {
                                            anchors.fill: parent
                                            source: modelData.waveform_url || ""
                                            fillMode: Image.Stretch
                                            visible: modelData.waveform_url !== ""
                                        }
                                        Image {
                                            anchors.fill: parent
                                            source: modelData.automation_url || ""
                                            fillMode: Image.Stretch
                                            visible: modelData.automation_url !== ""
                                        }

                                        Rectangle {
                                            x: 0; y: 0
                                            width: Math.min(modelData.fade_in_px || 0, parent.width)
                                            height: parent.height
                                            color: "#88ccff"; opacity: 0.25
                                            visible: (modelData.fade_in_px || 0) > 0
                                        }
                                        Rectangle {
                                            x: parent.width - Math.min(modelData.fade_out_px || 0, parent.width)
                                            y: 0
                                            width: Math.min(modelData.fade_out_px || 0, parent.width)
                                            height: parent.height
                                            color: "#88ccff"; opacity: 0.25
                                            visible: (modelData.fade_out_px || 0) > 0
                                        }

                                        Rectangle {
                                            anchors.fill: parent
                                            radius: 3
                                            color: "transparent"
                                            border.color: "#5588bb"
                                            border.width: 1
                                        }

                                        Text {
                                            anchors.centerIn: parent
                                            text: modelData.name || ""
                                            color: "#dddddd"
                                            font.pixelSize: 9
                                            elide: Text.ElideRight
                                            width: parent.width - 6
                                        }
                                    }
                                }

                                Rectangle {
                                    y: model.height - 1
                                    width: parent.width
                                    height: 1
                                    color: "#333333"
                                }
                            }
                        }

                        // Drag overlay (relative to content)
                        Rectangle {
                            id: dragOverlay
                            x: tl.drag_overlay_x
                            y: tl.drag_overlay_y
                            width: tl.drag_overlay_w
                            height: tl.drag_overlay_h
                            color: "#88ccff"
                            opacity: 0.35
                            border.color: "#ffffff"
                            border.width: 1
                            radius: 3
                            visible: tl.drag_active
                            z: 30
                        }

                        // Mouse area for drag interaction (filling content)
                        MouseArea {
                            id: timelineMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            acceptedButtons: Qt.LeftButton
                            onPressed: tl.on_timeline_pressed(mouseX, mouseY)
                            onMouseXChanged: {
                                if (pressed) tl.on_timeline_moved(mouseX, mouseY)
                            }
                            onMouseYChanged: {
                                if (pressed) tl.on_timeline_moved(mouseX, mouseY)
                            }
                            onReleased: tl.on_timeline_released()
                            cursorShape: {
                                var ct = tl.cursor_type
                                if (ct === 0) return Qt.ArrowCursor
                                if (ct === 2) return Qt.PointingHandCursor
                                if (ct === 3) return Qt.SizeHorCursor
                                if (ct === 4) return Qt.OpenHandCursor
                                if (ct === 5) return Qt.ClosedHandCursor
                                return Qt.ArrowCursor
                            }
                            z: 25
                        }

                        // Playhead (relative to content)
                        Rectangle {
                            x: tl.playhead_x - 1
                            y: 0
                            width: 2
                            height: tlContent.height
                            color: "#ff4444"
                            z: 40
                        }
                    }
                }

                // Zoom controls (fixed on viewport)
                Rectangle {
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    anchors.margins: 4
                    width: 50
                    height: 20
                    radius: 3
                    color: "#333333"
                    z: 50
                    Row {
                        anchors.centerIn: parent
                        spacing: 4
                        Button {
                            text: "+"
                            implicitWidth: 18
                            implicitHeight: 16
                            flat: true
                            font.pixelSize: 10
                            onClicked: tl.zoom_in()
                        }
                        Button {
                            text: "-"
                            implicitWidth: 18
                            implicitHeight: 16
                            flat: true
                            font.pixelSize: 10
                            onClicked: tl.zoom_out()
                        }
                    }
                }

                ListModel { id: trackModel }
                ListModel { id: rulerModel }

                property real totalWidth: 1200
                property real totalHeight: 200

                function updateTimeline(jsonStr) {
                    try {
                        var data = JSON.parse(jsonStr)
                        if (!data.tracks) { return }

                        var dur = data.duration_secs || 60
                        var zoom = data.zoom || 50
                        totalWidth = dur * zoom + 40

                        var maxY = 0
                        for (var ti = 0; ti < data.tracks.length; ti++) {
                            var t = data.tracks[ti]
                            var endY = (t.y || ti * 90) + (t.height || 90)
                            if (endY > maxY) maxY = endY
                        }
                        totalHeight = maxY + 4

                        rulerModel.clear()
                        if (data.ruler_ticks) {
                            for (var ri = 0; ri < data.ruler_ticks.length; ri++) {
                                rulerModel.append(data.ruler_ticks[ri])
                            }
                        }

                        var needsRebuild = (trackModel.count !== data.tracks.length)
                        if (!needsRebuild) {
                            for (var ti = 0; ti < data.tracks.length; ti++) {
                                if (trackModel.get(ti).id !== data.tracks[ti].id) {
                                    needsRebuild = true
                                    break
                                }
                                var oldClips = trackModel.get(ti).clips || []
                                var newClips = data.tracks[ti].clips || []
                                if (oldClips.length !== newClips.length) {
                                    needsRebuild = true
                                    break
                                }
                            }
                        }

                        if (needsRebuild) {
                            trackModel.clear()
                            for (var ti = 0; ti < data.tracks.length; ti++) {
                                var t = data.tracks[ti]
                                trackModel.append({
                                    id: t.id,
                                    name: t.name,
                                    color: t.color || "#88cc88",
                                    y: t.y || ti * 90,
                                    height: t.height || 90,
                                    clips: t.clips || []
                                })
                            }
                        }
                    } catch(e) {}
                }

                Connections {
                    target: tl
                    function onTimeline_jsonChanged() { timelineArea.updateTimeline(tl.timeline_json) }
                }

                Component.onCompleted: tl.refresh()
            }

            // ---- FX Editor (right) ----
            Rectangle {
                id: fxArea
                Layout.preferredWidth: fxVisible ? 280 : 0
                Layout.fillHeight: true
                visible: fxVisible
                clip: true
                color: "#121212"

                property string fxTitle: ""
                property bool fxBypassed: false
                property real grValue: 0.0

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 24
                        color: "#1a1a1a"
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 8
                            anchors.rightMargin: 4
                            Text {
                                text: "EFFECT"
                                color: "#888888"
                                font.pixelSize: 11
                                verticalAlignment: Text.AlignVCenter
                            }
                            Item { Layout.fillWidth: true }
                            Text {
                                text: fxArea.fxTitle
                                color: "#cccccc"
                                font.pixelSize: 11
                                font.bold: true
                            }
                            Item { Layout.fillWidth: true }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 28
                        color: "#222222"
                        Row {
                            anchors.fill: parent
                            anchors.leftMargin: 2
                            anchors.rightMargin: 2
                            anchors.topMargin: 2
                            anchors.bottomMargin: 2
                            spacing: 2
                            Repeater {
                                model: chainModel
                                Button {
                                    text: model.selected ? model.name + " *" : model.name
                                    flat: true
                                    implicitHeight: 24
                                    implicitWidth: 60
                                    background: Rectangle {
                                        color: model.selected ? "#3a5c3a" : "#2a2a2a"
                                        radius: 3
                                    }
                                }
                            }
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true

                        ColumnLayout {
                            width: parent.width
                            spacing: 0

                            Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: 22
                                color: "transparent"
                                CheckBox {
                                    text: "Bypassed"
                                    checked: fxArea.fxBypassed
                                    font.pixelSize: 10
                                    onCheckedChanged: {
                                        if (!pressed) { return }
                                        fx.toggle_bypass()
                                    }
                                }
                                Rectangle {
                                    width: parent.width
                                    height: 1
                                    color: "#333333"
                                    anchors.bottom: parent.bottom
                                }
                            }

                            Repeater {
                                model: paramModel
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 36
                                    color: "transparent"
                                    ColumnLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: 6
                                        anchors.rightMargin: 6
                                        spacing: 0
                                        Text {
                                            text: model.label + "  " + model.display
                                            color: "#aaaaaa"
                                            font.pixelSize: 9
                                        }
                                        Slider {
                                            Layout.fillWidth: true
                                            from: model.min
                                            to: model.max
                                            value: model.value
                                            stepSize: Math.max((model.max - model.min) / 200, 0.001)
                                            onMoved: fx.set_param(model.name, value)
                                        }
                                    }
                                }
                            }

                            Rectangle {
                                id: grSection
                                visible: false
                                Layout.fillWidth: true
                                Layout.preferredHeight: 32
                                color: "#1a1a2a"
                                Item {
                                    anchors.fill: parent
                                    anchors.margins: 4
                                    Text {
                                        text: "GR: 0.0 dB"
                                        color: "#88cc88"
                                        font.pixelSize: 9
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                    }
                                    Rectangle {
                                        anchors.left: parent.left
                                        anchors.leftMargin: 60
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: Math.min(Math.max(-fxArea.grValue / 60.0 * (parent.width - 80), 0), parent.width - 80)
                                        height: 10
                                        color: "#44aa44"
                                        radius: 2
                                    }
                                }
                            }
                        }
                    }
                }

                ListModel { id: chainModel }
                ListModel { id: paramModel }

                function updateEffect(jsonStr) {
                    var data
                    try { data = JSON.parse(jsonStr) } catch (e) { return }
                    if (!data.title) {
                        fxTitle = ""
                        fxBypassed = false
                        paramModel.clear()
                        chainModel.clear()
                        grSection.visible = false
                        return
                    }
                    fxTitle = data.title
                    fxBypassed = data.bypassed
                    var isCompressor = (data.effect_type === "Compressor")
                    grSection.visible = isCompressor

                    chainModel.clear()
                    if (data.chain) {
                        for (var i = 0; i < data.chain.length; i++) {
                            chainModel.append(data.chain[i])
                        }
                    }

                    paramModel.clear()
                    if (data.params) {
                        for (var i = 0; i < data.params.length; i++) {
                            paramModel.append(data.params[i])
                        }
                    }
                }

                Connections {
                    target: fx
                    function onEffect_jsonChanged() { fxArea.updateEffect(fx.effect_json) }
                }
                Connections {
                    target: fx
                    function onCompressor_grChanged() { fxArea.grValue = fx.compressor_gr }
                }

                Component.onCompleted: fx.refresh()
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#333333"; visible: mixerVisible }

        // ---- Mixer Panel (bottom) ----
        Rectangle {
            id: mixerArea
            Layout.fillWidth: true
            Layout.preferredHeight: mixerVisible ? 220 : 0
            visible: mixerVisible
            color: "#121212"
            ListView {
                id: stripList
                anchors.fill: parent
                anchors.margins: 2
                orientation: ListView.Horizontal
                model: stripModel
                spacing: 1
                clip: true

                delegate: Rectangle {
                    id: stripRoot
                    width: 50
                    height: stripList.height - 4
                    color: {
                        if (model.type === "master") return "#2a2a1a"
                        if (model.type === "bus") return "#1a1a2a"
                        return "#1a2a1a"
                    }
                    radius: 2

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 1
                        spacing: 0

                        Item {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 14
                            Row {
                                anchors.fill: parent
                                spacing: 2
                                Repeater {
                                    model: model.fx || []
                                    Rectangle {
                                        width: 14
                                        height: 12
                                        radius: 2
                                        color: "#3a5c3a"
                                        border.color: "#557755"
                                        border.width: 1
                                        Text {
                                            anchors.centerIn: parent
                                            text: modelData
                                            color: "#ccddcc"
                                            font.pixelSize: 7
                                        }
                                        MouseArea {
                                            anchors.fill: parent
                                            onClicked: mixer.select_effect(model.id, index)
                                        }
                                    }
                                }
                            }
                        }

                        Text {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 10
                            text: model.name || ""
                            color: model.type === "master" ? "#ffaa44" : model.type === "bus" ? "#4488cc" : "#88cc88"
                            font.pixelSize: 8
                            elide: Text.ElideRight
                            horizontalAlignment: Text.AlignHCenter
                        }

                        Item {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 32
                            Rectangle {
                                id: meterBg
                                anchors.fill: parent
                                anchors.margins: 1
                                color: "#111111"
                                radius: 1
                                clip: true

                                Rectangle {
                                    id: peakLBar
                                    width: 8
                                    height: Math.max(1, Math.min(parent.height * (model.peakL || 0) * 1.5, parent.height))
                                    anchors.bottom: parent.bottom
                                    anchors.left: parent.left
                                    anchors.leftMargin: 6
                                    color: {
                                        var p = (model.peakL || 0)
                                        if (p > 0.85) return "#dd4444"
                                        if (p > 0.65) return "#dddd44"
                                        return "#44dd44"
                                    }
                                    radius: 1
                                }

                                Rectangle {
                                    id: peakRBar
                                    width: 8
                                    height: Math.max(1, Math.min(parent.height * (model.peakR || 0) * 1.5, parent.height))
                                    anchors.bottom: parent.bottom
                                    anchors.right: parent.right
                                    anchors.rightMargin: 6
                                    color: {
                                        var p = (model.peakR || 0)
                                        if (p > 0.85) return "#dd4444"
                                        if (p > 0.65) return "#dddd44"
                                        return "#44dd44"
                                    }
                                    radius: 1
                                }
                            }
                        }

                        Item {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            Slider {
                                id: volSlider
                                anchors.fill: parent
                                anchors.margins: 2
                                orientation: Qt.Vertical
                                from: 0.0
                                to: 1.0
                                value: model.vol || 1.0
                                stepSize: 0.01
                                onMoved: mixer.set_volume(model.id, value)
                                background: Rectangle {
                                    x: volSlider.leftPadding + volSlider.width / 2 - 2
                                    y: volSlider.topPadding
                                    width: 4
                                    height: volSlider.availableHeight
                                    radius: 2
                                    color: "#333333"
                                }
                                handle: Rectangle {
                                    x: volSlider.leftPadding + volSlider.width / 2 - 6
                                    y: volSlider.topPadding + volSlider.visualPosition * (volSlider.availableHeight - 8)
                                    width: 12
                                    height: 6
                                    radius: 2
                                    color: volSlider.pressed ? "#cccccc" : "#888888"
                                    border.color: "#555555"
                                    border.width: 1
                                }
                            }
                        }

                        Item {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 14
                            Row {
                                anchors.fill: parent
                                spacing: 1

                                Rectangle {
                                    width: 14
                                    height: 12
                                    radius: 2
                                    color: model.mut ? "#884444" : "#333333"
                                    border.color: model.mut ? "#cc6666" : "#555555"
                                    border.width: 1
                                    Text {
                                        anchors.centerIn: parent
                                        text: "M"
                                        color: model.mut ? "#ff8888" : "#888888"
                                        font.pixelSize: 7
                                    }
                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: mixer.toggle_mute(model.id)
                                    }
                                }

                                Rectangle {
                                    width: 14
                                    height: 12
                                    radius: 2
                                    color: model.sol ? "#888844" : "#333333"
                                    border.color: model.sol ? "#cccc66" : "#555555"
                                    border.width: 1
                                    Text {
                                        anchors.centerIn: parent
                                        text: "S"
                                        color: model.sol ? "#ffff88" : "#888888"
                                        font.pixelSize: 7
                                    }
                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: mixer.toggle_solo(model.id)
                                    }
                                }

                                Rectangle {
                                    width: 14
                                    height: 12
                                    radius: 2
                                    visible: model.type === "track"
                                    color: model.arm ? "#884444" : "#333333"
                                    border.color: model.arm ? "#cc6666" : "#555555"
                                    border.width: 1
                                    Text {
                                        anchors.centerIn: parent
                                        text: "R"
                                        color: model.arm ? "#ff8888" : "#888888"
                                        font.pixelSize: 7
                                    }
                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: mixer.toggle_arm(model.id)
                                    }
                                }
                            }
                        }

                        Text {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 8
                            text: model.out || ""
                            color: "#666666"
                            font.pixelSize: 6
                            horizontalAlignment: Text.AlignHCenter
                        }
                    }
                }
            }

            ListModel { id: stripModel }

            function buildStrips(jsonStr) {
                var data
                try { data = JSON.parse(jsonStr) } catch (e) { return }
                if (!Array.isArray(data)) { return }

                var needsRebuild = (stripModel.count !== data.length)
                if (!needsRebuild) {
                    for (var i = 0; i < data.length; i++) {
                        var item = stripModel.get(i)
                        if (item.id !== data[i].id) { needsRebuild = true; break }
                    }
                }
                if (needsRebuild) {
                    stripModel.clear()
                    for (var i = 0; i < data.length; i++) {
                        var entry = data[i]
                        stripModel.append({
                            id: entry.id,
                            name: entry.name,
                            type: entry.type,
                            vol: entry.vol,
                            pan: entry.pan,
                            mut: entry.mut,
                            sol: entry.sol,
                            arm: entry.arm || false,
                            out: entry.out || "",
                            peakL: 0,
                            peakR: 0,
                            fx: entry.fx || []
                        })
                    }
                }
            }

            function updatePeaks(jsonStr) {
                try {
                    var data = JSON.parse(jsonStr)
                    for (var i = 0; i < stripModel.count; i++) {
                        var item = stripModel.get(i)
                        var peaks = data[item.id]
                        if (peaks) {
                            item.peakL = peaks.l || 0
                            item.peakR = peaks.r || 0
                        }
                    }
                } catch(e) {}
            }

            Connections {
                target: mixer
                function onMixer_jsonChanged() { mixerArea.buildStrips(mixer.mixer_json) }
            }

            Connections {
                target: mixer
                function onPeaks_jsonChanged() { mixerArea.updatePeaks(mixer.peaks_json) }
            }

            Component.onCompleted: mixer.refresh()
        }
    }
}
