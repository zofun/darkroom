param(
  [string]$Exe = "$PSScriptRoot\..\src-tauri\target\release\app.exe",
  [string]$PhotoDir = "$PSScriptRoot\..\.demo\West-Lake",
  [string]$OutDir = "$PSScriptRoot\..\docs\screenshots"
)

Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32 {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hWnd, out RECT lpRect);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr hWnd, ref POINT lpPoint);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr value);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
  public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lp);
  public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
  public struct POINT { public int X; public int Y; }

  public static IntPtr FindMainWindow(uint targetPid) {
    IntPtr best = IntPtr.Zero;
    long bestArea = 0;
    EnumWindows((h, l) => {
      uint pid;
      GetWindowThreadProcessId(h, out pid);
      if (pid != targetPid || !IsWindowVisible(h)) return true;
      RECT r;
      if (!GetWindowRect(h, out r)) return true;
      long w = r.Right - r.Left;
      long hh = r.Bottom - r.Top;
      if (w > 200 && hh > 200 && w * hh > bestArea) { bestArea = w * hh; best = h; }
      return true;
    }, IntPtr.Zero);
    return best;
  }
}
"@

[Win32]::SetProcessDpiAwarenessContext([IntPtr]::new(-4)) | Out-Null
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Get-Process -Name app -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

$info = Get-Content "$PSScriptRoot\demo-info.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$dateText = $info.date
$placeText = $info.place
$resolvedPhotoDir = (Resolve-Path $PhotoDir).Path

function Capture-Screen {
  param(
    [string]$Name,
    [hashtable]$Env,
    [int]$WaitSeconds = 12
  )
  Get-Process -Name app -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Milliseconds 500
  foreach ($k in $Env.Keys) { Set-Item -Path "env:$k" -Value $Env[$k] }
  $proc = Start-Process -FilePath $Exe -PassThru
  $hwnd = [IntPtr]::Zero
  for ($i = 0; $i -lt 100; $i++) {
    Start-Sleep -Milliseconds 250
    $hwnd = [Win32]::FindMainWindow([uint32]$proc.Id)
    if ($hwnd -ne [IntPtr]::Zero) { break }
  }
  if ($hwnd -eq [IntPtr]::Zero) {
    Write-Output "window not found for $Name"
    return
  }
  [Win32]::ShowWindow($hwnd, 9) | Out-Null
  [Win32]::SetForegroundWindow($hwnd) | Out-Null
  Start-Sleep -Seconds $WaitSeconds

  $rect = [Win32+RECT]::new()
  [Win32]::GetClientRect($hwnd, [ref]$rect) | Out-Null
  $origin = [Win32+POINT]::new()
  $origin.X = 0
  $origin.Y = 0
  [Win32]::ClientToScreen($hwnd, [ref]$origin) | Out-Null
  $w = $rect.Right - $rect.Left
  $h = $rect.Bottom - $rect.Top
  if ($w -le 0 -or $h -le 0) {
    Write-Output "invalid client rect for $Name"
    return
  }
  $bmp = New-Object System.Drawing.Bitmap($w, $h)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($origin.X, $origin.Y, 0, 0, $bmp.Size)
  $path = Join-Path $OutDir "$Name.png"
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose()
  $bmp.Dispose()
  Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
  Write-Output "saved $path ($w x $h)"
  foreach ($k in $Env.Keys) { Remove-Item "env:$k" -ErrorAction SilentlyContinue }
}

$base = @{
  DARKROOM_DEMO_DIR   = $resolvedPhotoDir
  DARKROOM_DEMO_DATE  = $dateText
  DARKROOM_DEMO_PLACE = $placeText
}

$main = $base.Clone()
$main["DARKROOM_DEMO_STYLE"] = "footerband"
$main["DARKROOM_DEMO_VIEW"] = "result"
Capture-Screen -Name "main" -Env $main -WaitSeconds 12

$crop = $base.Clone()
$crop["DARKROOM_DEMO_STYLE"] = "cornernote"
$crop["DARKROOM_DEMO_VIEW"] = "crop"
Capture-Screen -Name "crop" -Env $crop -WaitSeconds 14

$readable = $base.Clone()
$readable["DARKROOM_DEMO_STYLE"] = "engraved"
$readable["DARKROOM_DEMO_READABILITY"] = "plate"
$readable["DARKROOM_DEMO_VIEW"] = "result"
Capture-Screen -Name "readability" -Env $readable -WaitSeconds 12
