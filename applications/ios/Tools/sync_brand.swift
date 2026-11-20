#!/usr/bin/env swift
// Draws Whisper's mark from geometry and writes every brand asset from it, so
// the website and the app cannot drift:
//
//   applications/axum/server/assets/logo.svg                     (site + favicon)
//   applications/ios/Whisper/Assets.xcassets/Mark.imageset/mark.svg (in-app mark)
//   applications/ios/Whisper/Assets.xcassets/AppIcon.appiconset/icon-1024.png
//
// Run from the repo root (style: aurora | obsidian | ivory, default aurora):
//
//   xcrun swift applications/ios/Tools/sync_brand.swift [style]

import AppKit

// MARK: - The mark

/// One continuous ribbon on a 1024 grid, y pointing down — the same shape the
/// original hand-traced logo had, redrawn with true straight lines, one stroke
/// width and one bend radius so it stays crisp at every size.
func markCenterline() -> CGPath {
    let r: CGFloat = 40  // centreline radius of every bend
    let p = CGMutablePath()
    func pt(_ x: CGFloat, _ y: CGFloat) -> CGPoint { CGPoint(x: x, y: y) }

    // Top room: the stub that stops short of the corridor, then a notch.
    p.move(to: pt(510, 420))
    p.addArc(tangent1End: pt(510, 165), tangent2End: pt(380, 165), radius: r)
    p.addArc(tangent1End: pt(380, 165), tangent2End: pt(380, 242.5), radius: r)
    p.addLine(to: pt(380, 242.5))
    p.addArc(center: pt(317.5, 242.5), radius: 62.5, startAngle: 0, endAngle: .pi, clockwise: false)
    p.addArc(tangent1End: pt(255, 165), tangent2End: pt(130, 165), radius: r)
    p.addArc(tangent1End: pt(130, 165), tangent2End: pt(130, 545), radius: r)
    // The corridor, then the bottom room with its mirrored notch.
    p.addArc(tangent1End: pt(130, 545), tangent2End: pt(886, 545), radius: r)
    p.addArc(tangent1End: pt(886, 545), tangent2End: pt(886, 850), radius: r)
    p.addArc(tangent1End: pt(886, 850), tangent2End: pt(762.5, 850), radius: r)
    p.addArc(tangent1End: pt(762.5, 850), tangent2End: pt(762.5, 776.75), radius: r)
    p.addLine(to: pt(762.5, 776.75))
    p.addArc(center: pt(698.75, 776.75), radius: 63.75, startAngle: 0, endAngle: -.pi, clockwise: true)
    p.addArc(tangent1End: pt(635, 850), tangent2End: pt(510, 850), radius: r)
    p.addArc(tangent1End: pt(510, 850), tangent2End: pt(510, 545), radius: r)
    p.addLine(to: pt(510, 545))
    return p
}

/// The ribbon as a filled outline, centred on a 1024 canvas at `scale`.
func mark(scale: CGFloat) -> CGPath {
    let outline = markCenterline()
        .copy(strokingWithWidth: 70, lineCap: .round, lineJoin: .round, miterLimit: 10)
        .normalized()
    let box = outline.boundingBoxOfPath
    var t = CGAffineTransform(translationX: 512, y: 512)
        .scaledBy(x: scale, y: scale)
        .translatedBy(x: -box.midX, y: -box.midY)
    return outline.copy(using: &t)!
}

// MARK: - SVG (website + in-app template)

func svgPathData(_ path: CGPath) -> String {
    var d = ""
    func f(_ p: CGPoint) -> String { String(format: "%.2f %.2f", p.x, p.y) }
    path.applyWithBlock { element in
        let e = element.pointee
        switch e.type {
        case .moveToPoint: d += "M\(f(e.points[0]))"
        case .addLineToPoint: d += "L\(f(e.points[0]))"
        case .addQuadCurveToPoint: d += "Q\(f(e.points[0])) \(f(e.points[1]))"
        case .addCurveToPoint: d += "C\(f(e.points[0])) \(f(e.points[1])) \(f(e.points[2]))"
        case .closeSubpath: d += "Z"
        @unknown default: break
        }
    }
    return d
}

/// The website's logo: a black tile with the mark knocked out, so the site's
/// dark-mode `invert(1)` sees through the glyph.
func writeLogoSVG(to path: String) throws {
    let tile = CGPath(
        roundedRect: CGRect(x: 0, y: 0, width: 1024, height: 1024),
        cornerWidth: 230, cornerHeight: 230, transform: nil)
    let knockout = tile.subtracting(mark(scale: 0.78))
    try """
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">
    <path fill="#000000" d="\(svgPathData(knockout))"/>
    </svg>

    """.write(toFile: path, atomically: true, encoding: .utf8)
}

/// The bare mark, cropped to its bounds, for the app to tint as a template.
func writeMarkImageset(at dir: String) throws {
    let glyph = mark(scale: 1)
    let box = glyph.boundingBoxOfPath.integral
    try FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
    try """
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="\(Int(box.minX)) \(Int(box.minY)) \(Int(box.width)) \(Int(box.height))" width="\(Int(box.width))" height="\(Int(box.height))">
    <path fill="#000000" d="\(svgPathData(glyph))"/>
    </svg>

    """.write(toFile: "\(dir)/mark.svg", atomically: true, encoding: .utf8)
    try """
    {
      "images" : [ { "filename" : "mark.svg", "idiom" : "universal" } ],
      "info" : { "author" : "xcode", "version" : 1 },
      "properties" : {
        "preserves-vector-representation" : true,
        "template-rendering-intent" : "template"
      }
    }

    """.write(toFile: "\(dir)/Contents.json", atomically: true, encoding: .utf8)
}

