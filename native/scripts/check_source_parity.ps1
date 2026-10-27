#requires -Version 7.3
<#
.SYNOPSIS
Compile the original C# criteria in memory and check the captured source oracle.
.DESCRIPTION
Uses PowerShell's bundled Roslyn and runtime; no SDK, NuGet, Wine or original
project build is needed. The original C# files are read without modification.
Run in a fresh pwsh process. -UpdateExpected explicitly replaces the JSON capture.
#>
param(
    [string]$RepositoryRoot = (Join-Path $PSScriptRoot '../..'),
    [string]$Cases = '',
    [string]$Expected = '',
    [switch]$UpdateExpected,
    [switch]$AllowSourceErrors
)
$ErrorActionPreference = 'Stop'
$RepositoryRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path
if (-not $Cases) { $Cases = Join-Path $RepositoryRoot 'native/fixtures/oracle-cases.jsonl' }
if (-not $Expected) { $Expected = Join-Path $RepositoryRoot 'native/fixtures/oracle-expected.json' }

$relativeSources = @(
    'ObservatoryExplorer/DefaultCriteria.cs',
    'ObservatoryExplorer/ExplorerSettings.cs',
    'ObservatoryFramework/Attributes.cs',
    'ObservatoryFramework/Files/Journal/JournalBase.cs',
    'ObservatoryFramework/Files/Journal/Exploration/ScanBaryCentre.cs',
    'ObservatoryFramework/Files/Journal/Exploration/Scan.cs',
    'ObservatoryFramework/Files/Journal/Exploration/SAASignalsFound.cs',
    'ObservatoryFramework/Files/Journal/Exploration/FSSBodySignals.cs',
    'ObservatoryFramework/Files/Converters/MaterialCompositionConverter.cs',
    'ObservatoryFramework/Files/ParameterTypes/Parent.cs',
    'ObservatoryFramework/Files/ParameterTypes/Ring.cs',
    'ObservatoryFramework/Files/ParameterTypes/MaterialComposition.cs',
    'ObservatoryFramework/Files/ParameterTypes/Composition.cs',
    'ObservatoryFramework/Files/ParameterTypes/Genus.cs',
    'ObservatoryFramework/Files/ParameterTypes/Signal.cs'
)
$enumSource = 'ObservatoryFramework/Files/ParameterTypes/Enumerations.cs'
$sourceMaterial = [Text.StringBuilder]::new()
foreach ($relative in (@($relativeSources) + $enumSource | Sort-Object)) {
    [void]$sourceMaterial.Append($relative)
    [void]$sourceMaterial.Append("`n")
    [void]$sourceMaterial.Append([IO.File]::ReadAllText((Join-Path $RepositoryRoot $relative)).Replace("`r`n", "`n"))
    [void]$sourceMaterial.Append("`n")
}
$sourceHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($sourceMaterial.ToString())))
$enumText = [IO.File]::ReadAllText((Join-Path $RepositoryRoot $enumSource))
$parentEnum = [regex]::Match($enumText, '(?s)public enum ParentType\s*\{[^}]+\}').Value
if (-not $parentEnum) { throw 'The original ParentType enum could not be found.' }

