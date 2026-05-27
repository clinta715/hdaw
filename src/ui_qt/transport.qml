import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import ui_qt.transport_object 1.0
import ui_qt.pool 1.0
import ui_qt.effect_editor 1.0
import ui_qt.timeline 1.0

// ---- Transport Toolbar ----
ApplicationWindow {
    id: transportWindow
    visible: true
    width: 500
    height: 40
    title: "HDAW Transport"
    flags: Qt.WindowStaysOnTopHint

    TransportBar {
        id: transport
    }

    Timer {
        interval: 16
        running: true
        repeat: true
        onTriggered: transport.syncState()
    }

    RowLayout {
        anchors.fill: parent
        spacing: 4

        Button {
            id: playBtn
            text: "\u25B6" // ▶
            onClicked: transport.play()
            implicitWidth: 36
            implicitHeight: 28
            flat: true
            background: Rectangle {
                color: transport.playing ? "#1a5c1a" : "transparent"
                radius: 3
            }
        }

        Button {
            id: stopBtn
            text: "\u25A0" // ■
            onClicked: transport.stop()
            implicitWidth: 36
            implicitHeight: 28
            flat: true
        }

        Button {
            id: recBtn
            text: "\u25CF" // ●
            onClicked: transport.toggleRecord()
            implicitWidth: 36
            implicitHeight: 28
            flat: true
            background: Rectangle {
                color: transport.recording ? "#5c1a1a" : "transparent"
                radius: 3
            }
        }

        Rectangle {
            width: 1
            height: parent.height - 4
            color: "#444444"
        }

        Label {
            text: "120 BPM"
            Layout.preferredWidth: 80
            horizontalAlignment: Text.AlignHCenter
            color: "#cccccc"
            font.pixelSize: 13
        }

        Item { Layout.fillWidth: true }  // spacer

        Button {
            text: "Import"
            onClicked: transport.importFile()
            implicitWidth: 60
            implicitHeight: 28
            flat: true
        }

        Button {
            id: poolBtn
            text: "Pool"
            onClicked: {
                transport.togglePool()
                poolWindow.visible = !poolWindow.visible
            }
            implicitWidth: 50
            implicitHeight: 28
            flat: true
            background: Rectangle {
                color: poolWindow.visible ? "#1a3a5c" : "transparent"
                radius: 3
            }
        }

        Button {
            id: fxBtn
            text: "FX"
            onClicked: {
                fxWindow.visible = !fxWindow.visible
            }
            implicitWidth: 40
            implicitHeight: 28
            flat: true
            background: Rectangle {
                color: fxWindow.visible ? "#3a5c3a" : "transparent"
                radius: 3
            }
        }

        Button {
            id: mixBtn
            text: "Mix"
            onClicked: {
                mixerWindow.visible = !mixerWindow.visible
            }
            implicitWidth: 40
            implicitHeight: 28
            flat: true
            background: Rectangle {
                color: mixerWindow.visible ? "#3a3a5c" : "transparent"
                radius: 3
            }
        }

        Button {
            id: tlBtn
            text: "TL"
            onClicked: {
                tlWindow.visible = !tlWindow.visible
            }
            implicitWidth: 36
            implicitHeight: 28
            flat: true
            background: Rectangle {
                color: tlWindow.visible ? "#5c3a1a" : "transparent"
                radius: 3
            }
        }
    }
}

