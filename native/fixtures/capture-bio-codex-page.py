"""Capture original Codex page builders, compiling only readable source.

The map bitmap and gallery chooser are explicit inert seams. No installed
assembly is loaded, no journal/profile data is read, and no network is used.
Run capture-bio-predictions.py first to prepare the original CodexLib source.
"""
from pathlib import Path
import hashlib, html as html_module, json, re

source = Path('/home/loic/.cache/observatory-readable/bio/BioInsights/UIHandler.cs')
text = source.read_text(encoding='utf-8')
base = Path('/home/loic/.cache/observatory-bio-predictions-oracle')
out = Path('/home/loic/.cache/observatory-bio-codex-page-oracle')
out.mkdir(exist_ok=True)

def block(start):
    a = text.index('{', start)
    depth, quote, escaped, i = 1, None, False, a + 1
    while depth:
        c = text[i]
        if quote:
            if escaped: escaped = False
            elif c == '\\': escaped = True
            elif c == quote: quote = None
        elif c in ('"', "'"): quote = c
        elif c == '{': depth += 1
        elif c == '}': depth -= 1
        i += 1
    return text[start:i]

methods = {}
for m in re.finditer(r'^\t(?:public |private |internal |protected |static )[^\n;]*?\b(\w+)\([^\n]*\)\n\t\{', text, re.M):
    methods[m[1]] = block(m.start())
for name in ['FormatReq','RequirementAsString']:
    method = methods[name]
    declarations = re.findall(r'^\s*([\w<>\[\], ?]+ \w+ = default;)\s*$',method,re.M)
    for declaration in declarations:
        method = method.replace(declaration,'')
    at = method.index('{') + 1
    methods[name] = method[:at] + '\n' + '\n'.join(declarations) + method[at:]
names = ['buildBasePage','buildGenusPage','buildSpeciesPage','buildVariantsTable',
         'insertBioScanImage2','insertSwitchImage','RequirementAsString','FormatReq',
         'FirstLetterToUpper','StarTypeFromCode']
names += ['GenerateAboutPage','GenerateUnsoldDataPage']
needed = set(names)
seams = {'insertRegionMap','GetManifestLine'}
while True:
    body = '\n'.join(methods[n] for n in needed)
    more = {n for n in methods if n not in seams and re.search(r'\b' + n + r'\s*\(',body)}
    if more <= needed: break
    needed |= more

# The compiler-created sort helpers are pure property accessors.
nested = block(text.index('\tprivate sealed class _003C_003Ec'))
nested = nested.replace('\n\t\tstatic ', '\n\t\tpublic static ')
dictionary = block(text.index('\tpublic static Dictionary<VariantStar, string> RadiantStarName')) + ';'
code = '\n'.join(methods[n] for n in methods if n in needed)
code = code.replace('private StringBuilder build', 'public StringBuilder build')
# ILSpy places this IL local initialization at a reentered dispatch label.
# Hoist its declaration to restore the original single method-local lifetime.
code = code.replace('private string insertBioScanImage2(GenusType genus)\n\t{', 'private string insertBioScanImage2(GenusType genus)\n\t{\n\t\tUri uri = default;').replace('\n\t\tUri uri = default;\n\t\tswitch', '\n\t\tswitch')
symbols = (source.parent.parent/'EDJP.Observatory/OutputSymbol.cs').read_text(encoding='utf-8').replace('namespace EDJP.Observatory;','')
symbol_list = block(text.index('SymbolList = new List<OutputSymbol>')) + ';'
program = r'''
using System;using System.Text;using System.Collections;using System.Collections.Generic;
using System.Linq;using System.Globalization;using System.Runtime.CompilerServices;
using System.Text.Json;using System.Text.RegularExpressions;using CodexLib;using RegionLookup;
class Utils {public static string[] SplitCamelCase(string s)=>Regex.Split(s,"(?<!^)(?=[A-Z])");public static string FirstLetterToUpper(string s)=>s.Length==0?s:char.ToUpper(s[0])+s.Substring(1);}
class Logger {public void WriteLog(string s){}}
class Base {public Logger logger=new();}
SYMBOLS
class BiologicalCodexEntry {public const ulong AllRegions=8796093022206;public ulong Mask;
 public ulong ExistsInRegions(GenusType g,SpeciesType s,VariantColour c)=>Mask;}
class BioInsights {public static CodexChecker CodexChecker=new();public static string CurrentCommanderName="Synthetic";
 public static BiologicalCodexEntry CodexEntriesSeen=new(),CodexEntriesMarked=new();}
class ManifestLine {public string Name="",Cmdr="",Uri="gallery://synthetic";}
class Symbol {public string Description="";}
class Theme {public Dictionary<OutputSymbol,Symbol> SymbolLookup=Enum.GetValues<OutputSymbol>().ToDictionary(x=>x,x=>new Symbol{Description="description://"+x});}
class UIHandler:Base {
 string BaseURL="/bioinsights";
 string BaseWSURL="/bioinsights";
 NumberFormatInfo nfi=new(){NumberDecimalDigits=0,NumberGroupSeparator=" "};
 public string LastPage="";
 Theme theme=new();
 public List<OutputSymbol> SYMBOL_LIST
 ulong unsoldtotal;
 void AddButtonsAndOpenContentDiv(StringBuilder s){}
 void CloseContentDiv(StringBuilder s){}
 void SavePageAndMaybeUpdate(string page,StringBuilder s,bool showIfCurrent){LastPage=s.ToString();}
 DICTIONARY
 string GetSymbol(OutputSymbol s,string value=null)=>"symbol://"+s+(value==null?"":"["+value+"]");
 string insertRegionMap(ulong p,ulong s)=>"map://"+p+"/"+s;
 ManifestLine GetManifestLine(GenusType g,SpeciesType s)=>new();
 METHODS
 NESTED
}
class Program {static void Main(){var output=new List<object>();var ui=new UIHandler();
 output.Add(new{Genus=0,Species=0,Seen=0,Marked=false,Html=ui.buildBasePage(0).ToString()});
 foreach(var g in BioInsights.CodexChecker.Codex.Genus){
 output.Add(new{Genus=(int)g.Key,Species=0,Seen=0,Marked=false,Html=ui.buildGenusPage(g.Key).ToString()});
 foreach(var s in g.Value.Species){
 foreach(ulong seen in new ulong[]{0,2,6,BiologicalCodexEntry.AllRegions,1UL<<42}){
 BioInsights.CodexEntriesSeen.Mask=seen;BioInsights.CodexEntriesMarked.Mask=seen==6?BiologicalCodexEntry.AllRegions:0;
 output.Add(new{Genus=(int)g.Key,Species=(int)s.Key,Seen=seen,Marked=seen==6,Html=ui.buildSpeciesPage(g.Key,s.Key).ToString()});
 }}}
 ui.GenerateAboutPage();string about=ui.LastPage;
 var unsold=new List<object>();
 foreach(int count in new[]{0,1,2,4})foreach(bool bonus in new[]{false,true}){
 var lines=Enumerable.Range(0,count).Select(i=>(i+1,new DateTime(2026,10,26,9,5,47,DateTimeKind.Utc).AddDays(i),i<2?"Source A 1":"Source A 2",(short)(i==3?91:1+i),1234567+i,bonus)).ToList();
 ui.GenerateUnsoldDataPage(lines);unsold.Add(new{Count=count,Bonus=bonus,Html=ui.LastPage});
 }
 Console.WriteLine(JsonSerializer.Serialize(new{SourceSha="SOURCE_SHA",Cases=output,About=about,Unsold=unsold}));}}
'''.replace('METHODS',code).replace('NESTED',nested).replace('DICTIONARY',dictionary).replace('SYMBOL_LIST',symbol_list).replace('SYMBOLS',symbols).replace('SOURCE_SHA',hashlib.sha256(text.encode()).hexdigest())
(out/'Program.cs').write_text(program,encoding='utf-8')
project = (base/'oracle.csproj').read_text(encoding='utf-8')
for path in ['CodexLib/*.cs','CodexLib.SystemDetails/*.cs','RegionLookup/*.cs','Resources.cs','Framework/**/*.cs']:
    project = project.replace('Include="'+path+'"','Include="'+str(base/path)+'"')
