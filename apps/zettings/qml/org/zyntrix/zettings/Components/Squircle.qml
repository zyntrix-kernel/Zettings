import QtQuick
import org.zyntrix.zettings.Style
import QtQuick.Shapes

Shape {
    id: root

    property real radius: ZdlTheme.radiusCard
    property real exponent: 4.0
    property color fillColor: "transparent"
    property color strokeColor: "transparent"
    property real strokeWidth: 1

    readonly property real effectiveExponent: {
        const halfMin = Math.min(width, height) / 2;
        if (halfMin <= 0 || radius <= 0)
            return 2.0;
        return 2.0 + (exponent - 2.0) * Math.min(radius, halfMin) / halfMin;
    }

    antialiasing: true

    function superellipsePoints(n, samples) {
        const hw = width / 2 - strokeWidth / 2;
        const hh = height / 2 - strokeWidth / 2;
        const pts = [];
        if (hw <= 0 || hh <= 0)
            return pts;
        for (let i = 0; i < samples; i++) {
            const t = (i / samples) * 2 * Math.PI;
            const ct = Math.cos(t);
            const st = Math.sin(t);
            const x = hw * Math.sign(ct) * Math.pow(Math.abs(ct), 2 / n);
            const y = hh * Math.sign(st) * Math.pow(Math.abs(st), 2 / n);
            pts.push(Qt.point(hw + x, hh + y));
        }
        return pts;
    }

    function buildPathData(samples) {
        const pts = superellipsePoints(effectiveExponent, samples);
        if (pts.length === 0)
            return "";
        let d = "M " + pts[0].x.toFixed(2) + " " + pts[0].y.toFixed(2);
        for (let i = 1; i <= pts.length; i++) {
            const p = pts[i % pts.length];
            d += " L " + p.x.toFixed(2) + " " + p.y.toFixed(2);
        }
        return d + " Z";
    }

    ShapePath {
        strokeWidth: root.strokeWidth
        fillColor: root.fillColor
        strokeColor: root.strokeColor
        // qmllint disable missing-property
        joinStyle: ShapePath.Round
        // qmllint disable missing-property
        capStyle: ShapePath.Round

        PathSvg {
            // qmllint disable missing-property
            pathData: root.width > 0 && root.height > 0 ? root.buildPathData(96) : ""
        }
    }
}