// ---- Pool Panel ----
ApplicationWindow {
    id: poolWindow
    visible: false
    width: 280
    height: 300
    title: "Audio Pool"

    PoolModel {
        id: poolModel
    }

    Timer {
        interval: 250
        running: true
        repeat: true
        onTriggered: poolModel.refresh()
    }

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

                Button {
                    text: "x"
                    implicitWidth: 16
                    implicitHeight: 16
                    flat: true
                    onClicked: poolWindow.close()
                    background: Rectangle {
                        color: parent.hovered ? "#553333" : "transparent"
                        radius: 2
                    }
                    contentItem: Text {
                        text: "x"
                        color: "#aa6666"
                        font.pixelSize: 10
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }

        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: 1
            model: ListModel {
                id: poolListModel
            }

            delegate: Rectangle {
                width: parent.width
                height: 36
                color: mouseArea.containsMouse ? "#2a3a5a" : "transparent"
                radius: 3

                MouseArea {
                    id: mouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: poolModel.insertPoolAudio(model.path)
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
        var data = JSON.parse(jsonStr)
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
        onPoolJsonChanged: poolWindow.updatePool(poolModel.poolJson)
    }

    Component.onCompleted: poolModel.refresh()
}

// ---- Effect Editor Panel ----
ApplicationWindow {
    id: fxWindow
    visible: false
    width: 280
    height: 320
    title: "Effect Editor"

    EffectEditor {
        id: fx
    }

    Timer {
        interval: 16
        running: true
        repeat: true
        onTriggered: {
            if (fxWindow.visible) {
                fx.syncGr()
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header bar
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
                    id: fxTitleText
                    text: fxTitle
                    color: "#cccccc"
                    font.pixelSize: 11
                    font.bold: true
                }
                Item { Layout.fillWidth: true }
                Button {
                    text: "x"
                    implicitWidth: 16
                    implicitHeight: 16
                    flat: true
                    onClicked: fxWindow.close()
                    background: Rectangle {
                        color: parent.hovered ? "#553333" : "transparent"
                        radius: 2
                    }
                    contentItem: Text {
                        text: "x"
                        color: "#aa6666"
                        font.pixelSize: 10
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }

        // Effect chain tabs
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 28
            color: "#222222"
            Row {
                id: chainRow
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

        // Parameter area (scrollable)
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ColumnLayout {
                width: parent.width
                spacing: 0

                // Bypass checkbox
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 22
                    color: "transparent"
                    CheckBox {
                        id: bypassCheck
                        text: "Bypassed"
                        checked: fxBypassed
                        font.pixelSize: 10
                        onCheckedChanged: {
                            if (!bypassCheck.pressed) { return }
                            fx.toggleBypass()
                        }
                    }
                    Rectangle {
                        width: parent.width
                        height: 1
                        color: "#333333"
                        anchors.bottom: parent.bottom
                    }
                }

                // Parameter sliders
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
                                id: pSlider
                                Layout.fillWidth: true
                                from: model.min
                                to: model.max
                                value: model.value
                                stepSize: Math.max((model.max - model.min) / 200, 0.001)
                                onMoved: fx.setParam(model.name, value)
                            }
                        }
                    }
                }

                // Compressor GR meter
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
                            id: grText
                            text: "GR: 0.0 dB"
                            color: "#88cc88"
                            font.pixelSize: 9
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        Rectangle {
                            id: grBar
                            anchors.left: parent.left
                            anchors.leftMargin: 60
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.min(Math.max(-grValue / 60.0 * (parent.width - 80), 0), parent.width - 80)
                            height: 10
                            color: "#44aa44"
                            radius: 2
                        }
                    }
                }
            }
        }
    }

    // Models
    ListModel { id: chainModel }
    ListModel { id: paramModel }

    // State properties
    property string fxTitle: ""
    property bool fxBypassed: false
    property real grValue: 0.0

    function updateEffect(jsonStr) {
        var data = JSON.parse(jsonStr)
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

        // Update chain tabs
        chainModel.clear()
        if (data.chain) {
            for (var i = 0; i < data.chain.length; i++) {
                chainModel.append(data.chain[i])
            }
        }

        // Update parameter sliders
        paramModel.clear()
        if (data.params) {
            for (var i = 0; i < data.params.length; i++) {
                paramModel.append(data.params[i])
            }
        }
    }

    Connections {
        target: fx
        onEffectJsonChanged: fxWindow.updateEffect(fx.effectJson)
    }

    Connections {
        target: fx
        onCompressorGrChanged: {
            grValue = fx.compressorGr
        }
    }

    Component.onCompleted: fx.refresh()
}

// ---- Mixer Panel ----
ApplicationWindow {
    id: mixerWindow
    visible: false
    width: 600
    height: 220
    title: "Mixer"
    minimumWidth: 300

    MixerModel { id: mixer }

    Timer {
        interval: 16
        running: true
        repeat: true
        onTriggered: {
            if (mixerWindow.visible) {
                mixer.syncPeaks()
            }
        }
    }

    Timer {
        interval: 300
        running: true
        repeat: true
        onTriggered: {
            if (mixerWindow.visible) {
                mixer.refresh()
            }
        }
    }

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

                // Effect slots (top)
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
                                    onClicked: mixer.selectEffect(model.id, index)
                                }
                            }
                        }
                    }
                }

                // Label
                Text {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 10
                    text: model.name || ""
                    color: model.type === "master" ? "#ffaa44" : model.type === "bus" ? "#4488cc" : "#88cc88"
                    font.pixelSize: 8
                    elide: Text.ElideRight
                    horizontalAlignment: Text.AlignHCenter
                }

                // Peak meter bars
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

                        // Left channel bar
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

                        // Right channel bar
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

                // Volume fader (vertical)
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
                        onMoved: mixer.setVolume(model.id, value)
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

                // Buttons row: Mute / Solo / Arm
                Item {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 14
                    Row {
                        anchors.fill: parent
                        spacing: 1

                        // Mute
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
                                onClicked: mixer.toggleMute(model.id)
                            }
                        }

                        // Solo
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
                                onClicked: mixer.toggleSolo(model.id)
                            }
                        }

                        // Record arm (tracks only)
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
                                onClicked: mixer.toggleArm(model.id)
                            }
                        }
                    }
                }

                // Output routing label
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

    // Data models
    ListModel { id: stripModel }

    function buildStrips(jsonStr) {
        var data = JSON.parse(jsonStr)
        if (!Array.isArray(data)) { return }

        // Check if model already matches (same strips in same order)
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
        // If model already matches, only vol/mut/sol/arm might have changed.
        // Those are updated individually via setVolume/toggleMute/etc.
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
        onMixerJsonChanged: mixerWindow.buildStrips(mixer.mixerJson)
    }

    Connections {
        target: mixer
        onPeaksJsonChanged: mixerWindow.updatePeaks(mixer.peaksJson)
    }

    Component.onCompleted: mixer.refresh()
}