(out/'oracle.csproj').write_text(project,encoding='utf-8')
print(json.dumps({'methods':len(needed),'source_sha':hashlib.sha256(text.encode()).hexdigest()}))

# After executing the source-only harness, --templates materializes its static
# installed-catalogue markup. Personal state and image assets remain live inputs.
import sys
if '--templates' in sys.argv:
    fixture = Path(__file__).with_name('bioinsights-codex-page-oracle.json')
    cases = json.loads(fixture.read_text(encoding='utf-8'))['Cases']
    templates = {}
    details = {}
    for case in cases:
        if case['Seen'] != 0: continue
        g,s = case['Genus'],case['Species']
        html = case['Html'].replace('CMDR Synthetic','CMDR $$Commander$$')
        if s:
            content=html.split('<span class="cc">',1)[1].split('</span>',1)[0]
            # A final BR terminates the preceding line; it does not add an
            # additional empty line box at the end of this inline span.
            lines=content.split('<br />')
            if lines[-1]=='':lines.pop()
            details[f'{g}/{s}']=[html_module.unescape(line)for line in lines]
        html = html.replace('<img src="gallery://synthetic" width=480 height=270 />','$$Gallery$$')
        html = re.sub(r'map://\d+/0',lambda _: '$$Map$$',html)
        html = re.sub(r'(<span id="regionmapname" class="cc">).*?(</span>)',r'\1$$MapName$$\2',html)
        html = re.sub(r'(id="mark-\d+-\d+-(\d+)">)symbol://OptionOff',lambda m:m[1]+'$$Mark:'+m[2]+'$$',html)
        html = re.sub(r'(id="show-\d+-\d+-(\d+)" class="cc">)No',lambda m:m[1]+'$$Seen:'+m[2]+'$$',html)
        templates[f'{g}/{s}'] = html
    target = Path(__file__).resolve().parents[1]/'observatory-plugins/src/bioinsights/companion-pages.json'
    target.write_text(json.dumps(templates,ensure_ascii=False,separators=(',',':')),encoding='utf-8')
    target.with_name('companion-page-details.json').write_text(json.dumps(details,ensure_ascii=False,separators=(',',':')),encoding='utf-8')
    about=json.loads(fixture.read_text(encoding='utf-8'))['About']
    about=re.sub(r'symbol://(\w+)(?:\[([^\]]+)\])?',lambda m:'['+m[1]+(':'+m[2] if m[2] else '')+']',about)
    about=re.sub(r'description://(\w+)',lambda m:'$$Description:'+m[1]+'$$',about)
    target.with_name('about-source.html').write_text(about,encoding='utf-8')
    print(json.dumps({'templates':len(templates),'bytes':target.stat().st_size}))
