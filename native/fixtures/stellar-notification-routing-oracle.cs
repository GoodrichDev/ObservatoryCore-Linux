// Source OnNotificationEvent copied verbatim. Only data/queue/log stubs are authored.
using System; using System.Collections.Generic; using System.Runtime.CompilerServices; using System.Text.Json;
[Flags] public enum NotificationRendering { Popup=1,Voice=2,Plugin=4 }
public class NotificationArgs {public NotificationRendering Rendering;public Guid? Guid;public int Timeout;public double XPos,YPos;}
public class Settings {public bool HandleObservatoryNotifications=true;public int TTL=5000;}
public class QueueStub {public List<NotificationArgs> Queue=new();public bool Known;public bool InstantItemsHasKey(Guid key){return Known;}}
public class OverlayStub {public QueueStub NotificationQueue=new();}
public class LogStub {public void WriteException(string s,Exception e){throw e;}}
public class OverlayWorker {public Settings settings=new();public OverlayStub Overlay=new();public List<string> DebugLog=new();public LogStub logger=new();public List<NotificationArgs> Instant=new();public void AddInstantNotification(NotificationArgs n){Instant.Add(n);}
public void OnNotificationEvent(NotificationArgs noti)
	{
		//IL_0011: Unknown result type (might be due to invalid IL or missing references)
		if (!settings.HandleObservatoryNotifications)
		{
			return;
		}
		try
		{
			if (!_202E_200D_202C_200E_202D_200C_206D_202A_206B_202E_206A_200D_200F_200C_202E_200B_206B_200B_202D_206E_206F_202A_206A_200E_200F_202E_200E_206D_206F_202C_202C_206D_202E_200B_200E_200C_206C_202C_200F_206D_202E((Enum)(object)noti.Rendering, (Enum)(object)(NotificationRendering)1))
			{
				return;
			}
			Guid? guid = default;
			Guid empty = default;
			while (true)
			{
				int num = 1396133495;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ 0xAFB78A5)) % 20)
					{
					case 0u:
						break;
					default:
						return;
					case 4u:
						guid = noti.Guid;
						num = (int)((num2 * 1947407924) ^ 0x7007F9E4);
						continue;
					case 18u:
						num = ((int)num2 * -570120849) ^ -1409247325;
						continue;
					case 5u:
						empty = Guid.Empty;
						num = (int)((num2 * 810118843) ^ 0x5E66D9AF);
						continue;
					case 10u:
						Overlay.NotificationQueue.Queue.Add(noti);
						num = 1205596515;
						continue;
					case 3u:
						AddInstantNotification(noti);
						num = 1301922941;
						continue;
					case 7u:
						noti.Timeout = settings.TTL * 4;
						AddInstantNotification(noti);
						num = 1301922941;
						continue;
					case 2u:
					{
						List<string> debugLog = DebugLog;
						DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(49, 4);
						defaultInterpolatedStringHandler.AppendLiteral("[OnNotificationEvent] <GUID<");
						defaultInterpolatedStringHandler.AppendFormatted(noti.Guid);
						defaultInterpolatedStringHandler.AppendLiteral(">, <TTL:");
						defaultInterpolatedStringHandler.AppendFormatted(noti.Timeout);
						defaultInterpolatedStringHandler.AppendLiteral(">, <X:");
						defaultInterpolatedStringHandler.AppendFormatted(noti.XPos);
						defaultInterpolatedStringHandler.AppendLiteral(">, <Y:");
						defaultInterpolatedStringHandler.AppendFormatted(noti.YPos);
						defaultInterpolatedStringHandler.AppendLiteral(">");
						debugLog.Add(defaultInterpolatedStringHandler.ToStringAndClear());
						num = 312005138;
						continue;
					}
					case 19u:
					{
						int num9;
						int num10;
						if (noti.YPos != -1.0)
						{
							num9 = -599428377;
							num10 = num9;
						}
						else
						{
							num9 = -485561274;
							num10 = num9;
						}
						num = num9 ^ (int)(num2 * 23455951);
						continue;
					}
					case 12u:
						AddInstantNotification(noti);
						num = (int)(num2 * 464634971) ^ -1714375373;
						continue;
					case 17u:
						DebugLog.RemoveAt(0);
						num = (int)((num2 * 832100505) ^ 0x7351D0AD);
						continue;
					case 16u:
					{
						int num5;
						int num6;
						if (noti.Guid.HasValue)
						{
							num5 = -2147453715;
							num6 = num5;
						}
						else
						{
							num5 = -1823000018;
							num6 = num5;
						}
						num = num5 ^ ((int)num2 * -256034596);
						continue;
					}
					case 1u:
						if (!guid.HasValue)
						{
							num = (int)((num2 * 212686157) ^ 0x7C62697F);
							continue;
						}
						if (guid.GetValueOrDefault() == empty)
						{
							num = 314357262;
							continue;
						}
						goto case 9u;
					case 6u:
					{
						int num8;
						if (noti.XPos != -1.0)
						{
							num = 1540081454;
							num8 = num;
						}
						else
						{
							num = 333002019;
							num8 = num;
						}
						continue;
					}
					case 9u:
					case 15u:
					{
						int num11;
						if (noti.Timeout == 0)
						{
							num = 1628318553;
							num11 = num;
						}
						else
						{
							num = 2089863343;
							num11 = num;
						}
						continue;
					}
					case 13u:
					{
						int num7;
						if (noti.Timeout != 0)
						{
							num = 748133536;
							num7 = num;
						}
						else
						{
							num = 1954253213;
							num7 = num;
						}
						continue;
					}
					case 14u:
						num = ((int)num2 * -689763100) ^ -1416087515;
						continue;
					case 11u:
					{
						int num3;
						int num4;
						if (DebugLog.Count <= 50)
						{
							num3 = -90008658;
							num4 = num3;
						}
						else
						{
							num3 = -1881833666;
							num4 = num3;
						}
						num = num3 ^ (int)(num2 * 351854306);
						continue;
					}
					case 8u:
						return;
					}
					break;
				}
			}
		}
		catch (Exception ex)
		{
			logger.WriteException("[OnNotificationEvent] Exception: ", ex);
		}
	}