$temporaryDirectory = Join-Path ([IO.Path]::GetTempPath()) ('observatory-source-parity-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temporaryDirectory | Out-Null
try {
    # Implicit project usings and the exact source enum are supplied separately.
    # The enum's unrelated JSON-converter attributes are unnecessary here; Scan
    # constructs ParentType values from the original Parent class itself.
    $globals = @"
global using System;
global using System.Collections.Generic;
global using System.Linq;
namespace Observatory.Framework.Files.ParameterTypes {
$parentEnum
}
"@
    $wrapperName = 'ObservatorySourceOracle' + [Guid]::NewGuid().ToString('N')
    $wrapper = @'
using System.Globalization;
using System.Text.Json;
using Observatory.Framework.Files.Journal;
using Observatory.Explorer;
public static class __WRAPPER__ {
    public static string Run(string input, bool allowSourceErrors) {
        CultureInfo.CurrentCulture = new CultureInfo("en-US");
        var settings = new ExplorerSettings();
        foreach (var property in typeof(ExplorerSettings).GetProperties().Where(p => p.PropertyType == typeof(bool)))
            property.SetValue(settings, true);
        var scans = new Dictionary<ulong, Dictionary<int, Scan>>();
        var signals = new Dictionary<ulong, Dictionary<int, FSSBodySignals>>();
        var results = new List<object>();
        foreach (var line in input.Split('\n').Where(s => !string.IsNullOrWhiteSpace(s))) {
            var value = JsonDocument.Parse(line).RootElement;
            if (value.GetProperty("event").GetString() == "FSSBodySignals") {
                var signal = JsonSerializer.Deserialize<FSSBodySignals>(line);
                if (!signals.ContainsKey(signal.SystemAddress)) signals[signal.SystemAddress] = new();
                signals[signal.SystemAddress][signal.BodyID] = signal;
            } else if (value.GetProperty("event").GetString() == "Scan") {
                var scan = JsonSerializer.Deserialize<Scan>(line);
                if (!scans.ContainsKey(scan.SystemAddress)) scans[scan.SystemAddress] = new();
                scans[scan.SystemAddress][scan.BodyID] = scan;
                try {
                    foreach (var result in DefaultCriteria.CheckInterest(scan, scans, signals, settings))
                        results.Add(new {body = result.SystemWide ? scan.StarSystem : scan.BodyName,
                            title = result.Description, detail = result.Detail});
                } catch (Exception error) when (allowSourceErrors) {
                    Console.Error.WriteLine($"Original source skipped {scan.BodyName}: {error.GetType().Name}: {error.Message}");
                }
            }
        }
        return JsonSerializer.Serialize(results);
    }
}
'@
    $globalsFile = Join-Path $temporaryDirectory 'Globals.cs'
    $wrapperFile = Join-Path $temporaryDirectory 'Oracle.cs'
    [IO.File]::WriteAllText($globalsFile, $globals)
    [IO.File]::WriteAllText($wrapperFile, $wrapper.Replace('__WRAPPER__', $wrapperName))
    $sources = @($globalsFile, $wrapperFile) + @($relativeSources | ForEach-Object { Join-Path $RepositoryRoot $_ })
    $types = Add-Type -Path $sources -PassThru -IgnoreWarnings -CompilerOptions '/nowarn:0618'
    $oracleType = $types | Where-Object Name -CEQ $wrapperName
    $inputText = [IO.File]::ReadAllText($Cases)
    $actualText = $oracleType.GetMethod('Run').Invoke($null, @($inputText, $AllowSourceErrors.IsPresent))
    $actual = @($actualText | ConvertFrom-Json)
    if ($UpdateExpected) {
        [IO.File]::WriteAllText($Expected, (ConvertTo-Json -InputObject $actual -Depth 10).Replace("`r`n", "`n") + "`n")
        Write-Output "Updated $Expected"
    } else {
        $expectedRows = @([IO.File]::ReadAllText($Expected) | ConvertFrom-Json)
        if ($actual.Count -ne $expectedRows.Count) {
            throw "Original source produced $($actual.Count) findings; captured oracle has $($expectedRows.Count)."
        }
        for ($index = 0; $index -lt $actual.Count; $index++) {
            foreach ($field in 'body', 'title', 'detail') {
                if (-not [string]::Equals($actual[$index].$field, $expectedRows[$index].$field, [StringComparison]::Ordinal)) {
                    throw "Original source differs from capture at finding $index, field $field. Review the source change before updating expected results."
                }
            }
        }
        Write-Output "Original source parity: $($actual.Count) findings match exactly in body, title, detail and order."
    }
    Write-Output "Original source set SHA256 (LF-normalized): $sourceHash"
    Write-Output "Runtime: $([System.Runtime.InteropServices.RuntimeInformation]::FrameworkDescription); PowerShell $($PSVersionTable.PSVersion)"
} finally {
    # Only remove the fresh, verified temporary directory belonging to this run.
    $resolvedTemporary = (Resolve-Path -LiteralPath $temporaryDirectory).Path
    $temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if (-not $resolvedTemporary.StartsWith($temporaryRoot, [StringComparison]::OrdinalIgnoreCase) -or
        -not [IO.Path]::GetFileName($resolvedTemporary).StartsWith('observatory-source-parity-', [StringComparison]::Ordinal)) {
        throw 'Refusing to remove a temporary directory outside this run.'
    }
    Remove-Item -LiteralPath $resolvedTemporary -Recurse -Force
}
