# Regenerates every Reelwright app-icon file (assets/app-icon/) from the drawing below. Original artwork, drawn in code
# with System.Drawing only (Windows PowerShell; no SVG renderer needed):
#   powershell -ExecutionPolicy Bypass -File packaging/icons.ps1
param([string]$Repo = (Split-Path $PSScriptRoot -Parent), [string]$Slug = 'reelwright', [string]$Title = 'Reelwright', [string]$Motif = 'reel')
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$root = Join-Path $Repo 'assets\app-icon'
$id = "online.jonmcgee.$Slug"
$pal = @{
  spline = @{ ink = '#0b1d2a'; end = '#12a89a'; paper = '#f2efe6'; spark = '#ff6b4a' }
  reel   = @{ ink = '#1b0f1f'; end = '#d6264a'; paper = '#f4efe6'; spark = '#ffd23f' }
  lumen  = @{ ink = '#241b3d'; end = '#ff9f1c'; paper = '#fff6e0'; spark = '#ffffff' }
  folio  = @{ ink = '#14213d'; end = '#e4572e'; paper = '#f6f1e7'; spark = '#ffc857' }
  tween  = @{ ink = '#140f2d'; end = '#7c3aed'; paper = '#f1ecff'; spark = '#34d399' }
  spread = @{ ink = '#1a1127'; end = '#d6336c'; paper = '#f7f0ea'; spark = '#4dd4c4' }
}[$Motif]
$ink = $pal.ink; $end = $pal.end; $paper = $pal.paper; $spark = $pal.spark