// MARK: - App icon

struct Style {
    var background: [NSColor]  // top → bottom
    var bloom: NSColor  // soft light behind the mark
    var mark: [NSColor]  // top-left → bottom-right
    var glow: NSColor
    var glowBlur: CGFloat
}

func hex(_ v: UInt32, _ a: CGFloat = 1) -> NSColor {
    NSColor(
        srgbRed: CGFloat((v >> 16) & 0xFF) / 255, green: CGFloat((v >> 8) & 0xFF) / 255,
        blue: CGFloat(v & 0xFF) / 255, alpha: a)
}

let styles: [String: Style] = [
    // Violet-to-cyan ribbon on midnight — matches the app's accent.
    "aurora": Style(
        background: [hex(0x1C1640), hex(0x06060C)], bloom: hex(0x7C5CFF, 0.55),
        mark: [hex(0xFFFFFF), hex(0xC4B5FD), hex(0x5EEAD4)], glow: hex(0x8B5CF6, 0.9), glowBlur: 70),
    // Monochrome, like the website: silver on graphite.
    "obsidian": Style(
        background: [hex(0x34343E), hex(0x07070A)], bloom: hex(0xFFFFFF, 0.16),
        mark: [hex(0xFFFFFF), hex(0xB9BDC9)], glow: hex(0xFFFFFF, 0.45), glowBlur: 60),
    // Light: ink on porcelain.
    "ivory": Style(
        background: [hex(0xFFFFFF), hex(0xE2E3EA)], bloom: hex(0xFFFFFF, 0.9),
        mark: [hex(0x34343C), hex(0x050507)], glow: hex(0x000000, 0.28), glowBlur: 36),
]

func gradient(_ colors: [NSColor]) -> CGGradient {
    CGGradient(
        colorsSpace: CGColorSpace(name: CGColorSpace.sRGB),
        colors: colors.map(\.cgColor) as CFArray, locations: nil)!
}

func renderIcon(_ style: Style, to path: String) throws {
    let side = 1024
    let ctx = CGContext(
        data: nil, width: side, height: side, bitsPerComponent: 8, bytesPerRow: 0,
        space: CGColorSpace(name: CGColorSpace.sRGB)!,
        bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue)!
    let full = CGRect(x: 0, y: 0, width: side, height: side)

    // CoreGraphics is y-up; the mark was drawn y-down.
    var flip = CGAffineTransform(translationX: 0, y: 1024).scaledBy(x: 1, y: -1)
    let glyph = mark(scale: 0.70).copy(using: &flip)!
    let box = glyph.boundingBoxOfPath

    // Background: vertical gradient, then a bloom of light behind the mark.
    ctx.drawLinearGradient(
        gradient(style.background), start: CGPoint(x: 512, y: 1024), end: CGPoint(x: 512, y: 0),
        options: [])
    ctx.drawRadialGradient(
        gradient([style.bloom, style.bloom.withAlphaComponent(0)]),
        startCenter: CGPoint(x: 512, y: 560), startRadius: 0,
        endCenter: CGPoint(x: 512, y: 560), endRadius: 620, options: [])

    // Glow: the mark's own shadow, unoffset.
    ctx.saveGState()
    ctx.setShadow(offset: .zero, blur: style.glowBlur, color: style.glow.cgColor)
    ctx.addPath(glyph)
    ctx.setFillColor(style.mark[0].cgColor)
    ctx.fillPath()
    ctx.restoreGState()

    // The mark: diagonal gradient, then a gloss over its upper half.
    ctx.saveGState()
    ctx.addPath(glyph)
    ctx.clip()
    ctx.drawLinearGradient(
        gradient(style.mark), start: CGPoint(x: box.minX, y: box.maxY),
        end: CGPoint(x: box.maxX, y: box.minY), options: [])
    ctx.drawLinearGradient(
        gradient([hex(0xFFFFFF, 0.35), hex(0xFFFFFF, 0)]), start: CGPoint(x: 512, y: box.maxY),
        end: CGPoint(x: 512, y: box.midY), options: [])
    ctx.restoreGState()

    // A faint sheen across the top of the tile, like glass catching light.
    ctx.saveGState()
    ctx.clip(to: full)
    ctx.drawRadialGradient(
        gradient([hex(0xFFFFFF, 0.10), hex(0xFFFFFF, 0)]),
        startCenter: CGPoint(x: 512, y: 1200), startRadius: 0,
        endCenter: CGPoint(x: 512, y: 1200), endRadius: 700, options: [])
    ctx.restoreGState()

    let rep = NSBitmapImageRep(cgImage: ctx.makeImage()!)
    try rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: path))
}

// MARK: - Main

let styleName = CommandLine.arguments.dropFirst().first ?? "aurora"
guard let style = styles[styleName] else {
    fatalError("unknown style \(styleName); pick one of \(styles.keys.sorted())")
}

let assets = "applications/ios/Whisper/Assets.xcassets"
let iconPath = "\(assets)/AppIcon.appiconset/icon-1024.png"

try writeLogoSVG(to: "applications/axum/server/assets/logo.svg")
try writeMarkImageset(at: "\(assets)/Mark.imageset")
try renderIcon(style, to: iconPath)

print("wrote the site logo, the in-app mark and \(iconPath) in the \(styleName) style")
