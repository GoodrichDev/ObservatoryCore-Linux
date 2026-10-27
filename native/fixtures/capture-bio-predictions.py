"""Compile the original pure CodexLib source; decode constants statically.
No installed assembly or module initializer is loaded. The harness uses only
synthetic scans and ships captured source results for native differential tests.
"""
from pathlib import Path
import json,lzma,re,struct
root=Path('/home/loic/.cache/observatory-plugins/bio-codex')
out=Path('/home/loic/.cache/observatory-bio-predictions-oracle');out.mkdir(exist_ok=True)
module=(root/'-Module-.cs').read_text()
array=re.search(r'uint\[\] array = new uint\[560\]\s*\{(.*?)\};',module,re.S)[1]
words=[int(n)for n in re.findall(r'(\d+)u',array)];assert len(words)==560
seed=2472186668;keys=[]
for _ in range(16):seed^=seed>>12;seed^=(seed<<25)&0xffffffff;seed^=seed>>27;seed&=0xffffffff;keys.append(seed)
clear=bytearray()
for start in range(0,len(words),16):
    for i in range(16):v=words[start+i]^keys[i];clear.extend(struct.pack('<I',v));keys[i]^=v
props=clear[0];dictionary,length=struct.unpack_from('<II',clear,1)
decoder=lzma.LZMADecompressor(format=lzma.FORMAT_RAW,filters=[dict(id=lzma.FILTER_LZMA1,dict_size=dictionary,lc=props%9,lp=props//9%5,pb=props//45)])
table=decoder.decompress(bytes(clear[9:]),max_length=length);assert len(table)==length
decoders={}
for m in re.finditer(r'internal static _002D ([_0-9A-F]+)<_002D>\(int id\)(.*?)(?=\n\tinternal static|\n\})',module,re.S):
    ar=re.search(r'id = \(id \* (-?\d+)\) \^ (-?\d+|0x[0-9A-F]+);',m[2]);assert ar
    decoders[m[1]]=(int(ar[1]),int(ar[2],16 if ar[2].startswith('0x')else 10))
def constant(m):
    a,b=decoders[m[1]];n=((int(m[2])*a)&0xffffffff)^(b&0xffffffff);offset=(n&0x3fffffff)<<2
    size=struct.unpack_from('<I',table,offset)[0];return json.dumps(table[offset+4:offset+4+size].decode(),ensure_ascii=False)
expression=re.compile(r'_003CModule_003E\.([_0-9A-F]+)<string>\((-?\d+)\)')
def readable(source):
    source=expression.sub(constant,source).replace('base._002Ector();','')
    source=re.sub(r'P_0\.set_(\w+)\(([^;]+)\);',r'P_0.\1 = \2;',source)
    source=re.sub(r'(set\s*\{\s*[_A-Za-z0-9]+\s*=\s*)(text|num|erC|obj)(;\s*\})',r'\1value\3',source)
    if 'public class CodexChecker\n' in source:
        marker='using Dictionary<VariantColour, VariantResult>.ValueCollection.Enumerator enumerator3'
        a=source.index(marker);b=source.index(marker,a+len(marker));source=source[:a]+source[a:b].replace('enumerator3','variantFilterEnumerator')+source[b:]
        a=source.index('private bool A_003DFB_');b=source.index('\n\t}',a)+4
        method=source[a:b].replace('bool result = false;','string text2="";double? dValue=null;\n\t\tbool result = false;').replace('string text2 = text;','text2 = text;').replace('double? dValue = P_1.DValue;','dValue = P_1.DValue;')
        source=source[:a]+method+source[b:]
    return source
for directory in ['CodexLib','CodexLib.SystemDetails','RegionLookup']:
    for file in(root/directory).glob('*.cs'):
        target=out/directory/file.name;target.parent.mkdir(exist_ok=True);target.write_text(readable(file.read_text()))
resources=(root/'-l-w-A--G1DnF81-k98-Ji-r4.cs').read_text()
(out/'Resources.cs').write_text(readable(resources).replace('= cultureInfo;','= value;'))
repo=Path(__file__).resolve().parents[2]
for file in(repo/'ObservatoryFramework/Files').rglob('*.cs'):
    target=out/'Framework'/file.relative_to(repo/'ObservatoryFramework/Files');target.parent.mkdir(parents=True,exist_ok=True);target.write_text(file.read_text(encoding='utf-8-sig').replace('init;','set;').replace('init\n','set\n'))
project=f'''<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework><EnableDefaultCompileItems>false</EnableDefaultCompileItems><ImplicitUsings>enable</ImplicitUsings><NoWarn>0108;0162;0168;0219;0618;0672;8632;8981</NoWarn><GenerateAssemblyInfo>false</GenerateAssemblyInfo></PropertyGroup><ItemGroup><Compile Include="CodexLib/*.cs"/><Compile Include="CodexLib.SystemDetails/*.cs"/><Compile Include="RegionLookup/*.cs"/><Compile Include="Resources.cs"/><Compile Include="Program.cs"/><Compile Include="Framework/**/*.cs"/><EmbeddedResource Include="{root}/-l-w-A--G1DnF81.-k98-Ji-r4.resx" LogicalName="RESOURCE_NAME.resources"/></ItemGroup></Project>'''
# The ResourceManager name is itself one statically decoded source constant.
name=json.loads(re.search(r'= _[A-Za-z0-9_]+\(("(?:[^"\\]|\\.)*"),',(out/'Resources.cs').read_text())[1])
for file in out.rglob('*.cs'):file.write_text(file.read_text().replace(json.dumps(name),json.dumps('BioOracleResources')))
project=project.replace('</ItemGroup>', ''.join(f'<EmbeddedResource Include="{root/name}" LogicalName="{name}"/>'for name in ['CodexLib.Resources.spheres.csv','CodexLib.Resources.species_description.tsv','CodexLib.Resources.codex.json.gz'])+'</ItemGroup>')
(out/'oracle.csproj').write_text(project.replace('RESOURCE_NAME','BioOracleResources'))
(out/'Program.cs').write_text(Path(__file__).with_name('bioinsights-predictions-oracle.cs').read_text())
import runpy
inputs=runpy.run_path(str(Path(__file__).with_name('bioinsights-prediction-inputs.py')))['generate']()
(out/'inputs.json').write_text(json.dumps(inputs))
print(json.dumps({'decoded_bytes':len(table),'decoder_count':len(decoders),'resource_name':name}))