# --- shapes in a 512-unit square: type, geometry, fill / stroke ---------------------------------
function Bez($p0, $c1, $c2, $p3, $t) { $u = 1 - $t; @( ($u*$u*$u*$p0[0] + 3*$u*$u*$t*$c1[0] + 3*$u*$t*$t*$c2[0] + $t*$t*$t*$p3[0]), ($u*$u*$u*$p0[1] + 3*$u*$u*$t*$c1[1] + 3*$u*$t*$t*$c2[1] + $t*$t*$t*$p3[1]) ) }
$Shapes = New-Object System.Collections.Generic.List[object]
switch ($Motif) {
  'spline' {
    $p0 = @(112, 372); $c1 = @(112, 120); $c2 = @(400, 392); $p3 = @(400, 140)
    $Shapes.Add(@{ t='bez'; p=@($p0,$c1,$c2,$p3); w=26; c=$paper })
    $Shapes.Add(@{ t='line'; x1=$p0[0]; y1=$p0[1]; x2=$c1[0]; y2=$c1[1]; w=7; c=$paper; a=0.55 })
    $Shapes.Add(@{ t='line'; x1=$p3[0]; y1=$p3[1]; x2=$c2[0]; y2=$c2[1]; w=7; c=$paper; a=0.55 })
    $Shapes.Add(@{ t='circle'; cx=$c1[0]; cy=$c1[1]; r=15; c=$paper })
    $Shapes.Add(@{ t='circle'; cx=$c2[0]; cy=$c2[1]; r=15; c=$paper })
    $Shapes.Add(@{ t='rrect'; x=$p0[0]-24; y=$p0[1]-24; w=48; h=48; r=8; c=$spark })
    $Shapes.Add(@{ t='rrect'; x=$p3[0]-24; y=$p3[1]-24; w=48; h=48; r=8; c=$spark })
  }
  'reel' {
    $Shapes.Add(@{ t='circle'; cx=256; cy=256; r=168; c=$paper })
    foreach ($i in 0..4) { $a = ($i * 72 - 90) * [Math]::PI / 180; $Shapes.Add(@{ t='circle'; cx=256 + 96 * [Math]::Cos($a); cy=256 + 96 * [Math]::Sin($a); r=40; c=$ink }) }
    $Shapes.Add(@{ t='circle'; cx=256; cy=256; r=30; c=$ink })
    $Shapes.Add(@{ t='circle'; cx=256; cy=256; r=16; c=$spark })
    $Shapes.Add(@{ t='rrect'; x=392; y=370; w=70; h=26; r=6; c=$paper })
  }
  'lumen' {
    foreach ($i in 0..11) { $a = $i * 30 * [Math]::PI / 180; $in = 126; $out = if ($i % 2 -eq 0) { 188 } else { 166 }
      $Shapes.Add(@{ t='line'; x1=256 + $in * [Math]::Cos($a); y1=256 + $in * [Math]::Sin($a); x2=256 + $out * [Math]::Cos($a); y2=256 + $out * [Math]::Sin($a); w=26; c=$paper }) }
    $Shapes.Add(@{ t='circle'; cx=256; cy=256; r=92; c=$paper })
    $Shapes.Add(@{ t='circle'; cx=256; cy=256; r=44; c=$spark })
  }
  'folio' {
    $Shapes.Add(@{ t='poly'; p=@(@(158,100),@(300,100),@(366,166),@(366,412),@(158,412)); c=$paper; round=14 })
    $Shapes.Add(@{ t='poly'; p=@(@(300,100),@(366,166),@(300,166)); c=$spark; round=4 })
    foreach ($row in @(@(222,160),@(268,160),@(314,134),@(360,160))) { $Shapes.Add(@{ t='rrect'; x=190; y=$row[0]; w=$row[1]; h=20; r=10; c=$ink }) }
  }
  'tween' {
    $p0 = @(132, 346); $c1 = @(150, 110); $c2 = @(360, 90); $p3 = @(380, 170)
    $Shapes.Add(@{ t='bez'; p=@($p0,$c1,$c2,$p3); w=10; c=$paper; a=0.5 })
    foreach ($t in 0.28, 0.5, 0.72) { $pt = Bez $p0 $c1 $c2 $p3 $t; $Shapes.Add(@{ t='circle'; cx=$pt[0]; cy=$pt[1]; r=17; c=$paper }) }
    $Shapes.Add(@{ t='poly'; p=@(@(132,296),@(182,346),@(132,396),@(82,346)); c=$paper; round=6 })
    $Shapes.Add(@{ t='poly'; p=@(@(380,118),@(430,168),@(380,218),@(330,168)); c=$spark; round=6 })
    $Shapes.Add(@{ t='rrect'; x=70; y=420; w=372; h=14; r=7; c=$paper; a=0.35 })
  }
  'spread' {
    $Shapes.Add(@{ t='rrect'; x=70; y=118; w=178; h=276; r=14; c=$paper })
    $Shapes.Add(@{ t='rrect'; x=264; y=118; w=178; h=276; r=14; c=$paper })
    $Shapes.Add(@{ t='rrect'; x=96; y=146; w=126; h=104; r=8; c=$spark })
    foreach ($y in 276, 306, 336) { $Shapes.Add(@{ t='rrect'; x=96; y=$y; w=$(if ($y -eq 336) { 84 } else { 126 }); h=14; r=7; c=$ink }) }
    foreach ($x in 290, 366) { foreach ($y in 146, 176, 206, 236, 266, 296, 326) { $Shapes.Add(@{ t='rrect'; x=$x; y=$y; w=58; h=14; r=7; c=$ink }) } }
  }
}

