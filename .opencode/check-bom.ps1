$files = @(
  "C:\Users\USER\Desktop\Tanay's Project (Dont Delete)\Zyntrix OS\Zettings app\apps\zettings\tauri.conf.json",
  "C:\Users\USER\Desktop\Tanay's Project (Dont Delete)\Zyntrix OS\Zettings app\apps\zettings\capabilities\zettings-shell.json"
)
foreach ($f in $files) {
  $bytes = [System.IO.File]::ReadAllBytes($f)
  $hex = ($bytes[0..2] | ForEach-Object { '{0:X2}' -f $_ }) -join ' '
  Write-Output "$f -> first3=$hex"
}
