$file = "C:\Users\USER\Desktop\Tanay's Project (Dont Delete)\Zyntrix OS\Zettings app\apps\zettings\tauri.conf.json"
$csp = "default-src 'self'; img-src 'self' data: blob: asset: https://api.zyntrix.os; script-src 'self'; style-src 'self' 'unsafe-inline'"
$json = @"
{"`$schema":"https://schema.tauri.app/config/2","productName":"Zettings","version":"0.1.0","identifier":"os.zyntrix.zettings","build":{"frontendDist":"web/dist","devUrl":"http://localhost:5173","beforeDevCommand":"pnpm -F zettings-web dev","beforeBuildCommand":"pnpm -F zettings-web build"},"app":{"windows":[{"title":"Zettings","width":1280,"height":800,"minWidth":960,"minHeight":600,"decorations":true,"transparent":false,"resizable":true,"fullscreen":false,"center":true}],"security":{"csp":"$csp","capabilities":["zettings-shell"]}},"bundle":{"active":true,"targets":"all","icon":["icons/icon.png"],"category":"Settings","shortDescription":"Zyntrix OS settings","longDescription":"Next-generation system settings application for Zyntrix OS, built with Tauri v2 and React 19."},"plugins":{}}
"@
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($file, $json, $utf8NoBom)
# Verify clean.
(Get-Content $file -Raw) -match '"icon":\["icons/icon.png"\]' | Out-Null
Write-Output "icon path ok: $($matches.Count -gt 0)"
