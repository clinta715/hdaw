import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import com.hdaw 1.0

Rectangle {
    id: mixerArea
    Layout.fillWidth: true
    Layout.preferredHeight: mixerVisible ? 220 : 0
    visible: mixerVisible
    color: "#121212"

    property bool mixerVisible: true
    property alias stripModel: stripModel

    MixerModel { id: mixer }

    Timer { interval: 16; running: true; repeat: true; onTriggered: mixer.sync_peaks() }
    Timer { interval: 300; running: true; repeat: true; onTriggered: mixer.refresh() }

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
                                    onClicked: { if (mixerArea.mixer) mixerArea.mixer.select_effect(model.id, index) }
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
                    Layout.preferredHeight: 16
                    Slider {
                        id: panSlider
                        anchors.fill: parent
                        anchors.margins: 2
                        orientation: Qt.Horizontal
                        from: -1.0
                        to: 1.0
                        value: model.pan || 0.0
                        stepSize: 0.01
                        onMoved: { if (mixerArea.mixer) mixerArea.mixer.set_pan(model.id, value) }
                        background: Rectangle {
                            x: panSlider.leftPadding
                            y: panSlider.topPadding + panSlider.availableHeight / 2 - 2
                            width: panSlider.availableWidth
                            height: 4
                            radius: 1
                            color: "#333333"
                            Rectangle {
                                x: parent.width / 2 - 1
                                width: 2; height: parent.height
                                color: "#555555"
                            }
                        }
                        handle: Rectangle {
                            x: panSlider.leftPadding + panSlider.visualPosition * (panSlider.availableWidth - 10)
                            y: panSlider.topPadding
                            width: 10; height: panSlider.availableHeight
                            radius: 2
                            color: "#64b5f6"
                            border.color: "#88ccee"
                            border.width: 1
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
                        onMoved: { if (mixerArea.mixer) mixerArea.mixer.set_volume(model.id, value) }
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
                                onClicked: { if (mixerArea.mixer) mixerArea.mixer.toggle_mute(model.id) }
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
                                onClicked: { if (mixerArea.mixer) mixerArea.mixer.toggle_solo(model.id) }
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
                                onClicked: { if (mixerArea.mixer) mixerArea.mixer.toggle_arm(model.id) }
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