# --- SVG ---------------------------------------------------------------------------------------
function F([double]$v) { [string]::Format([Globalization.CultureInfo]::InvariantCulture, '{0:0.##}', $v) }
$svg = New-Object System.Text.StringBuilder
[void]$svg.AppendLine('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">')
[void]$svg.AppendLine("  <title>$Title</title>")
[void]$svg.AppendLine("  <defs><linearGradient id=`"bg`" x1=`"0`" y1=`"0`" x2=`"1`" y2=`"1`"><stop offset=`"0`" stop-color=`"$ink`"/><stop offset=`"1`" stop-color=`"$end`"/></linearGradient>")
[void]$svg.AppendLine('  <linearGradient id="sheen" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0.16"/><stop offset="0.55" stop-color="#fff" stop-opacity="0"/></linearGradient></defs>')
[void]$svg.AppendLine('  <rect width="512" height="512" rx="112" fill="url(#bg)"/>')
[void]$svg.AppendLine('  <rect width="512" height="512" rx="112" fill="url(#sheen)"/>')
foreach ($s in $Shapes) {
  $op = if ($s.a) { " opacity=`"$(F $s.a)`"" } else { '' }
  switch ($s.t) {
    'rrect'  { [void]$svg.AppendLine("  <rect x=`"$(F $s.x)`" y=`"$(F $s.y)`" width=`"$(F $s.w)`" height=`"$(F $s.h)`" rx=`"$(F $s.r)`" fill=`"$($s.c)`"$op/>") }
    'circle' { [void]$svg.AppendLine("  <circle cx=`"$(F $s.cx)`" cy=`"$(F $s.cy)`" r=`"$(F $s.r)`" fill=`"$($s.c)`"$op/>") }
    'line'   { [void]$svg.AppendLine("  <line x1=`"$(F $s.x1)`" y1=`"$(F $s.y1)`" x2=`"$(F $s.x2)`" y2=`"$(F $s.y2)`" stroke=`"$($s.c)`" stroke-width=`"$(F $s.w)`" stroke-linecap=`"round`"$op/>") }
    'bez'    { $q = $s.p; [void]$svg.AppendLine("  <path d=`"M$(F $q[0][0]) $(F $q[0][1]) C$(F $q[1][0]) $(F $q[1][1]) $(F $q[2][0]) $(F $q[2][1]) $(F $q[3][0]) $(F $q[3][1])`" fill=`"none`" stroke=`"$($s.c)`" stroke-width=`"$(F $s.w)`" stroke-linecap=`"round`"$op/>") }
    'poly'   { $pts = ($s.p | ForEach-Object { "$(F $_[0]),$(F $_[1])" }) -join ' '; [void]$svg.AppendLine("  <polygon points=`"$pts`" fill=`"$($s.c)`" stroke=`"$($s.c)`" stroke-width=`"$(F ($s.round * 2))`" stroke-linejoin=`"round`"$op/>") }
  }
}
[void]$svg.AppendLine('</svg>')
$svgText = $svg.ToString()
foreach ($p in @("$Slug.svg", "$Slug-small.svg", "hicolor\scalable\apps\$id.svg")) {
  $dest = Join-Path $root $p; New-Item -ItemType Directory -Force (Split-Path $dest) | Out-Null
  [IO.File]::WriteAllText($dest, $svgText, [Text.UTF8Encoding]::new($false))
}

# --- raster renders -------------------------------------------------------------------------
function New-RoundedPath([single]$x, [single]$y, [single]$w, [single]$h, [single]$r) {
  $p = New-Object System.Drawing.Drawing2D.GraphicsPath; $d = 2 * $r
  $p.AddArc($x, $y, $d, $d, 180, 90); $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
  $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90); $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90); $p.CloseFigure(); $p
}
function Hex([string]$h, [int]$a = 255) { [System.Drawing.Color]::FromArgb($a, [Convert]::ToInt32($h.Substring(1, 2), 16), [Convert]::ToInt32($h.Substring(3, 2), 16), [Convert]::ToInt32($h.Substring(5, 2), 16)) }

