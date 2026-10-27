"""Original Core NotificationForm geometry methods with inert forms/screens."""
from pathlib import Path
import json
root=Path('/home/loic/.cache/observatory-readable/core/Observatory.UI/NotificationForm.cs');s=root.read_text()
def method(name):
 a=s.index(name);b=s.index('{',a);i=b+1;d=1
 while d:d+=(s[i]=='{')-(s[i]=='}');i+=1
 return s[a:i]
program='''using System;using System.Drawing;using System.Linq;using System.Collections.Generic;using System.Text.Json;
using System.Windows.Forms;
namespace System.Windows.Forms {public class Control{public int Width,Height;}public class Form:Control{public Point Location;}public class Screen{public Rectangle Bounds;public static Screen[] AllScreens=Array.Empty<Screen>();public static Screen PrimaryScreen=>AllScreens[0];public static Rectangle GetBounds(Control c)=>PrimaryScreen.Bounds;}}
public class Prefs{public int NativeNotifyScreen=-1,NativeNotifyCorner;}public static class Core{public static Prefs Default=new();}
public class NotificationForm:Form {bool _defaultPosition=true;Point _originalLocation;
METHODS
public void Init(double x,double y)=>AdjustPosition(x,y);}
class Program{static void Main(){var rows=new List<object>();foreach(var screen in new[]{new Rectangle(0,0,1920,1080),new Rectangle(1920,0,1280,1024),new Rectangle(-1281,-799,1281,799),new Rectangle(101,103,1001,801)})foreach(var size in new[]{new Size(355,145),new Size(444,181),new Size(178,72)})for(int corner=0;corner<8;corner++)foreach(var pos in new[]{new[]{-1d,-1d},new[]{0d,0d},new[]{0.125,0.5},new[]{0.5,0.5},new[]{1d,1d}}){Core.Default.NativeNotifyCorner=corner;Screen.AllScreens=new[]{new Screen{Bounds=screen}};var f=new NotificationForm{Width=size.Width,Height=size.Height};f.Init(pos[0],pos[1]);var initial=f.Location;f.AdjustOffset(true);var offset1=f.Location;f.AdjustOffset(true);var offset2=f.Location;Core.Default.NativeNotifyCorner=(corner+4)%8;f.AdjustOffset(false);rows.Add(new{Screen=new[]{screen.X,screen.Y,screen.Width,screen.Height},Size=new[]{size.Width,size.Height},Corner=corner,Position=pos,Initial=new[]{initial.X,initial.Y},Offset1=new[]{offset1.X,offset1.Y},Offset2=new[]{offset2.X,offset2.Y},ChangedCorner=(corner+4)%8,Closed=new[]{f.Location.X,f.Location.Y}});}Console.WriteLine(JsonSerializer.Serialize(rows));}}
'''.replace('METHODS',method('private void AdjustPosition(')+'\n'+method('public void AdjustOffset('))
out=Path('/home/loic/.cache/observatory-core-popup-oracle');out.mkdir(exist_ok=True);(out/'Program.cs').write_text(program);(out/'oracle.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><NoWarn>8632</NoWarn></PropertyGroup></Project>')
