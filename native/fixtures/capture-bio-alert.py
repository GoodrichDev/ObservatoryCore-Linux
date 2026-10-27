"""Trace original GenerateAlert with inert drawing calls; never loads the plugin."""
from pathlib import Path
import re
source = Path('/home/loic/.cache/observatory-readable/bio/BioInsights')
codex = Path('/home/loic/.cache/observatory-plugins/bio-codex/CodexLib')
def body(text, start):
    a=text.index(start); b=text.index('{',a); depth=1; end=b+1
    while depth:
        depth += (text[end]=='{')-(text[end]=='}'); end+=1
    return text[a:end]
alert=(source/'AlertBuilder.cs').read_text()
generate=body(alert,'public Bitmap GenerateAlert(')
wrappers=alert[alert.index('\tstatic Font '):alert.rfind('}')]
types='\n'.join(re.sub(r'namespace \w+;','',p.read_text()) for p in [source/'VisualAlert.cs',source/'VariantOutputLine.cs',source/'BioSeen.cs',source/'BioAnalysed.cs',codex/'GenusType.cs',codex/'SpeciesType.cs',codex/'VariantType.cs',codex/'VariantColour.cs'])
types=re.sub(r'^using .*;\s*','',types,flags=re.M)
program=r'''
using System; using System.Collections.Generic; using System.Linq;
using System.Drawing;using System.Drawing.Drawing2D;using System.Drawing.Imaging;
using System.Runtime.CompilerServices;using System.Text.Json;
public enum OutputSymbol {Likely,Maybe,Unlikely,PersonalCodexEntry,Mark,PossibleCodexEntry,PossibleCodexEntryNeverSeen}
public class VariantResult {
 public VariantColour Colour;public bool ShowInUI=true,PossibleCodexEntry,PossibleCodexEntryNeverSeen,PersonalCodexEntry,CCMark,Likely,Unlikely;
}
public class UIHandler { public string FormatMillions(int n) => (n/1000000.0).ToString("0.##",System.Globalization.CultureInfo.InvariantCulture); }
public class AlertBuilder {
 int FontHeight=16, FontHeightB=16, RowPaddingY=3,RowPaddingX=3,BorderWidth=2,SpaceWidth=4;
 UIHandler ui=new();Dictionary<string,Brush> Brushes=new(){["header"]=new Brush(),["row1"]=new Brush(),["row2"]=new Brush(),["border"]=new Brush()};
 public static List<object> Trace=new();
 public int WriteString(Graphics g,string f,string txt,string brush,int x,int y) {int width=txt.Length*8;Trace.Add(new{op="text",font=f,text=txt,brush,x,y,width});return width;}
 public int WriteString(Graphics g,OutputSymbol sym,int x,int y) {Trace.Add(new{op="symbol",symbol=sym.ToString(),x,y,width=8});return 8;}
GENERATE
WRAPPERS
}
namespace System.Drawing {
 public enum FontStyle{Regular,Bold} public class Font {public Font(string f,float s){} public Font(string f,float s,FontStyle st){}}
 public class Brush{} public class SolidBrush:Brush{public SolidBrush(Color c){}}
 public class Image:IDisposable{public void Dispose(){}}
 public class Bitmap:Image {public Bitmap(int w,int h){}public Bitmap(int w,int h,Imaging.PixelFormat f){}public Bitmap Clone(Rectangle r,Imaging.PixelFormat p){AlertBuilder.Trace.Add(new{op="crop",r.X,r.Y,r.Width,r.Height});return this;}}
 public class Pen:IDisposable {public Pen(Brush b,float w){}public void Dispose(){}}
 public class Graphics:IDisposable {
 public static Graphics FromImage(Image i)=>new();public Drawing2D.SmoothingMode SmoothingMode{get;set;}
 public SizeF MeasureString(string s,Font f)=>new(s.Length*8+4,16);
 public void FillRectangle(Brush b,int x,int y,int w,int h){AlertBuilder.Trace.Add(new{op="fill",x,y,w,h});}
 public void DrawLine(Pen p,int a,int b,int c,int d){AlertBuilder.Trace.Add(new{op="line",a,b,c,d});}
 public void DrawString(string s,Font f,Brush b,float x,float y){} public void Dispose(){}}
}
namespace System.Drawing.Drawing2D {public enum SmoothingMode{Default}}
namespace System.Drawing.Imaging {public enum PixelFormat{Default}}
public class Program {public static void Main(){
 var cases=new List<object>();
 for(int flags=0;flags<64;flags++){
 AlertBuilder.Trace.Clear();
 var lines=new List<VariantOutputLine>{
 new(){Genus="Bacterium",Species="Alcyoneum",BaseValue=1658500,IsPersonalCodexEntry=(flags&1)!=0,IsMarked=(flags&2)!=0,IsGalacticCodexEntry=(flags&4)!=0,IsGalacticCodexEntryNeverSeen=(flags&8)!=0,PossibleVariants=new(){new(){Colour=VariantColour.Green,PersonalCodexEntry=(flags&1)!=0,CCMark=(flags&2)!=0,PossibleCodexEntry=(flags&4)!=0,PossibleCodexEntryNeverSeen=(flags&8)!=0,Likely=(flags&16)!=0,Unlikely=(flags&32)!=0}}},
 new(){Genus="Bacterium",Species="Aurasus",BaseValue=1000000,PossibleVariants=new(){new(){Colour=VariantColour.Red,Likely=true},new(){Colour=VariantColour.Blue,Unlikely=true},new(){Colour=VariantColour.Grey}}},
 new(){Genus="Frutexa",Species="Flabellum",BaseValue=1800000,PossibleVariants=new(){new(){Colour=VariantColour.Emerald}}},
 new(){Genus="Anemone",Species="Luteolum Anemone",Type=VariantType.Horizons,BaseValue=500000,PossibleVariants=new()}
 };
 new AlertBuilder().GenerateAlert(new(){BodyName="Source 1 a",BioCount=4,Lines=lines});
 cases.Add(new{Flags=flags,Trace=AlertBuilder.Trace.ToArray()});
 }
 Console.WriteLine(JsonSerializer.Serialize(cases));
}}
'''.replace('GENERATE',generate).replace('WRAPPERS',wrappers)
program=program.replace('public class AlertBuilder {',types+'\npublic class AlertBuilder {')
out=Path('/home/loic/.cache/observatory-bio-alert-oracle');out.mkdir(exist_ok=True)
(out/'Program.cs').write_text(program)
(out/'oracle.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework><NoWarn>0436;0162;0168;0219</NoWarn></PropertyGroup></Project>')
