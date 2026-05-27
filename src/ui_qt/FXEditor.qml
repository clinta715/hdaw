import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import com.hdaw 1.0

Rectangle {
    id: fxArea
    Layout.preferredWidth: fxVisible ? 220 : 0
    Layout.fillHeight: true
    visible: fxVisible
    clip: true
    color: "#121212"

    property bool fxVisible: false

    EffectEditor { id: fx }

    Timer { interval: 16; running: true; repeat: true; onTriggered: fx.sync_gr() }

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
                anchors.verticalCenter: parent.verticalCenter
                Repeater {
                    model: chainModel
                    Rectangle {
                        height: parent.height - 4
                        width: 55
                        radius: 1
                        color: model.selected ? "#3a5c3a" : "#222222"
                        Text {
                            anchors.centerIn: parent
                            text: (model.selected ? "*" : "") + model.name
                            color: model.selected ? "#88cc88" : "#666666"
                            font.pixelSize: 8
                        }
                    }
                }
            }
        }

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
                    if (fx) fx.toggle_bypass()
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
                    RowLayout {
                        Layout.fillWidth: true
                        Text {
                            text: model.label
                            color: "#aaaaaa"
                            font.pixelSize: 9
                            Layout.preferredWidth: 60
                            elide: Text.ElideRight
                        }
                        Text {
                            text: model.display
                            color: "#8bc34a"
                            font.pixelSize: 8
                            Layout.preferredWidth: 60
                            horizontalAlignment: Text.AlignRight
                        }
                    }
                    Slider {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 16
                        from: model.min
                        to: model.max
                        value: model.value
                        stepSize: (model.max - model.min) / 200.0
                        onMoved: { if (fx) fx.set_param(model.name, value) }
                    }
                }
            }
        }

        Rectangle {
            id: grSection
            Layout.fillWidth: true
            Layout.preferredHeight: 32
            color: "#1a1a2a"
            visible: false
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

    ListModel { id: chainModel }
    ListModel { id: paramModel }

    function updateEffect(jsonStr) {
        var data
        try { data = JSON.parse(jsonStr) } catch(e) { return }
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