function Render-Icon([int]$canvas, [int]$tile) {
  $bmp = New-Object System.Drawing.Bitmap $canvas, $canvas, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = 'AntiAlias'; $g.PixelOffsetMode = 'HighQuality'; $g.InterpolationMode = 'HighQualityBicubic'; $g.Clear([System.Drawing.Color]::Transparent)
  $off = ($canvas - $tile) / 2.0; $k = $tile / 512.0
  $rect = New-Object System.Drawing.RectangleF $off, $off, $tile, $tile
  $path = New-RoundedPath $off $off $tile $tile (112 * $k)
  $bg = New-Object System.Drawing.Drawing2D.LinearGradientBrush $rect, (Hex $ink), (Hex $end), 45.0
  $g.FillPath($bg, $path)
  $sheen = New-Object System.Drawing.Drawing2D.LinearGradientBrush $rect, (Hex '#ffffff' 41), (Hex '#ffffff' 0), 90.0
  $g.FillPath($sheen, $path)
  foreach ($s in $Shapes) {
    $alpha = if ($s.a) { [int](255 * $s.a) } else { 255 }
    $col = Hex $s.c $alpha
    switch ($s.t) {
      'rrect'  { $p = New-RoundedPath ($off + $s.x * $k) ($off + $s.y * $k) ($s.w * $k) ($s.h * $k) ($s.r * $k); $b = New-Object System.Drawing.SolidBrush $col; $g.FillPath($b, $p); $b.Dispose(); $p.Dispose() }
      'circle' { $b = New-Object System.Drawing.SolidBrush $col; $g.FillEllipse($b, [single]($off + ($s.cx - $s.r) * $k), [single]($off + ($s.cy - $s.r) * $k), [single](2 * $s.r * $k), [single](2 * $s.r * $k)); $b.Dispose() }
      'line'   { $pen = New-Object System.Drawing.Pen $col, ([single]($s.w * $k)); $pen.StartCap = 'Round'; $pen.EndCap = 'Round'
                 $g.DrawLine($pen, [single]($off + $s.x1 * $k), [single]($off + $s.y1 * $k), [single]($off + $s.x2 * $k), [single]($off + $s.y2 * $k)); $pen.Dispose() }
      'bez'    { $q = $s.p; $pen = New-Object System.Drawing.Pen $col, ([single]($s.w * $k)); $pen.StartCap = 'Round'; $pen.EndCap = 'Round'
                 $g.DrawBezier($pen, [single]($off + $q[0][0] * $k), [single]($off + $q[0][1] * $k), [single]($off + $q[1][0] * $k), [single]($off + $q[1][1] * $k), [single]($off + $q[2][0] * $k), [single]($off + $q[2][1] * $k), [single]($off + $q[3][0] * $k), [single]($off + $q[3][1] * $k)); $pen.Dispose() }
      'poly'   { $pts = [System.Drawing.PointF[]]($s.p | ForEach-Object { New-Object System.Drawing.PointF ([single]($off + $_[0] * $k)), ([single]($off + $_[1] * $k)) })
                 $b = New-Object System.Drawing.SolidBrush $col; $g.FillPolygon($b, $pts)
                 $pen = New-Object System.Drawing.Pen $col, ([single]($s.round * 2 * $k)); $pen.LineJoin = 'Round'; $g.DrawPolygon($pen, $pts); $b.Dispose(); $pen.Dispose() }
    }
  }
  $bg.Dispose(); $sheen.Dispose(); $path.Dispose(); $g.Dispose(); $bmp
}
function Png-Bytes($bmp) { $ms = New-Object System.IO.MemoryStream; $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png); $b = $ms.ToArray(); $ms.Dispose(); $b }
function Render-Bytes([int]$canvas, [int]$tile) { $b = Render-Icon $canvas $tile; $x = Png-Bytes $b; $b.Dispose(); , $x }

foreach ($n in 16, 24, 32, 48, 64, 128, 256, 512) {
  $dir = Join-Path $root "hicolor\${n}x${n}\apps"; New-Item -ItemType Directory -Force $dir | Out-Null
  [IO.File]::WriteAllBytes((Join-Path $dir "$id.png"), (Render-Bytes $n $n))
}
$mac = @{}; foreach ($n in 64, 128, 256, 512, 1024) { $mac[$n] = Render-Bytes $n ([int][Math]::Round($n * 824 / 1024)) }
[IO.File]::WriteAllBytes((Join-Path $root "$Slug-1024.png"), $mac[1024])
[IO.File]::WriteAllBytes((Join-Path $root "$Slug-macos-512.png"), $mac[512])

# --- .ico (PNG-compressed entries) ----------------------------------------------------------
$sizes = 16, 24, 32, 48, 64, 128, 256
$blobs = New-Object System.Collections.Generic.List[byte[]]; foreach ($n in $sizes) { $blobs.Add([byte[]](Render-Bytes $n $n)) }
$ms = New-Object System.IO.MemoryStream; $w = New-Object System.IO.BinaryWriter $ms
$w.Write([uint16]0); $w.Write([uint16]1); $w.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
  $n = $sizes[$i]; $w.Write([byte]$(if ($n -ge 256) { 0 } else { $n })); $w.Write([byte]$(if ($n -ge 256) { 0 } else { $n }))
  $w.Write([byte]0); $w.Write([byte]0); $w.Write([uint16]1); $w.Write([uint16]32); $w.Write([uint32]$blobs[$i].Length); $w.Write([uint32]$offset); $offset += $blobs[$i].Length }
