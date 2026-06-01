$ErrorActionPreference = "Stop"

$limit = 1474560
$exe = Join-Path $PSScriptRoot "target\release\win32-pixel-probe.exe"

$cases = @(
    @{
        Name = "core_pixel_sound_no_capacity_assets"
        Args = @("build", "--release", "--no-default-features", "--features", "pixel_tile,pixel_sound")
    },
    @{
        Name = "asset_tiles"
        Args = @("build", "--release", "--no-default-features", "--features", "pixel_sound,asset_tiles")
    },
    @{
        Name = "asset_sprites"
        Args = @("build", "--release", "--no-default-features", "--features", "pixel_sound,asset_sprites")
    },
    @{
        Name = "asset_maps"
        Args = @("build", "--release", "--no-default-features", "--features", "pixel_tile,pixel_sound,asset_maps")
    },
    @{
        Name = "asset_sound"
        Args = @("build", "--release", "--no-default-features", "--features", "pixel_tile,asset_sound")
    },
    @{
        Name = "first_window_v0_1_default_all_assets"
        Args = @("build", "--release")
    },
    @{
        Name = "v0_6_win32_pixel_probe"
        Args = @("build", "--release", "--features", "win32_pixel")
    }
)

$rows = foreach ($case in $cases) {
    Write-Host "== $($case.Name) =="
    & cargo @($case.Args)
    if ($LASTEXITCODE -ne 0) {
        throw "cargo failed for $($case.Name)"
    }

    $size = (Get-Item $exe).Length
    $remaining = $limit - $size
    $used = [Math]::Round(($size / $limit) * 100, 2)

    [PSCustomObject]@{
        Case = $case.Name
        ExeBytes = $size
        RemainingBytes = $remaining
        UsedPercent = $used
    }
}

$rows | Format-Table -AutoSize
