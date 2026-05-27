import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import com.hdaw 1.0

Rectangle {
    id: timelineArea
    Layout.fillWidth: true
    Layout.fillHeight: true
    color: "#111111"
    clip: true

    property MixerModel mixer: null
    property QtObject stripModel: null
    property bool fxVisible: false
    signal fxToggleRequested()
    property alias viewFlick: tlFlick

    TimelineModel { id: tl }
    Timer { interval: 16; running: true; repeat: true; onTriggered: tl.sync_playhead() }
    Timer { interval: 300; running: true; repeat: true; onTriggered: tl.refresh() }

    Rectangle {
        id: headerBg
        width: 220
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
                        width: 220
                        height: 90
                        color: "#222222"

                        property var stripData: (function() {
                            if (!timelineArea.stripModel) return null
                            for (var i = 0; i < timelineArea.stripModel.count; i++) {
                                if (timelineArea.stripModel.get(i).id === model.id) return timelineArea.stripModel.get(i)
                            }
                            return null
                        })()

                        Rectangle {
                            x: 2; y: 0
                            width: 4; height: parent.height
                            color: model.color || "#4caf50"
                        }

                        Text {
                            x: 10; y: 2
                            text: model.name
                            color: model.color || "#88cc88"
                            font.pixelSize: 11
                            width: 142
                            elide: Text.ElideRight
                        }

                        Text {
                            x: 154; y: 2
                            text: stripData && stripData.out ? "=>" + stripData.out.substring(0,6) : ""
                            color: "#666666"
                            font.pixelSize: 7
                        }

                        Rectangle {
                            x: 10; y: 18
                            width: 155; height: 6
                            color: "#333333"; radius: 2
                            Rectangle {
                                width: (stripData ? stripData.vol : 0.8) * parent.width
                                height: parent.height
                                color: "#4caf50"; radius: 2
                            }
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onPressed: { if (timelineArea.mixer) timelineArea.mixer.set_volume(model.id, mouseX / width) }
                                onMouseXChanged: { if (pressed && timelineArea.mixer) timelineArea.mixer.set_volume(model.id, Math.max(0, Math.min(1, mouseX / width))) }
                            }
                        }
                        Text {
                            x: 168; y: 16
                            text: stripData ? (stripData.vol === 0 ? "-inf" : (20 * Math.log10(Math.max(0.001, stripData.vol))).toFixed(1) + "dB") : "0.0dB"
                            color: "#777777"; font.pixelSize: 7
                        }

                        Rectangle {
                            x: 10; y: 26
                            width: 155; height: 4
                            color: "#333333"; radius: 1
                            Rectangle {
                                x: 2
                                width: 2; height: parent.height
                                color: "#444444"
                            }
                            Rectangle {
                                x: (156 + (stripData ? stripData.pan || 0 : 0) * 156) / 2 - 3
                                width: 6; height: 10
                                color: "#64b5f6"; radius: 2
                            }
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onPressed: { if (timelineArea.mixer) timelineArea.mixer.set_pan(model.id, (mouseX / width) * 2.0 - 1.0) }
                                onMouseXChanged: { if (pressed && timelineArea.mixer) timelineArea.mixer.set_pan(model.id, Math.max(-1, Math.min(1, (mouseX / width) * 2.0 - 1.0))) }
                            }
                        }
                        Text {
                            x: 168; y: 26
                            text: {
                                var p = stripData ? stripData.pan || 0 : 0
                                if (Math.abs(p) < 0.05) return "C"
                                return p < 0 ? "L" : "R"
                            }
                            color: "#64b5f6"; font.pixelSize: 7
                        }

                        Rectangle {
                            x: 200; y: 2; width: 6; height: 24
                            color: "#111111"
                            property real peakVal: {
                                if (timelineArea.stripModel && index < timelineArea.stripModel.count) { var s = timelineArea.stripModel.get(index); return s.peakL || 0 }
                                return 0
                            }
                            Rectangle {
                                anchors.bottom: parent.bottom
                                width: 6; height: parent.peakVal * 24
                                color: parent.peakVal > 0.85 ? "#cc3333" : (parent.peakVal > 0.65 ? "#cccc33" : "#4caf50")
                            }
                        }
                        Rectangle {
                            x: 208; y: 2; width: 6; height: 24
                            color: "#111111"
                            property real peakVal: {
                                if (timelineArea.stripModel && index < timelineArea.stripModel.count) { var s = timelineArea.stripModel.get(index); return s.peakR || 0 }
                                return 0
                            }
                            Rectangle {
                                anchors.bottom: parent.bottom
                                width: 6; height: parent.peakVal * 24
                                color: parent.peakVal > 0.85 ? "#cc3333" : (parent.peakVal > 0.65 ? "#cccc33" : "#4caf50")
                            }
                        }

                        Row {
                            x: 10; y: 32
                            spacing: 2
                            Rectangle {
                                width: 14; height: 14; radius: 7
                                color: stripData && stripData.arm ? "#cc3333" : "#333333"
                                Text { anchors.centerIn: parent; text: "R"; color: "white"; font.pixelSize: 7 }
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { if (timelineArea.mixer) timelineArea.mixer.toggle_arm(model.id) } }
                            }
                            Rectangle {
                                width: 22; height: 14; radius: 2
                                color: stripData && stripData.mut ? "#cc3333" : "#333333"
                                Text { anchors.centerIn: parent; text: "M"; color: "white"; font.pixelSize: 8 }
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { if (timelineArea.mixer) timelineArea.mixer.toggle_mute(model.id) } }
                            }
                            Rectangle {
                                width: 22; height: 14; radius: 2
                                color: stripData && stripData.sol ? "#cccc33" : "#333333"
                                Text { anchors.centerIn: parent; text: "S"; color: "white"; font.pixelSize: 8 }
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { if (timelineArea.mixer) timelineArea.mixer.toggle_solo(model.id) } }
                            }
                        }

                        Row {
                            x: 75; y: 32
                            spacing: 1
                            Repeater {
                                model: stripData ? stripData.fx || [] : []
                                Rectangle {
                                    width: 18; height: 14; radius: 1
                                    color: "#1a3a5c"
                                    Text { anchors.centerIn: parent; text: modelData; color: "#88ccff"; font.pixelSize: 7 }
                                    MouseArea {
                                        anchors.fill: parent
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: { if (timelineArea.mixer) timelineArea.mixer.select_effect(model.id, index) }
                                    }
                                }
                            }
                            Rectangle {
                                width: 14; height: 14; radius: 1
                                color: "#222222"
                                Text { anchors.centerIn: parent; text: "+"; color: "#666666"; font.pixelSize: 9 }
                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: { timelineArea.fxToggleRequested(); if (timelineArea.mixer) timelineArea.mixer.select_effect(model.id, 0) }
                                }
                            }
                        }

                        Rectangle {
                            x: 0; y: 56
                            width: 220; height: 34
                            color: "#1a1a24"
                            Text {
                                anchors.centerIn: parent
                                text: "+A"; color: "#666666"; font.pixelSize: 9
                            }
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: { /* TODO: cycle automation param */ }
                            }
                        }

                        Rectangle {
                            anchors.bottom: parent.bottom
                            width: parent.width; height: 1
                            color: "#333333"
                        }
                    }
                }
            }
        }
    }

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

            Rectangle {
                id: dragOverlay
                x: tl ? tl.drag_overlay_x : 0
                y: tl ? tl.drag_overlay_y : 0
                width: tl ? tl.drag_overlay_w : 0
                height: tl ? tl.drag_overlay_h : 0
                color: "#88ccff"
                opacity: 0.35
                border.color: "#ffffff"
                border.width: 1
                radius: 3
                visible: tl ? tl.drag_active : false
                z: 30
            }

            MouseArea {
                id: timelineMouse
                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.LeftButton
                onPressed: { if (timelineArea.tl) timelineArea.tl.on_timeline_pressed(mouseX, mouseY) }
                onMouseXChanged: { if (pressed && timelineArea.tl) timelineArea.tl.on_timeline_moved(mouseX, mouseY) }
                onMouseYChanged: { if (pressed && timelineArea.tl) timelineArea.tl.on_timeline_moved(mouseX, mouseY) }
                onReleased: { if (timelineArea.tl) timelineArea.tl.on_timeline_released() }
                cursorShape: {
                    if (!timelineArea.tl) return Qt.ArrowCursor
                    var ct = timelineArea.tl.cursor_type
                    if (ct === 0) return Qt.ArrowCursor
                    if (ct === 2) return Qt.PointingHandCursor
                    if (ct === 3) return Qt.SizeHorCursor
                    if (ct === 4) return Qt.OpenHandCursor
                    if (ct === 5) return Qt.ClosedHandCursor
                    return Qt.ArrowCursor
                }
                z: 25
            }

            Rectangle {
                x: tl ? tl.playhead_x - 1 : 0
                y: 0
                width: 2
                height: tlContent.height
                color: "#ff4444"
                z: 40
            }
        }
    }

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
                onClicked: { if (timelineArea.tl) timelineArea.tl.zoom_in() }
            }
            Button {
                text: "-"
                implicitWidth: 18
                implicitHeight: 16
                flat: true
                font.pixelSize: 10
                onClicked: { if (timelineArea.tl) timelineArea.tl.zoom_out() }
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
