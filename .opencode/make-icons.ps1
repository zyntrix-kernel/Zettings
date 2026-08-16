# Build a minimal but valid Windows ICO containing one 32x32 RGBA PNG frame.
# Spec: https://docs.fileformat.com/image/ico/
# We use a 32x32 PNG (which the ICO container supports per the PNG-in-ICO
# extension that Windows Vista+ accepts) and emit proper ICONDIR + ICONDIRENTRY
# headers around the PNG bytes.

Add-Type -AssemblyName System.Drawing
$iconsDir = "C:\Users\USER\Desktop\Tanay's Project (Dont Delete)\Zyntrix OS\Zettings app\apps\zettings\src-tauri\icons"
New-Item -ItemType Directory -Path $iconsDir -Force | Out-Null

# 1. Render a 32x32 PNG into memory.
$bmp = New-Object System.Drawing.Bitmap 32, 32
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = 'AntiAlias'
$g.Clear([System.Drawing.Color]::FromArgb(28, 25, 23))
$br = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(88, 174, 188))
$g.FillEllipse($br, 6, 6, 20, 20)
$g.Dispose(); $br.Dispose()
$ms = New-Object System.IO.MemoryStream
$bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
$png = $ms.ToArray()
$ms.Dispose()

# 2. Build the ICO container around the PNG.
# ICONDIR: reserved(2)=0, type(2)=1, count(2)=1
$icondir = New-Object byte[] 6
$icondir[2] = 1   # type = ICO
$icondir[4] = 1   # count = 1

# ICONDIRENTRY (16 bytes):
#  width(1)=32, height(1)=32, colors(1)=0, reserved(1)=0,
#  planes(2)=1, bpp(2)=32,
#  size(4)=png.Length, offset(4)=22 (right after dir+entry)
$entry = New-Object byte[] 16
$entry[0] = 32        # width  (0 means 256)
$entry[1] = 32        # height
$entry[2] = 0         # colors
$entry[3] = 0         # reserved
$entry[4] = 1; $entry[5] = 0  # planes = 1
$entry[6] = 32; $entry[7] = 0 # bpp = 32
[BitConverter]::GetBytes([uint32]$png.Length).CopyTo($entry, 8)
$entry[12] = 22; $entry[13] = 0; $entry[14] = 0; $entry[15] = 0 # offset = 22

$ico = New-Object System.IO.MemoryStream
$ico.Write($icondir, 0, 6)
$ico.Write($entry, 0, 16)
$ico.Write($png, 0, $png.Length)
$icoBytes = $ico.ToArray()
$ico.Dispose()

$icoPath = Join-Path $iconsDir "icon.ico"
[System.IO.File]::WriteAllBytes($icoPath, $icoBytes)
Get-Item $icoPath | Select-Object Name, Length