foreach ($b in $blobs) { $w.Write([byte[]]$b) }
$w.Flush(); [IO.File]::WriteAllBytes((Join-Path $root "$Slug.ico"), $ms.ToArray()); $w.Dispose()

# --- .icns (PNG chunks; big-endian lengths) -------------------------------------------------
function BE([int]$v) { [byte[]]@((($v -shr 24) -band 255), (($v -shr 16) -band 255), (($v -shr 8) -band 255), ($v -band 255)) }
$chunks = @( @('ic07', 128), @('ic08', 256), @('ic09', 512), @('ic10', 1024), @('ic11', 64), @('ic12', 128), @('ic13', 256), @('ic14', 512) )
$body = New-Object System.IO.MemoryStream
foreach ($c in $chunks) { $data = $mac[[int]$c[1]]; $t = [Text.Encoding]::ASCII.GetBytes($c[0]); $body.Write($t, 0, 4); $l = BE ($data.Length + 8); $body.Write($l, 0, 4); $body.Write($data, 0, $data.Length) }
$icns = New-Object System.IO.MemoryStream; $h = [Text.Encoding]::ASCII.GetBytes('icns'); $icns.Write($h, 0, 4); $l = BE ($body.Length + 8); $icns.Write($l, 0, 4)
$bb = $body.ToArray(); $icns.Write($bb, 0, $bb.Length); [IO.File]::WriteAllBytes((Join-Path $root "$Slug.icns"), $icns.ToArray())

# --- README + LICENSE for the icon folder -----------------------------------------------------
$desc = @{ spline = 'a smooth Bezier curve with its two anchor points and handles'; reel = 'a film reel with five sprocket holes'; lumen = 'a radiant light with twelve rays'
           folio = 'a page with a folded corner and lines of text'; tween = 'two keyframe diamonds joined by a motion path with in-between dots'; spread = 'two facing pages of a layout spread' }[$Motif]
$readme = @"
# $Title app icon

$Title's icon is $($desc): a flat figure on a rounded tile with a diagonal gradient. It is original artwork
made for $Title, drawn in code by ``packaging/icons.ps1``; it replaces the upstream project's icon, which
is not used anywhere in this repository.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| Ink | ``$ink`` | the tile gradient, top-left |
| $Title colour | ``$end`` | the tile gradient, bottom-right |
| Paper | ``$paper`` | the figure |
| Spark | ``$spark`` | the accent |

## Files

- ``$Slug.svg``, ``$Slug-small.svg``: the vector master (identical; the second name is kept for packaging scripts that expect it), also the hicolor scalable icon.
- ``$Slug-1024.png``: 1024 px render on the macOS grid; ``$Slug-macos-512.png``: the runtime Dock icon.
- ``$Slug.icns``: macOS bundle icon; ``$Slug.ico``: Windows icon, 16-256 px.
- ``hicolor/<size>/apps/$id.png`` (16-512) and ``hicolor/scalable/...svg``: Linux icon theme.

## Regenerate

``powershell -ExecutionPolicy Bypass -File packaging/icons.ps1`` (Windows; uses System.Drawing only).
"@
[IO.File]::WriteAllText((Join-Path $root 'README.md'), $readme, [Text.UTF8Encoding]::new($false))
$lic = @"
Original artwork made for $Title by Jon McGee, drawn in code (packaging/icons.ps1).
Licensed under the same terms as the rest of the project: MIT OR Apache-2.0 (see LICENSE-MIT and LICENSE-APACHE).
"@
[IO.File]::WriteAllText((Join-Path $root 'LICENSE.txt'), $lic, [Text.UTF8Encoding]::new($false))
'icon set written to ' + $root
