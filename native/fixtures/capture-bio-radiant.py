"""Capture the original BSystem stellar enrichment using synthetic DTOs only."""
from pathlib import Path
import runpy,re
here=Path(__file__).resolve().parent
ns=runpy.run_path(str(here/'capture-bio-predictions.py'))
out=ns['out'];readable=ns['readable'];source=Path('/home/loic/.cache/observatory-readable/bio')
for relative in ['BioInsights.SystemDetails/BSystem.cs','BioInsights/BIPlanetSpecies.cs','BioInsights/BioSeen.cs','BioInsights/BioAnalysed.cs']:
    target=out/'BioContext'/Path(relative).name;target.parent.mkdir(exist_ok=True);target.write_text(('using GenusType = CodexLib.GenusType;\n' if target.name=='BSystem.cs' else '')+readable((source/relative).read_text()).replace('\t\tstatic ','\t\tinternal static '))
(out/'BioContext/Logger.cs').write_text('namespace EDJP.Observatory { public enum LogLevel { Normal,Verbose,Error,Debug } public class Logger { public void WriteLog(params object[] args){} public void WriteException(params object[] args){} } }')
project=(out/'oracle.csproj').read_text().replace('<Compile Include="Program.cs"/>','<Compile Include="RadiantProgram.cs"/><Compile Include="BioContext/*.cs"/>').replace('<OutputType>Exe','<AssemblyName>radiant</AssemblyName><OutputType>Exe')
(out/'radiant.csproj').write_text(project)
(out/'RadiantProgram.cs').write_text((here/'bioinsights-radiant-oracle.cs').read_text())

inputs=runpy.run_path(str(here/'bioinsights-radiant-inputs.py'))['generate']()
import json
(out/'radiant-inputs.json').write_text(json.dumps(inputs))
