$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class SynthWindowCapture {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdc, uint flags);

    public static IntPtr FindMainWindow(uint targetPid) {
        IntPtr found = IntPtr.Zero;
        int bestArea = 0;
        EnumWindows((hWnd, lParam) => {
            uint procId;
            GetWindowThreadProcessId(hWnd, out procId);
            if (procId == targetPid && IsWindowVisible(hWnd)) {
                RECT r;
                if (GetWindowRect(hWnd, out r)) {
                    int w = r.Right - r.Left;
                    int h = r.Bottom - r.Top;
                    int area = w * h;
                    if (w > 400 && h > 300 && area > bestArea) {
                        bestArea = area;
                        found = hWnd;
                    }
                }
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
"@

$repoRoot = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $repoRoot 'target\release\synthcraft-app.exe'
if (-not (Test-Path $exe)) {
    throw "Release binary not found at $exe. Build first: cargo build --release -p synthcraft-app"
}
$outDir = Join-Path $repoRoot 'docs\screenshots'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

$shots = @(
    @{ Name = '01_eurorack_studio'; Args = @('--page', 'rack', '--skin', 'obsidian') },
    @{ Name = '02_sequencer_drums'; Args = @('--page', 'sequencer', '--skin', 'obsidian') },
    @{ Name = '03_pipe_organ';      Args = @('--page', 'organ', '--skin', 'gold') },
    @{ Name = '04_daylight_nextgen'; Args = @('--page', 'nextgen', '--skin', 'daylight') }
)

foreach ($s in $shots) {
    $outPng = Join-Path $outDir "$($s.Name).png"
    $proc = Start-Process -FilePath $exe -ArgumentList $s.Args -PassThru
    try {
        $hwnd = [IntPtr]::Zero
        for ($i = 0; $i -lt 40; $i++) {
            Start-Sleep -Milliseconds 250
            $hwnd = [SynthWindowCapture]::FindMainWindow([uint32]$proc.Id)
            if ($hwnd -ne [IntPtr]::Zero) { break }
        }
        if ($hwnd -eq [IntPtr]::Zero) {
            throw "Could not find visible SynthCraft window for $($s.Name)"
        }
        Start-Sleep -Milliseconds 1400
        [SynthWindowCapture]::SetProcessDPIAware() | Out-Null
        $r = New-Object SynthWindowCapture+RECT
        [SynthWindowCapture]::GetWindowRect($hwnd, [ref]$r) | Out-Null
        $w = [Math]::Max(1, $r.Right - $r.Left)
        $h = [Math]::Max(1, $r.Bottom - $r.Top)
        $bmp = New-Object System.Drawing.Bitmap($w, $h)
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $hdc = $g.GetHdc()
        $ok = [SynthWindowCapture]::PrintWindow($hwnd, $hdc, 2)
        if (-not $ok) { [SynthWindowCapture]::PrintWindow($hwnd, $hdc, 0) | Out-Null }
        $g.ReleaseHdc($hdc)
        $bmp.Save($outPng, [System.Drawing.Imaging.ImageFormat]::Png)
        $g.Dispose()
        $bmp.Dispose()
        Write-Host "[ok] Captured $outPng ($w x $h)"
    } finally {
        if ($proc -and -not $proc.HasExited) {
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 300
        }
    }
}
