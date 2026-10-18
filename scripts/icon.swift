import AppKit
import Foundation
let output = URL(fileURLWithPath: CommandLine.arguments[1]).appendingPathComponent("Termcomplete.iconset")
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
for base in [16, 32, 128, 256, 512] {
    for scale in [1, 2] {
        let n = base * scale
        let image = NSImage(size: NSSize(width: CGFloat(n), height: CGFloat(n)))
        image.lockFocus()
        NSColor(calibratedRed: 0.09, green: 0.15, blue: 0.22, alpha: 1).setFill()
        NSBezierPath(roundedRect: NSRect(x: 0, y: 0, width: CGFloat(n), height: CGFloat(n)), xRadius: CGFloat(n) * 0.22, yRadius: CGFloat(n) * 0.22).fill()
        let attributes: [NSAttributedString.Key: Any] = [.font: NSFont.monospacedSystemFont(ofSize: CGFloat(n) * 0.55, weight: .medium), .foregroundColor: NSColor(calibratedRed: 0.60, green: 0.91, blue: 0.80, alpha: 1)]
        (">_" as NSString).draw(at: NSPoint(x: CGFloat(n) * 0.17, y: CGFloat(n) * 0.22), withAttributes: attributes)
        image.unlockFocus()
        let bitmap = NSBitmapImageRep(data: image.tiffRepresentation!)!
        let filename = "icon_\(base)x\(base)\(scale == 2 ? "@2x" : "").png"
        try bitmap.representation(using: .png, properties: [:])!.write(to: output.appendingPathComponent(filename))
    }
}
