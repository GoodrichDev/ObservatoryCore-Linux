#!/usr/bin/env python3
"""Copy source queue layout methods into a graphics-recording oracle, offline."""
from pathlib import Path
import re,hashlib,struct
src=Path('/home/loic/.cache/observatory-readable/stellar/StellarOverlay/NotificationQueue.cs').read_text()
def method(sig):
 start=src.index(sig);brace=src.index('{',start);depth=1;end=brace+1
 while depth:
  if src[end]=='{':depth+=1
  elif src[end]=='}':depth-=1
  end+=1
 return src[start:end]
body=method('public void AddNotificationItem(Graphics gfx, NotificationArgs item, ScreenReferencePosition? refpos, NotificationTransition intrans, NotificationTransition outtrans)')
# Field name is SHA256 of the source RVA bytes; verify instead of guessing trim chars.
trim=struct.pack('<3H',32,13,10);assert hashlib.sha256(trim).hexdigest().upper()=='631DD2A447570DAED0FB71936AA88BF84C797C036F13ED17901D2B0482D9884D'
body=re.sub(r'(_\w+)\(\(Array\)(array2?), __ldtoken\([^;]+;',lambda m:m[2]+"[0]=' ';"+m[2]+"[1]='\\r';"+m[2]+"[2]='\\n';",body)
helpers=[]
for name in sorted(set(re.findall(r'\b(_\w+)\(',body))):
 sig=re.search(r'^\tstatic [^\n]+\b'+name+r'\(',src,re.M)
 if sig:helpers.append(method(sig.group().rstrip('(')))
head='''// Actual AddNotificationItem source body. Only RVA trim constant substitution and data/graphics stubs are authored.
using System;using System.Linq;using System.Collections.Generic;using System.Runtime.CompilerServices;using System.Text.Json;
public enum NotificationTransition{None,Left,Right,Fade}
public enum NotificationItemShape{Closed,Open}
public enum Alignment{Left,Center,Right}
public enum ScreenReferenceDirection{None,Left,Right,Up,Down,Center,Proportional}
public class Point{public float X{get;set;}public float Y{get;set;}public Point(float x,float y){X=x;Y=y;}}
public class Font{public float FontSize;public Font(float size){FontSize=size;}}
public class Graphics{public int Width=1920,Height=1080;public Point MeasureString(Font f,float size,string text){if(string.IsNullOrEmpty(text))return new Point(0,0);var lines=text.Replace("\\r\\n","\\n").Split('\\n');return new Point(lines.Max(l=>l.Length)*size*.5f,lines.Length*size);}}
public class NotificationArgs{public string Title="",Detail="",Sender="Test";public int Timeout=-1;public Guid? Guid;public double XPos=-1,YPos=-1;}
public class ScreenReferencePosition{public int X{get;set;}public int Y{get;set;}public ScreenReferenceDirection Horizontal{get;set;}public ScreenReferenceDirection Vertical{get;set;}public ScreenReferencePosition(int x,int y,ScreenReferenceDirection h,ScreenReferenceDirection v){X=x;Y=y;Horizontal=h;Vertical=v;}}
public class NotificationItemLine{public string Text{get;set;}public float FontSize{get;set;}public bool IsBold{get;set;}public int OffsetY{get;set;}public Alignment Alignment{get;set;}public NotificationItemLine(string text,float fontSize,bool isBold,int offsetY,Alignment alignment){Text=text;FontSize=fontSize;IsBold=isBold;OffsetY=offsetY;Alignment=alignment;}}
public class BrushHandler{}
public class NotificationItem{public List<NotificationItemLine> Lines{get;set;}public Point Dimensions{get;set;}public long TTLms{get;set;}public ScreenReferencePosition Position{get;set;}public NotificationItem(List<NotificationItemLine> lines,Point dimensions,long ttl,ScreenReferencePosition position,NotificationTransition i,NotificationTransition o,int border,NotificationItemShape shape,string sender,BrushHandler brushes){Lines=lines;Dimensions=dimensions;TTLms=ttl;Position=position;}public void Update(int width,int height){}}
public class NotificationQueue{public int verticalPositionPercent=97;public bool fromRight=true;public float FontHeight;public bool ReinitialiseFont;public static int NotificationSpacing=15,EdgeIndent=20;public List<ScreenReferencePosition> ReferencePosition=new();public Font Bold=new Font(14),Normal=new Font(12);public float FontSize=12;public float MinimumFontSize=>FontSize*.75f;public int MaxWidth=480,DefaultTTL=5000,BorderWidth=3;public Alignment TextAlignment=Alignment.Left;public static int PaddingX=10,PaddingY=5,CornerRounding=20,ExtraBottomPadding=5,FontVerticalSpacing=0;public NotificationItemShape DefaultShape=NotificationItemShape.Open;public BrushHandler BrushHandler=new();public List<NotificationItem> QueueItems=new();public Dictionary<Guid,NotificationItem> InstantItems=new();
'''
foot='''
}
public class Program{public static void Main(){var cases=new List<object>();var texts=new[]{"","Short","Long words with multiple spaces and more words than fit inside a narrow notification width",new string('x',200),"First line\\r\\nSecond line\\r\\nThird", "  Trim me\\r\\n  "};foreach(var title in texts)foreach(var detail in texts)foreach(var maxWidth in new[]{120,240,480})foreach(var fixedPosition in new[]{false,true}){var q=new NotificationQueue{MaxWidth=maxWidth};var item=new NotificationArgs{Title=title,Detail=detail,XPos=fixedPosition?25:-1,YPos=fixedPosition?75:-1};q.AddNotificationItem(new Graphics(),item,fixedPosition?null:new ScreenReferencePosition(1900,1000,ScreenReferenceDirection.Left,ScreenReferenceDirection.Up),NotificationTransition.Left,NotificationTransition.Right);var output=q.QueueItems.SingleOrDefault()??q.InstantItems.Values.SingleOrDefault();cases.Add(new{title,detail,maxWidth,fixedPosition,output=JsonSerializer.SerializeToElement(output)});}var references=new List<object>();foreach(var height in new[]{720,1080,1440})foreach(var pct in new[]{0,25,50,51,97,100})foreach(var fromRight in new[]{false,true})foreach(var count in new[]{1,3,5}){var q=new NotificationQueue{verticalPositionPercent=pct,fromRight=fromRight};q.InitialiseFontHeight(new Graphics{Height=height},count);references.Add(new{height,pct,fromRight,count,positions=q.ReferencePosition});}Console.WriteLine(JsonSerializer.Serialize(new{cases,references}));}}
'''
# Include actual source Draw; graphics methods record calls, never render vendor code.
itemsrc=Path('/home/loic/.cache/observatory-readable/stellar/StellarOverlay/NotificationItem.cs').read_text()
oldsrc=src;src=itemsrc
draw=method('public void Draw(Graphics gfx, NotificationQueue queue)')
assert hashlib.sha256(struct.pack('<3H',13,32,9)).hexdigest().upper()=='6AE07D428F6CAF67324E4BCAF59665768CB02724CDD8A90DA23F7430FCD3B94F'
draw=re.sub(r'(_\w+)\(\(Array\)(array), __ldtoken\([^;]+;',lambda m:m[2]+"[0]='\\r';"+m[2]+"[1]=' ';"+m[2]+"[2]='\\t';",draw)
drawhelpers=[]
for name in sorted(set(re.findall(r'\b(_\w+)\(',draw))):
 sig=re.search(r'^\tstatic [^\n]+\b'+name+r'\(',src,re.M)
 if sig:drawhelpers.append(method(sig.group().rstrip('(')))
