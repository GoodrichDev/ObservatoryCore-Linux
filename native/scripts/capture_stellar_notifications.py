#!/usr/bin/env python3
"""Capture actual source OnNotificationEvent routing without loading its DLL."""
import pathlib, json
base=pathlib.Path('/home/loic/.cache/observatory-readable/stellar/StellarOverlay/OverlayWorker.cs')
source=base.read_text()
def method(signature):
    start=source.index(signature);brace=source.index('{',start);depth=1;end=brace+1
    while depth:
        if source[end]=='{':depth+=1
        elif source[end]=='}':depth-=1
        end+=1
    return source[start:end]
body=method('public void OnNotificationEvent(NotificationArgs noti)')
helper=method('static bool _202E_200D_202C_200E_202D_200C_206D_202A_206B_202E_206A_200D_200F_200C_202E_200B_206B_200B_202D_206E_206F_202A_206A_200E_200F_202E_200E_206D_206F_202C_202C_206D_202E_200B_200E_200C_206C_202C_200F_206D_202E(')
head='''// Source OnNotificationEvent copied verbatim. Only data/queue/log stubs are authored.
using System; using System.Collections.Generic; using System.Runtime.CompilerServices; using System.Text.Json;
[Flags] public enum NotificationRendering { Popup=1,Voice=2,Plugin=4 }
public class NotificationArgs {public NotificationRendering Rendering;public Guid? Guid;public int Timeout;public double XPos,YPos;}
public class Settings {public bool HandleObservatoryNotifications=true;public int TTL=5000;}
public class QueueStub {public List<NotificationArgs> Queue=new();public bool Known;public bool InstantItemsHasKey(Guid key){return Known;}}
public class OverlayStub {public QueueStub NotificationQueue=new();}
public class LogStub {public void WriteException(string s,Exception e){throw e;}}
public class OverlayWorker {public Settings settings=new();public OverlayStub Overlay=new();public List<string> DebugLog=new();public LogStub logger=new();public List<NotificationArgs> Instant=new();public void AddInstantNotification(NotificationArgs n){Instant.Add(n);}
'''
foot='''
}
public class Program {public static void Main(){var cases=new List<object>();foreach(var guid in new[]{"none","empty","valid"})foreach(var ttl in new[]{-1,0,1,5000})foreach(var x in new[]{-1.0,0.0,50.0})foreach(var y in new[]{-1.0,0.0,50.0})foreach(var rendering in new[]{0,1,2,3,4,5,6,7})foreach(var handle in new[]{false,true}){var w=new OverlayWorker();w.settings.HandleObservatoryNotifications=handle;var n=new NotificationArgs{Guid=guid=="none"?null:guid=="empty"?System.Guid.Empty:System.Guid.Parse("11111111-1111-1111-1111-111111111111"),Timeout=ttl,XPos=x,YPos=y,Rendering=(NotificationRendering)rendering};w.OnNotificationEvent(n);cases.Add(new{guid,ttl,x,y,rendering,handle,output=w.Instant.Count>0?"instant":w.Overlay.NotificationQueue.Queue.Count>0?"queue":"ignored",result_ttl=n.Timeout});}var updates=new List<object>();foreach(var known in new[]{false,true})foreach(var rendering in new[]{0,1,2,3,4,5,6,7})foreach(var handle in new[]{false,true}){var w=new OverlayWorker();w.settings.HandleObservatoryNotifications=handle;w.Overlay.NotificationQueue.Known=known;w.UpdateNotification(new NotificationArgs{Guid=System.Guid.Parse("11111111-1111-1111-1111-111111111111"),Rendering=(NotificationRendering)rendering});updates.Add(new{known,rendering,handle,instant=w.Instant.Count>0});}Console.WriteLine(JsonSerializer.Serialize(new{cases,updates}));}}
'''
root=pathlib.Path(__file__).resolve().parents[2];(root/'native/fixtures/stellar-notification-routing-oracle.cs').write_text(head+body+'\n'+method('public void UpdateNotification(NotificationArgs notificationEventArgs)')+'\n'+helper+foot)
