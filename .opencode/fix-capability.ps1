$file = "C:\Users\USER\Desktop\Tanay's Project (Dont Delete)\Zyntrix OS\Zettings app\apps\zettings\capabilities\zettings-shell.json"
$json = @'
{"$schema":"https://schema.tauri.app/acl/2/capability","identifier":"zettings-shell","description":"Capabilities granted to the Zettings shell webview.","windows":["main"],"permissions":["core:default","core:window:allow-set-title","core:webview:allow-internal-toggle-devtools","shell:allow-open","deep-link:default","window-state:default","log:default","decorum:default"]}
'@
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($file, $json, $utf8NoBom)
(Get-Content $file -Raw).Substring(0, 40)