src=oldsrc
head=head.replace('public class NotificationItem{','public class NotificationItem{public float PositionX=12,PositionY=34;public string BackgroundColorKey="notification_background",BorderColorKey="notification_border",TextColorKey="notification_text",TitleColorKey="notification_title";public int BorderWidth=3;public NotificationItemShape shape=NotificationItemShape.Open;'+draw+'\n'+'\n'.join(drawhelpers))
head=head.replace('public class BrushHandler{}','public class SolidBrush{public string Key;public SolidBrush(string key){Key=key;}}public class BrushHandler{public bool HasBrush(string key){return true;}public SolidBrush Brush(string key){return new SolidBrush(key);}}')
head=head.replace('public class Graphics{','public class Graphics{public bool NeedsClear;public List<object> Ops=new();public void DrawLine(SolidBrush b,float x,float y,float x2,float y2,float stroke){Ops.Add(new {op="Line",key=b.Key,args=new[]{x,y,x2,y2,stroke}});}public void DrawRoundedRectangle(SolidBrush b,float x,float y,float x2,float y2,float radius,float stroke){Ops.Add(new{op="Rectangle",key=b.Key,args=new[]{x,y,x2,y2,radius,stroke}});}public void FillRoundedRectangle(SolidBrush b,float x,float y,float x2,float y2,float radius){Ops.Add(new{op="Fill",key=b.Key,args=new[]{x,y,x2,y2,radius}});}public void DrawText(Font f,float size,SolidBrush b,float x,float y,string text){Ops.Add(new{op="Text",key=b.Key,args=new[]{x,y,size},text});}')
head=head.replace('public string Text{get;set;}','public List<int> Length=new();public List<string> ColourKey=new();public bool HasColours=>ColourKey.Count>0;public string Text{get;set;}')
foot=foot.replace('var cases=new List<object>();','var cases=new List<object>();var draws=new List<object>();')
foot=foot.replace('cases.Add(new{title,detail,maxWidth,fixedPosition,output=JsonSerializer.SerializeToElement(output)});','cases.Add(new{title,detail,maxWidth,fixedPosition,output=JsonSerializer.SerializeToElement(output)});if(title=="Short"&&maxWidth==480&&!fixedPosition){foreach(var alignment in new[]{0,1,2})foreach(var shape in new[]{0,1})foreach(var border in new[]{0,3,10}){foreach(var line in output.Lines)line.Alignment=(Alignment)alignment;output.shape=(NotificationItemShape)shape;output.BorderWidth=border;var g=new Graphics();output.Draw(g,q);draws.Add(new{detail,alignment,shape,border,lines=JsonSerializer.SerializeToElement(output.Lines),dimensions=output.Dimensions,ops=g.Ops});}}')
foot=foot.replace('new{cases,references}', 'new{cases,references,draws}')
root=Path(__file__).resolve().parents[2];(root/'native/fixtures/stellar-notification-layout-oracle.cs').write_text(head+body+'\n'+method('public void InitialiseFontHeight(Graphics gfx, int numqueues)')+'\n'+method('private float getNotificationHeight()')+'\n'+'\n'.join(helpers)+foot)
