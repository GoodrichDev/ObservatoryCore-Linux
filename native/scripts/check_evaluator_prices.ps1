param(
    [Parameter(Mandatory=$true)][string]$SourceDirectory,
    [string]$Expected = 'native/fixtures/evaluator-prices.json',
    [switch]$UpdateExpected
)
$ErrorActionPreference='Stop'
$taskBodyPath=Join-Path $SourceDirectory 'Evaluator/Body.cs'
$taskExtraPath=Join-Path $SourceDirectory 'Evaluator/BodyExtra.cs'
$taskTemp=Join-Path ([IO.Path]::GetTempPath()) ('observatory-evaluator-oracle-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($taskTemp)|Out-Null
try {
    $taskBodySource=Get-Content -LiteralPath $taskBodyPath -Raw
    # ILSpy synthesizes initlocals declarations at a branch label. Hoist those
    # declarations so a goto does not incorrectly reset the decoded type name.
    $taskBodySource=$taskBodySource.Replace("`t`tIL_001a:`n`t`tint num3 = default;`n`t`tstring text = default;","`t`tIL_001a:")
    $taskBodySource=$taskBodySource.Replace("private double GetK(bool terraforming)`n`t{","private double GetK(bool terraforming)`n`t{`n`t`tint num3 = default;`n`t`tstring text = default;")
    [IO.File]::WriteAllText((Join-Path $taskTemp 'Body.cs'),$taskBodySource)
    Copy-Item -LiteralPath $taskExtraPath -Destination (Join-Path $taskTemp 'BodyExtra.cs')
    Copy-Item -LiteralPath (Join-Path $SourceDirectory 'Evaluator/EDSystem.cs') -Destination (Join-Path $taskTemp 'EDSystem.cs')
    $taskOrrerySources=@('Orrery.cs','Vec3.cs','OrreryElementType.cs','OrreryCollisionEvent.cs','OrreryCollisionEventType.cs') | ForEach-Object { $taskTarget=Join-Path $taskTemp $_; Copy-Item -LiteralPath (Join-Path $SourceDirectory ('OrreryLib/'+$_)) -Destination $taskTarget; $taskTarget }
    # Another ILSpy C# artifact reuses an enumerator identifier in the same method.
    # Rename only the first lexical using block; numeric/orbit operations stay intact.
    $taskOrreryPath=Join-Path $taskTemp 'Orrery.cs';$taskOrreryText=Get-Content -LiteralPath $taskOrreryPath -Raw
    $taskOrbitStart=$taskOrreryText.IndexOf('public void UpdatePositions');$taskOrbitEnd=$taskOrreryText.IndexOf('using Dictionary', $taskOrbitStart)
    $taskOrreryText=$taskOrreryText.Substring(0,$taskOrbitStart)+$taskOrreryText.Substring($taskOrbitStart,$taskOrbitEnd-$taskOrbitStart).Replace('enumerator','relativeEnumerator')+$taskOrreryText.Substring($taskOrbitEnd)
    [IO.File]::WriteAllText($taskOrreryPath,$taskOrreryText)
    $taskStub=@'
namespace Observatory.Framework.Files.Journal {
 public class ScanBaryCentre { public int BodyID {get;set;} }
 public class Scan:ScanBaryCentre {
  public string StarType {get;set;}=""; public string PlanetClass {get;set;}="";
  public string BodyName {get;set;}=""; public string Volcanism {get;set;}="";
  public string TerraformState {get;set;}=""; public string AtmosphereType {get;set;}="";
  public string ScanType {get;set;}="Detailed";
  public bool Landable {get;set;} public bool WasDiscovered {get;set;}
  public bool WasMapped {get;set;} public bool WasFootfalled {get;set;}
  public float MassEM {get;set;} public float StellarMass {get;set;}
  public float SurfaceGravity {get;set;} public float SurfacePressure {get;set;}
  public double DistanceFromArrivalLS {get;set;}
 }
}
namespace Evaluator {
 public class StubSettings {
  public int MinimumValueForWorthMapping {get;set;}=1000000;
  public int MaxDistanceELW {get;set;} public int MaxDistanceWW {get;set;}
  public int MaxDistanceAW {get;set;} public int MaxDistanceOther {get;set;}
  public bool BIHighValueWorthMapping {get;set;} public bool BIPersonalCodexWorthMapping {get;set;}
  public bool BIGalacticCodexWorthMapping {get;set;} public bool BIMarkedWorthMapping {get;set;}
  public bool AlwaysShowMain {get;set;} public bool AlwaysShowELW {get;set;} public bool AlwaysShowWW {get;set;}
  public bool AlwaysShowTerraformables {get;set;} public bool AlwaysShowAtmospherics {get;set;}
  public bool AlwaysShowAtmosphericsBios {get;set;} public bool AlwaysShowNonAtmosphericsBios {get;set;}
  public bool AlwaysShowBIHighValue {get;set;} public bool AlwaysShowBIPersonalCodex {get;set;}
  public bool AlwaysShowBIGalacticCodex {get;set;} public bool AlwaysShowBIMarked {get;set;}
 }
 public static class Evaluator { public static StubSettings EvalSettings=new StubSettings(); }
}
namespace EDJP.Observatory { public class Logger { public void WriteLog(string value) {} } }
'@
    $taskStubPath=Join-Path $taskTemp 'Stub.cs'
    [IO.File]::WriteAllText($taskStubPath,$taskStub)
    Add-Type -Path (@((Join-Path $taskTemp 'Body.cs'),(Join-Path $taskTemp 'BodyExtra.cs'),(Join-Path $taskTemp 'EDSystem.cs'),$taskStubPath)+$taskOrrerySources) -IgnoreWarnings -WarningAction SilentlyContinue
    $taskBodyType=[Evaluator.Body]
    $taskGetK=$taskBodyType.GetMethod('GetK',[Reflection.BindingFlags]'NonPublic,Instance')
    $taskGetPlanet=$taskBodyType.GetMethod('GetPlanetValue',[Reflection.BindingFlags]'NonPublic,Static')
    $taskGetStar=$taskBodyType.GetMethod('GetStarValue',[Reflection.BindingFlags]'NonPublic,Static')
    $taskTypes=@('Sudarsky class I gas giant','Sudarsky class II gas giant','Sudarsky class III gas giant','Sudarsky class IV gas giant','Sudarsky class V gas giant','Water giant','Water giant with life','Gas giant with water based life','Gas giant with ammonia based life','Helium-rich gas giant','Helium gas giant','Earthlike body','Water world','Ammonia world','Metal rich body','High metal content body','Rocky body','Rocky ice body','Icy body','Unknown')
    $taskStars=@('O','B','A','F','G','K','M','L','T','Y','TTS','AeBe','W','WC','WN','WNC','WO','CS','S','MS','DA','DAB','DB','DC','D','N','H','SupermassiveBlackHole','X')
    $taskK=[Collections.Generic.List[object]]::new()
    foreach($taskIsStar in @($false,$true)) {
        $taskNames=if($taskIsStar){$taskStars}else{$taskTypes}
        foreach($taskName in $taskNames) {
            $taskBody=[Runtime.CompilerServices.RuntimeHelpers]::GetUninitializedObject($taskBodyType)
            $taskBody.IsStar=$taskIsStar; $taskBody.BodyType=$taskName
            $taskK.Add([ordered]@{body_type=$taskName;is_star=$taskIsStar;k=$taskGetK.Invoke($taskBody,@($false));terraform_k=$taskGetK.Invoke($taskBody,@($true))})
        }
    }
    $taskValues=[Collections.Generic.List[object]]::new()
    foreach($taskKs in @(300.0,9654.0,21790.0,64831.0,93328.0,96932.0,116295.0,1200.0,22628.0,14057.0)) {
        foreach($taskMass in @(0.0,0.000001,0.1,1.0,5.97,10.0,1000.0)) {
            foreach($taskFd in @($false,$true)) {foreach($taskMapped in @($false,$true)) {foreach($taskFm in @($false,$true)) {foreach($taskEff in @($false,$true)) {foreach($taskOdy in @($false,$true)) {foreach($taskCarrier in @($false,$true)) {
                $taskValue=$taskGetPlanet.Invoke($null,@([double]$taskKs,[double]$taskMass,$taskFd,$taskMapped,$taskFm,$taskEff,$taskOdy,$taskCarrier))
                $taskValues.Add(@($taskKs,$taskMass,$taskFd,$taskMapped,$taskFm,$taskEff,$taskOdy,$taskCarrier,$taskValue))
            }}}}}}
        }
    }
    $taskStarValues=[Collections.Generic.List[object]]::new()
    foreach($taskKs in @(1200.0,14057.0,22628.0)) {foreach($taskMass in @(0.0,0.01,1.0,10.0,100.0)) {foreach($taskFd in @($false,$true)) {foreach($taskCarrier in @($false,$true)) {
        $taskStarValues.Add(@($taskKs,$taskMass,$taskFd,$taskCarrier,$taskGetStar.Invoke($null,@([double]$taskKs,[double]$taskMass,$taskFd,$taskCarrier))))
    }}}}
    $taskModels=[Collections.Generic.List[object]]::new()
    foreach($taskName in $taskTypes) {foreach($taskFd in @($false,$true)) {foreach($taskTerra in @($false,$true)) {foreach($taskMapped in @($false,$true)) {foreach($taskOdy in @($false,$true)) {
        $taskScan=[Observatory.Framework.Files.Journal.Scan]::new();$taskScan.PlanetClass=$taskName;$taskScan.BodyName='Oracle 1';$taskScan.BodyID=1;$taskScan.MassEM=1.07317;$taskScan.Landable=$true;$taskScan.SurfaceGravity=2.12345;$taskScan.SurfacePressure=0.12345;$taskScan.AtmosphereType='CarbonDioxide';$taskScan.WasDiscovered=$taskFd;$taskScan.DistanceFromArrivalLS=999
        if($taskTerra){$taskScan.TerraformState='Terraformable'}
        $taskBody=[Evaluator.Body]::new($taskScan,$taskOdy,[Evaluator.BodyExtra]::new());$taskBody.IsMapped=$taskMapped;$taskBody.IsMappedEfficiently=$taskMapped;$taskBody.UpdateValues()
        $taskModels.Add([ordered]@{kind=$taskName;discovered=$taskFd;terraform=$taskTerra;mapped=$taskMapped;odyssey=$taskOdy;current=$taskBody.CurrentValue;potential=$taskBody.PotentialMaxValue;main_bonus=$taskBody.MainBonusValue;atmospheric=$taskBody.IsAtmospheric;gravity=$taskBody.Gravity;worth=$taskBody.WorthMapping})
    }}}}}
    $taskWorth=[Collections.Generic.List[object]]::new()
    foreach($taskName in @('Earthlike body','Water world','Ammonia world','Icy body')) {foreach($taskMax in @(0,500,1000)) {foreach($taskMin in @(0,1,1000000)) {foreach($taskBio in @($false,$true)) {
        $taskScan=[Observatory.Framework.Files.Journal.Scan]::new();$taskScan.PlanetClass=$taskName;$taskScan.BodyName='Oracle 1';$taskScan.BodyID=1;$taskScan.MassEM=1;$taskScan.WasDiscovered=$true;$taskScan.DistanceFromArrivalLS=999
        [Evaluator.Evaluator]::EvalSettings=[Evaluator.StubSettings]::new();$taskSettings=[Evaluator.Evaluator]::EvalSettings;$taskSettings.MinimumValueForWorthMapping=$taskMin;$taskSettings.MaxDistanceELW=$taskMax;$taskSettings.MaxDistanceWW=$taskMax;$taskSettings.MaxDistanceAW=$taskMax;$taskSettings.MaxDistanceOther=$taskMax;$taskSettings.BIHighValueWorthMapping=$taskBio
        $taskExtra=[Evaluator.BodyExtra]::new();$taskExtra.BioInsightsHighValue=$true
        $taskBody=[Evaluator.Body]::new($taskScan,$true,$taskExtra)
        $taskWorth.Add(@($taskName,$taskMax,$taskMin,$taskBio,$taskBody.WorthMapping))
    }}}}
    [Evaluator.Evaluator]::EvalSettings=[Evaluator.StubSettings]::new()
    $taskSystem=[Evaluator.EDSystem]::new('Oracle',[ulong]123,[datetime]::UtcNow,[EDJP.Observatory.Logger]::new())
    $taskScan=[Observatory.Framework.Files.Journal.Scan]::new();$taskScan.StarType='G';$taskScan.BodyName='Oracle';$taskScan.BodyID=0;$taskScan.StellarMass=1
    $taskBody=[Evaluator.Body]::new($taskScan,$true,[Evaluator.BodyExtra]::new());$taskSystem.Bodies.Add(0,$taskBody)
    $taskMain=[Evaluator.EDSystem].GetProperty('mainStarId',[Reflection.BindingFlags]'NonPublic,Instance');$taskMain.SetValue($taskSystem,0)
    $taskBonuses=[Collections.Generic.List[object]]::new()
    foreach($taskTotal in @(0,1,3)) {$taskSystem.TotalBodies=$taskTotal;$taskSystem.UpdateMainStarBonus();$taskBonuses.Add(@($taskTotal,$taskSystem.MainStarBonus,$taskSystem.TotalValue,$taskSystem.FullyScannedBonus,$taskSystem.FullyMappedBonus))}
    $taskScan=[Observatory.Framework.Files.Journal.Scan]::new();$taskScan.PlanetClass='Earthlike body';$taskScan.BodyName='Oracle 1';$taskScan.BodyID=1;$taskScan.MassEM=1
    $taskSystem.Bodies.Add(1,[Evaluator.Body]::new($taskScan,$true,[Evaluator.BodyExtra]::new()));$taskSystem.TotalBodies=2;$taskSystem.UpdateMainStarBonus()
    $taskBonuses.Add(@(2,$taskSystem.MainStarBonus,$taskSystem.TotalValue,$taskSystem.FullyScannedBonus,$taskSystem.FullyMappedBonus))
    $taskSystem.Bodies[1].IsMapped=$true;$taskSystem.Bodies[1].IsMappedEfficiently=$true;$taskSystem.Bodies[1].UpdateValues();$taskSystem.UpdateMainStarBonus()
    $taskBonuses.Add(@(2,$taskSystem.MainStarBonus,$taskSystem.TotalValue,$taskSystem.FullyScannedBonus,$taskSystem.FullyMappedBonus))
    $taskOrbits=[Collections.Generic.List[object]]::new();$taskEpoch=[datetime]::Parse('2026-01-01T00:00:00Z').ToUniversalTime()
    foreach($taskElements in @(@(0.0,0.0,0.0,0.0,0.0),@(63.0,11.0,53.0,10.0,0.6),@(103.0,173.0,-30.0,333.0,0.95))) {foreach($taskElapsed in @(0.0,100.0,10000.0,50000.0,10000000.0)) {
        $taskOrbit=[OrreryLib.Orrery]::new()
        $taskOrbit.Add(1,'Oracle',[OrreryLib.OrreryElementType]1,$taskElements[0],$taskElements[1],$taskElements[2],$taskElements[3],$taskEpoch,50000.0,$taskElements[4],123456789.0,0.0,[Collections.Generic.List[int]]::new())
        $taskMethod=[OrreryLib.Orrery].GetMethod('UpdateRelativePosition',[Reflection.BindingFlags]'NonPublic,Instance')
        $taskMethod.Invoke($taskOrbit,@([double]($taskEpoch.AddSeconds($taskElapsed).ToOADate()*86400),[int]1))|Out-Null
        $taskPos=$taskOrbit.Relative[0]
        $taskOrbits.Add(@($taskElements[0],$taskElements[1],$taskElements[2],$taskElements[3],$taskElements[4],$taskElapsed,$taskPos.x,$taskPos.y,$taskPos.z))
    }}
    $taskResult=[ordered]@{k=$taskK;planets=$taskValues;stars=$taskStarValues;models=$taskModels;worth=$taskWorth;bonuses=$taskBonuses;orbits=$taskOrbits}
    $taskJson=ConvertTo-Json -InputObject $taskResult -Depth 12 -Compress
    if($UpdateExpected) {[IO.File]::WriteAllText([IO.Path]::GetFullPath($Expected),$taskJson+"`n")}
    else { $taskCaptured=Get-Content -LiteralPath $Expected -Raw | ConvertFrom-Json -AsHashtable; $taskExpectedJson=ConvertTo-Json -InputObject $taskCaptured -Depth 12 -Compress; if($taskExpectedJson -cne $taskJson){throw 'Evaluator pricing source capture differs.'} }
    Write-Output "Evaluator pricing source oracle: $($taskK.Count) type constants, $($taskValues.Count) planet combinations, $($taskStarValues.Count) star combinations."
    Write-Output "Models: $($taskModels.Count) body states, $($taskWorth.Count) worth-mapping combinations, $($taskBonuses.Count) system bonus states, $($taskOrbits.Count) original Orrery positions."
    Write-Output ('Body.cs SHA256: '+(Get-FileHash -LiteralPath $taskBodyPath -Algorithm SHA256).Hash)
} finally {
    $taskAbsolute=[IO.Path]::GetFullPath($taskTemp)
    $taskTempRoot=[IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if($taskAbsolute.StartsWith($taskTempRoot,[StringComparison]::OrdinalIgnoreCase) -and [IO.Path]::GetFileName($taskAbsolute).StartsWith('observatory-evaluator-oracle-')){Remove-Item -LiteralPath $taskAbsolute -Recurse -Force}
}