// ---- Timeline Panel ----
ApplicationWindow {
    id: tlWindow
    visible: false
    width: 800
    height: 280
    title: "Timeline"

    TimelineModel { id: tl }

    Timer {
        interval: 16
        running: true
        repeat: true
        onTriggered: {
            if (tlWindow.visible) {
                tl.syncPlayhead()
            }
        }
    }

    Timer {
        interval: 300
        running: true
        repeat: true
        onTriggered: {
            if (tlWindow.visible) {
                tl.refresh()
            }
        }
    }

    // Track header column (scrolls vertically with timeline)
    Rectangle {
        id: headerBg
        width: 56
        anchors.top: parent.top
        anchors.topMargin: 20
        anchors.bottom: parent.bottom
        z: 10
        color: "#1a1a1a"
    }

    // Main scrollable timeline area
    Flickable {
        id: tlFlick
        anchors.fill: parent
        anchors.topMargin: 20
        contentWidth: Math.max(tlContent.width, tlFlick.width)
        contentHeight: Math.max(tlContent.height, tlFlick.height)
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Item {
            id: tlContent
            width: totalWidth
            height: totalHeight

            // Ruler ticks
            Repeater {
                model: rulerModel
                Rectangle {
                    x: model.x - (model.major ? 0 : 0)
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

            // Ruler + track separator line
            Rectangle {
                y: -1
                width: parent.width
                height: 1
                color: "#555555"
            }

            // Track lanes with clips
            Repeater {
                model: trackModel
                Item {
                    x: 0
                    y: model.y

                    // Header label
                    Rectangle {
                        id: trackHeader
                        x: 0
                        y: 0
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

                    // Clip rectangles
                    Repeater {
                        model: model.clips || []
                        Rectangle {
                            x: modelData.x
                            y: 2
                            width: modelData.width
                            height: model.height - 4
                            radius: 3
                            color: "#336699"
                            border.color: "#5588bb"
                            border.width: 1
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

                    // Track bottom separator
                    Rectangle {
                        y: model.height - 1
                        width: parent.width
                        height: 1
                        color: "#333333"
                    }
                }
            }
        }
    }

    // Playhead (overlay on viewport)
    Rectangle {
        x: tl.playheadX - tlFlick.contentX
        y: 0
        width: 2
        height: parent.height - 2
        color: "#ff4444"
        visible: tl.playheadX > tlFlick.contentX && tl.playheadX < tlFlick.contentX + tlFlick.width
    }

    // Zoom controls (bottom-right)
    Rectangle {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 4
        width: 50
        height: 20
        radius: 3
        color: "#333333"
        z: 20
        Row {
            anchors.centerIn: parent
            spacing: 4
            Button {
                text: "+"
                implicitWidth: 18
                implicitHeight: 16
                flat: true
                font.pixelSize: 10
                onClicked: tl.zoomIn()
            }
            Button {
                text: "-"
                implicitWidth: 18
                implicitHeight: 16
                flat: true
                font.pixelSize: 10
                onClicked: tl.zoomOut()
            }
        }
    }

    // Data models
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

            // Calculate total height
            var maxY = 0
            for (var ti = 0; ti < data.tracks.length; ti++) {
                var t = data.tracks[ti]
                var endY = (t.y || ti * 90) + (t.height || 90)
                if (endY > maxY) maxY = endY
            }
            totalHeight = maxY + 4

            // Rebuild ruler
            rulerModel.clear()
            if (data.ruler_ticks) {
                for (var ri = 0; ri < data.ruler_ticks.length; ri++) {
                    rulerModel.append(data.ruler_ticks[ri])
                }
            }

            // Rebuild tracks (only if structure changed)
            var needsRebuild = (trackModel.count !== data.tracks.length)
            if (!needsRebuild) {
                for (var ti = 0; ti < data.tracks.length; ti++) {
                    if (trackModel.get(ti).id !== data.tracks[ti].id) {
                        needsRebuild = true
                        break
                    }
                    // Basic check: same number of clips
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
        onTimelineJsonChanged: tlWindow.updateTimeline(tl.timelineJson)
    }

    Component.onCompleted: tl.refresh()
}