public void UpdateNotification(NotificationArgs notificationEventArgs)
	{
		//IL_001d: Unknown result type (might be due to invalid IL or missing references)
		Guid value = notificationEventArgs.Guid.Value;
		if (!settings.HandleObservatoryNotifications)
		{
			return;
		}
		try
		{
			if (!_202E_200D_202C_200E_202D_200C_206D_202A_206B_202E_206A_200D_200F_200C_202E_200B_206B_200B_202D_206E_206F_202A_206A_200E_200F_202E_200E_206D_206F_202C_202C_206D_202E_200B_200E_200C_206C_202C_200F_206D_202E((Enum)(object)notificationEventArgs.Rendering, (Enum)(object)(NotificationRendering)1))
			{
				return;
			}
			while (true)
			{
				int num = 1687460585;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ 0x452E0D5F)) % 7)
					{
					case 0u:
						break;
					default:
						return;
					case 3u:
						AddInstantNotification(notificationEventArgs);
						num = ((int)num2 * -727108051) ^ -879347863;
						continue;
					case 2u:
						DebugLog.RemoveAt(0);
						num = ((int)num2 * -1905887837) ^ 0x5BC100C4;
						continue;
					case 6u:
					{
						int num4;
						int num5;
						if (DebugLog.Count > 50)
						{
							num4 = -1627478373;
							num5 = num4;
						}
						else
						{
							num4 = -448356344;
							num5 = num4;
						}
						num = num4 ^ ((int)num2 * -1820992328);
						continue;
					}
					case 1u:
					{
						List<string> debugLog = DebugLog;
						DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(48, 4);
						defaultInterpolatedStringHandler.AppendLiteral("[UpdateNotification] <GUID<");
						defaultInterpolatedStringHandler.AppendFormatted(notificationEventArgs.Guid);
						defaultInterpolatedStringHandler.AppendLiteral(">, <TTL:");
						defaultInterpolatedStringHandler.AppendFormatted(notificationEventArgs.Timeout);
						defaultInterpolatedStringHandler.AppendLiteral(">, <X:");
						defaultInterpolatedStringHandler.AppendFormatted(notificationEventArgs.XPos);
						defaultInterpolatedStringHandler.AppendLiteral(">, <Y:");
						defaultInterpolatedStringHandler.AppendFormatted(notificationEventArgs.YPos);
						defaultInterpolatedStringHandler.AppendLiteral(">");
						debugLog.Add(defaultInterpolatedStringHandler.ToStringAndClear());
						num = 352317046;
						continue;
					}
					case 4u:
					{
						int num3;
						if (!Overlay.NotificationQueue.InstantItemsHasKey(value))
						{
							num = 860534301;
							num3 = num;
						}
						else
						{
							num = 864063131;
							num3 = num;
						}
						continue;
					}
					case 5u:
						return;
					}
					break;
				}
			}
		}
		catch (Exception ex)
		{
			logger.WriteException("[UpdateNotification] Exception: ", ex);
		}
	}
static bool _202E_200D_202C_200E_202D_200C_206D_202A_206B_202E_206A_200D_200F_200C_202E_200B_206B_200B_202D_206E_206F_202A_206A_200E_200F_202E_200E_206D_206F_202C_202C_206D_202E_200B_200E_200C_206C_202C_200F_206D_202E(Enum P_0, Enum P_1)
	{
		return P_0.HasFlag(P_1);
	}
}
public class Program {public static void Main(){var cases=new List<object>();foreach(var guid in new[]{"none","empty","valid"})foreach(var ttl in new[]{-1,0,1,5000})foreach(var x in new[]{-1.0,0.0,50.0})foreach(var y in new[]{-1.0,0.0,50.0})foreach(var rendering in new[]{0,1,2,3,4,5,6,7})foreach(var handle in new[]{false,true}){var w=new OverlayWorker();w.settings.HandleObservatoryNotifications=handle;var n=new NotificationArgs{Guid=guid=="none"?null:guid=="empty"?System.Guid.Empty:System.Guid.Parse("11111111-1111-1111-1111-111111111111"),Timeout=ttl,XPos=x,YPos=y,Rendering=(NotificationRendering)rendering};w.OnNotificationEvent(n);cases.Add(new{guid,ttl,x,y,rendering,handle,output=w.Instant.Count>0?"instant":w.Overlay.NotificationQueue.Queue.Count>0?"queue":"ignored",result_ttl=n.Timeout});}var updates=new List<object>();foreach(var known in new[]{false,true})foreach(var rendering in new[]{0,1,2,3,4,5,6,7})foreach(var handle in new[]{false,true}){var w=new OverlayWorker();w.settings.HandleObservatoryNotifications=handle;w.Overlay.NotificationQueue.Known=known;w.UpdateNotification(new NotificationArgs{Guid=System.Guid.Parse("11111111-1111-1111-1111-111111111111"),Rendering=(NotificationRendering)rendering});updates.Add(new{known,rendering,handle,instant=w.Instant.Count>0});}Console.WriteLine(JsonSerializer.Serialize(new{cases,updates}));}}
