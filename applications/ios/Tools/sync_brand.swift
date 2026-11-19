#!/usr/bin/env swift
// Syncs the app's brand assets from the website's canonical logo and
// regenerates the iOS app icon. Run from the repo root:
//
//   swift applications/ios/Tools/sync_brand.swift
//
// The logo lives in the web assets; the copy under Assets.xcassets exists only
// because an asset catalog needs the file inside it. Re-run this after changing
// the website logo so the two cannot drift.

import AppKit

let source = "applications/axum/server/assets/logo.svg"
let imageset = "applications/ios/Whisper/Assets.xcassets/Logo.imageset"
let iconPath = "applications/ios/Whisper/Assets.xcassets/AppIcon.appiconset/icon-1024.png"

guard let logo = NSImage(contentsOf: URL(fileURLWithPath: source)) else {
    fatalError("could not load \(source)")
}

// MARK: Copy the vector for in-app use

let destination = URL(fileURLWithPath: "\(imageset)/logo.svg")
try? FileManager.default.removeItem(at: destination)
try FileManager.default.copyItem(at: URL(fileURLWithPath: source), to: destination)

try """
{
  "images" : [
    { "filename" : "logo.svg", "idiom" : "universal" }
  ],
  "info" : { "author" : "xcode", "version" : 1 },
  "properties" : {
    "preserves-vector-representation" : true,
    "template-rendering-intent" : "template"
  }
}

""".write(to: URL(fileURLWithPath: "\(imageset)/Contents.json"), atomically: true, encoding: .utf8)

// MARK: Render the app icon

/// The artwork is a rounded tile with padding around it. iOS applies its own
/// corner mask, so the tile has to reach the edges of the icon or it looks
/// inset. Rather than hard-coding that padding, measure it.
func opaqueBounds(of image: NSImage, probe side: CGFloat) -> (CGRect, CGRect) {
    let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil, pixelsWide: Int(side), pixelsHigh: Int(side),
        bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
        colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!

    let scale = min(side / image.size.width, side / image.size.height)
    let drawn = CGRect(
        x: (side - image.size.width * scale) / 2,
        y: (side - image.size.height * scale) / 2,
        width: image.size.width * scale,
        height: image.size.height * scale)

    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    image.draw(in: drawn)
    NSGraphicsContext.restoreGraphicsState()

    var minX = Int(side), minY = Int(side), maxX = -1, maxY = -1
    for y in 0..<Int(side) {
        for x in 0..<Int(side) where (rep.colorAt(x: x, y: y)?.alphaComponent ?? 0) > 0.05 {
            minX = min(minX, x); maxX = max(maxX, x)
            minY = min(minY, y); maxY = max(maxY, y)
        }
    }
    guard maxX >= minX else { fatalError("the logo rendered empty") }

    // `colorAt` is top-down; graphics coordinates are bottom-up.
    let bounds = CGRect(
        x: CGFloat(minX), y: side - CGFloat(maxY + 1),
        width: CGFloat(maxX - minX + 1), height: CGFloat(maxY - minY + 1))
    return (bounds, drawn)
}

let probeSide = 512.0
let (tile, fitted) = opaqueBounds(of: logo, probe: probeSide)

let side = 1024.0
let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: Int(side), pixelsHigh: Int(side),
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!

NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)

// The glyph is a knockout in the tile, so the canvas colour shows through it.
// White keeps the mark reading exactly as it does on the site in light mode.
NSColor.white.set()
NSRect(x: 0, y: 0, width: side, height: side).fill()

// Scale the artwork so its opaque tile fills the canvas exactly.
let width = side * (fitted.width / tile.width)
let height = side * (fitted.height / tile.height)
logo.draw(
    in: NSRect(
        x: -(tile.minX - fitted.minX) / fitted.width * width,
        y: -(tile.minY - fitted.minY) / fitted.height * height,
        width: width, height: height))

NSGraphicsContext.restoreGraphicsState()

try rep.representation(using: .png, properties: [:])!
    .write(to: URL(fileURLWithPath: iconPath))

print("synced logo.svg -> \(imageset)")
print("rendered \(iconPath) (1024x1024) from the website logo")
