
using System;using System.Linq;using System.Collections.Generic;using System.Collections.Immutable;using System.Reflection;using System.Runtime.CompilerServices;using System.Text.Json;
using EDJP.Observatory;using Observatory.Framework;using Observatory.Framework.Files.Journal;using Observatory.Framework.Files.ParameterTypes;using Observatory.Framework.Interfaces;
namespace Observatory.Framework.Files.Journal{
 public class JournalBase { public int BodyID=1,BodyCount=5,ProbesUsed=2,EfficiencyTarget=6;public ulong SystemAddress=42;public string Commander="Source",FID="F123",JumpType="Hyperspace",StarSystem="Source",Event="test",Json="{}",BodyName="Source 1",PlanetClass="Earthlike body",StarType="",AtmosphereType="None";public bool Odyssey=true,InSRV,OnFoot,SRV,Docked,Landable;public float SurfacePressure;public DateTime TimestampDateTime=DateTime.UnixEpoch;public ImmutableList<Signal> Signals=ImmutableList<Signal>.Empty;}
public class LoadGame:JournalBase {}
public class Location:JournalBase {}
public class FSDJump:JournalBase {}
public class CarrierJump:FSDJump {}
public class StartJump:JournalBase {}
public class Scan:ScanBaryCentre {}
public class ScanBaryCentre:JournalBase {}
public class FSSBodySignals:SAASignalsFound {}
public class SAASignalsFound:JournalBase {}
public class SAAScanComplete:JournalBase {}
public class FSSAllBodiesFound:JournalBase {}
public class FSSDiscoveryScan:JournalBase {}
public class ApproachBody:JournalBase {}
public class SupercruiseExit:SupercruiseEntry {}
public class SupercruiseEntry:JournalBase {}
public class Embark:Disembark {}
public class Disembark:JournalBase {}
public class ScanOrganic:JournalBase {}
public class Died:JournalBase {}
public class Shutdown:JournalBase {}
public class SellExplorationData:JournalBase {}
public class MultiSellExplorationData:JournalBase {}
}
namespace Observatory.Framework.Files.ParameterTypes {public class Signal{public string Type="";public int Count;}}
namespace Observatory.Framework {public class NotificationArgs{public Guid Guid;public int Timeout;public int XPos,YPos;public string Title,Detail,TitleSsml,DetailSsml;public int CoalescingId;public string Sender;public NotificationRendering Rendering;}public enum NotificationRendering{Native=1,Voice=2,Plugin=4}public class PluginMessage{public string Command;public Dictionary<string,object>Payload;public PluginMessage(string s,Dictionary<string,object>d,Guid?g){Command=s;Payload=d;}}}
namespace Observatory.Framework.Interfaces {public interface IObservatoryPlugin{}public interface IObservatoryCore{bool IsLogMonitorBatchReading{get;}Guid SendNotification(NotificationArgs a);void CancelNotification(Guid g);void UpdateNotification(NotificationArgs n);void SendPluginMessage(IObservatoryPlugin p,Guid g,PluginMessage m);}}
namespace EDJP.Observatory {public class Logger{public void WriteLog(string s,LogLevel l=LogLevel.Verbose){}public void WriteException(string s,Exception e){throw new Exception(s,e);}}public enum LogLevel{Verbose,Debug}public static class PluginID{public static Guid Hecate=Guid.Empty;}public static class Utils{public static string NotificationBodyName(string s,string n,bool isTTS=false,bool includeSSMLtag=false)=>s;public static string SSML(string s)=>s;public static string SSMLify(string s)=>s;}}
public class CoreStub:IObservatoryCore{public List<string>Trace=new();public bool IsLogMonitorBatchReading{get;set;}public void CancelNotification(Guid g){Trace.Add("CancelNotification");}public void UpdateNotification(NotificationArgs n){Trace.Add("UpdateNotification:"+n.Detail);}public Guid SendNotification(NotificationArgs n){Trace.Add("Notify:"+n.Title+":"+n.Detail);return new Guid("00000000-0000-0000-0000-000000000001");}public void SendPluginMessage(IObservatoryPlugin p,Guid g,PluginMessage m){Trace.Add("Message:"+m.Command);}}
public class TmpCarrierJumpFlags{public bool Docked{get;set;} public bool OnFoot{get;set;}}
public enum TransportMode{Ship,SRV,OnFoot}
public class EvalSettings {public bool SetTargetedInFSSWorthMapping=true;public Notifications Notifications=new();}
public class Notifications {public int PersistentPositionX=80,PersistentPositionY=25;public Category AlwaysPersistent=new(),DSSPersistent=new(),BodiesWorthMapping=new(),EfficientRoute=new();}
public class Category {public bool Popup=true,Any=true;public NotificationRendering Rendering=NotificationRendering.Native;}
public class Body {public bool IsStar;public bool? OverrideIsWorthMapping;public bool WorthMapping=true,Mapped,IsOdyssey,EfficiencyBonus,IsMapped,IsMappedEfficiently;public string BodyName="Source 1";public void UpdateValues(){}public int Value=1000;}
public class EDSystem {public Dictionary<int,Body>Bodies=new(){{1,new Body()}};public Dictionary<int,object>Extra=new();public bool Active=true;public ulong SystemAddress=42;public string Name="Source";public int TotalBodies;public int FullyMappedBonus=10000;public bool AddBody(Scan s,bool o){Bodies.TryAdd(s.BodyID,new Body());return true;}public void SetBioGeocount(int i,int?b,int?g){}}
public class Orrery{public HashSet<int>GetMissingBodies=new();}public class UIHandler {public void HandleGuiStatusChange(FocusStatus f){}public void SetCommander(string f,string n){}}
public enum FocusStatus{None=0}public class Destination{public int Body;public ulong System;}public class Status{public FocusStatus GuiFocus;public Destination Destination;}
public class Evaluator:IObservatoryPlugin {
public void UpdatePersistentNotification(bool onlyUpdate=false)
	{
		//IL_034d: Unknown result type (might be due to invalid IL or missing references)
		string detail = " - none - ";
		string text2 = default;
		int num7 = default;
		string text = default;
		int num5 = default;
		NotificationArgs val = default;
		Guid empty = default;
		Guid? guid = default;
		while (true)
		{
			int num = -911900623;
			while (true)
			{
				uint num2;
				int num14;
				NotificationArgs val2;
				switch ((num2 = (uint)(num ^ -360542393)) % 4)
				{
				case 3u:
					break;
				case 2u:
					if (nextToVisit.Count > 0)
					{
						num = ((int)num2 * -1438501490) ^ 0x608E12FA;
						continue;
					}
					goto IL_032d;
				case 1u:
					text2 = "";
					num7 = 0;
					num = ((int)num2 * -1399661381) ^ -339294152;
					continue;
				default:
					{
						using (List<int>.Enumerator enumerator = nextToVisit.GetEnumerator())
						{
							while (true)
							{
								IL_0192:
								int num3;
								int num4;
								if (enumerator.MoveNext())
								{
									num3 = -778173398;
									num4 = num3;
								}
								else
								{
									num3 = -1475286652;
									num4 = num3;
								}
								while (true)
								{
									switch ((num2 = (uint)(num3 ^ -360542393)) % 10)
									{
									case 7u:
										num3 = -778173398;
										continue;
									default:
										goto end_IL_0084;
									case 2u:
										text2 = _202E_206A_206B_202E_206A_200C_206C_202D_206D_206A_202C_200B_206F_200D_200E_206F_206E_202C_206F_206D_206B_206F_200C_202E_206D_200F_200F_200E_202A_202B_200B_200D_206C_206A_206F_200D_200B_200B_200D_202D_202E(text2, "\n     ");
										num3 = -1238871405;
										continue;
									case 0u:
										text2 = _200F_206F_202C_206E_202A_206D_206A_200F_202C_200F_206F_206A_206D_202D_202E_206D_202D_200E_206E_200E_202D_206C_200D_202A_200E_206B_200B_206B_202C_202C_200E_202B_206A_206F_202E_200F_200D_206A_206F_200F_202E(text2, text, ", ");
										num7++;
										num3 = -1350799261;
										continue;
									case 6u:
									{
										int num12;
										int num13;
										if (!current.Bodies.ContainsKey(num5))
										{
											num12 = 2031347021;
											num13 = num12;
										}
										else
										{
											num12 = 1074048335;
											num13 = num12;
										}
										num3 = num12 ^ (int)(num2 * 1237800447);
										continue;
									}
									case 1u:
									{
										int num8;
										int num9;
										if (num7 <= 0)
										{
											num8 = -1326087839;
											num9 = num8;
										}
										else
										{
											num8 = -1954448508;
											num9 = num8;
										}
										num3 = num8 ^ (int)(num2 * 1471849570);
										continue;
									}
									case 3u:
									{
										int num10;
										int num11;
										if (num7 % 6 == 0)
										{
											num10 = -596823758;
											num11 = num10;
										}
										else
										{
											num10 = -471924734;
											num11 = num10;
										}
										num3 = num10 ^ (int)(num2 * 118868321);
										continue;
									}
									case 5u:
									{
										num5 = enumerator.Current;
										int num6;
										if (num5 <= -1)
										{
											num3 = -1350799261;
											num6 = num3;
										}
										else
										{
											num3 = -516261483;
											num6 = num3;
										}
										continue;
									}
									case 4u:
										break;
									case 8u:
										text = _202C_206B_200E_206C_202E_202D_202D_200D_202D_206D_200F_202E_206D_202D_206F_206F_202D_200B_202E_200E_206C_206F_206B_200D_200D_200B_202D_200D_206B_206E_200D_200D_200E_206F_206D_200F_200B_206D_206C_202C_202E(_206C_206E_206D_206C_200D_200D_202A_206D_206F_206B_202C_202E_206D_206B_202D_206D_202A_200C_200B_200B_200C_202D_202D_200D_200D_200C_200B_206F_200E_206B_202D_200C_200C_200D_200E_200F_200B_200D_200E_202E_202E(current.Bodies[num5].BodyName, current.Name, ""));
										num3 = (int)(num2 * 410141842) ^ -1326473566;
										continue;
									case 9u:
										goto end_IL_0084;
									}
									goto IL_0192;
									continue;
									end_IL_0084:
									break;
								}
								break;
							}
						}
						text2 = _202A_206D_202D_206E_206E_206E_200F_202B_206B_206A_202E_206A_206E_202C_202B_202A_200D_206E_206B_200D_200B_200F_202A_206C_200D_202D_202A_202B_206E_206E_200E_206F_206D_202D_200B_202E_206C_200E_200D_202D_202E(text2, new char[2] { ' ', ',' });
						goto IL_021c;
					}
					IL_0221:
					while (true)
					{
						switch ((num2 = (uint)(num14 ^ -360542393)) % 13)
						{
						case 6u:
							break;
						default:
							return;
						case 8u:
							return;
						case 12u:
							goto IL_027c;
						case 1u:
						case 7u:
							goto IL_02ad;
						case 5u:
							nextBodyNotification = _200C_200E_206A_200D_202B_202D_200E_200B_202B_206A_200D_200B_200D_200B_200F_206A_206F_206B_202D_200E_202A_200E_206C_202A_206D_200B_200B_200C_202C_200E_202B_200C_206D_206F_200C_200F_200E_206A_200C_202E(Core, val);
							num14 = ((int)num2 * -1764185448) ^ -969165324;
							continue;
						case 2u:
						{
							int num15;
							int num16;
							if (!nextBodyNotification.HasValue)
							{
								num15 = -1001426134;
								num16 = num15;
							}
							else
							{
								num15 = -1402243849;
								num16 = num15;
							}
							num14 = num15 ^ ((int)num2 * -1242248192);
							continue;
						}
						case 11u:
							goto IL_032d;
						case 10u:
							empty = Guid.Empty;
							num14 = ((int)num2 * -1674129148) ^ -1160179128;
							continue;
						case 3u:
							guid = nextBodyNotification;
							num14 = (int)(num2 * 1993673677) ^ -1018604443;
							continue;
						case 4u:
							detail = _202E_206A_206B_202E_206A_200C_206C_202D_206D_206A_202C_200B_206F_200D_200E_206F_206E_202C_206F_206D_206B_206F_200C_202E_206D_200F_200F_200E_202A_202B_200B_200D_206C_206A_206F_200D_200B_200B_200D_202D_202E("⋙ ", text2);
							num14 = -550729697;
							continue;
						case 0u:
							goto IL_03f0;
						case 9u:
							return;
						}
						break;
						IL_03f0:
						if (guid.HasValue)
						{
							if (guid.GetValueOrDefault() == empty)
							{
								num14 = -1472812246;
								continue;
							}
							goto IL_02ad;
						}
						num14 = ((int)num2 * -266876809) ^ -154587821;
						continue;
						IL_027c:
						int num17;
						if (!onlyUpdate)
						{
							num14 = -23711771;
							num17 = num14;
						}
						else
						{
							num14 = -1547355785;
							num17 = num14;
						}
						continue;
						IL_02ad:
						val.Guid = nextBodyNotification.Value;
						_202C_200D_202D_202A_202E_200E_200F_202E_206E_206C_202C_206A_200D_200B_206D_202C_202D_202D_206A_202B_200D_200E_200C_202A_200C_200D_202C_206A_206B_200E_200E_206F_206A_200F_202D_202D_206C_206A_206D_206C_202E(Core, val);
						num14 = -1547355785;
					}
					goto IL_021c;
					IL_021c:
					num14 = -1596469992;
					goto IL_0221;
					IL_032d:
					val2 = _206D_200B_202C_202D_206C_200C_206D_202E_202A_200C_206C_206E_206C_200E_200F_206C_200C_200E_202D_200C_206F_200E_202C_200D_202D_200D_200E_206F_206B_206B_202E_202D_202E_200C_206E_200E_200F_206F_200E_200D_202E();
					val2.Title = "Bodies still to map";
					val2.Detail = detail;
					val2.Rendering = (NotificationRendering)1;
					val2.Timeout = 0;
					val2.XPos = settings.Notifications.PersistentPositionX;
					val2.YPos = settings.Notifications.PersistentPositionY;
					val2.Sender = ShortName;
					val = val2;
					num14 = -465492747;
					goto IL_0221;
				}
				break;
			}
		}
	}
private void ClosePersistentNotification()
	{
		if (!nextBodyNotification.HasValue)
		{
			return;
		}
		while (true)
		{
			int num = -1293377453;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -1494816934)) % 4)
				{
				case 3u:
					break;
				default:
					return;
				case 1u:
					_200E_200C_202E_206D_206C_202A_202C_202A_206C_202B_206B_206B_202E_206B_200D_202D_202B_200C_200F_206C_200C_206C_206C_206D_206D_206E_206F_200E_200C_206C_200C_200C_202C_200D_202C_202A_202E_206F_200C_206D_202E(Core, nextBodyNotification.Value);
					num = (int)((num2 * 182815450) ^ 0xE65A890);
					continue;
				case 0u:
					nextBodyNotification = null;
					num = ((int)num2 * -1212081396) ^ -249624076;
					continue;
				case 2u:
					return;
				}
				break;
			}
		}
	}
static string _202A_206D_202D_206E_206E_206E_200F_202B_206B_206A_202E_206A_206E_202C_202B_202A_200D_206E_206B_200D_200B_200F_202A_206C_200D_202D_202A_202B_206E_206E_200E_206F_206D_202D_200B_202E_206C_200E_200D_202D_202E(string P_0, char[] P_1)
	{
		return P_0.TrimEnd(P_1);
	}
static string _202C_206B_200E_206C_202E_202D_202D_200D_202D_206D_200F_202E_206D_202D_206F_206F_202D_200B_202E_200E_206C_206F_206B_200D_200D_200B_202D_200D_206B_206E_200D_200D_200E_206F_206D_200F_200B_206D_206C_202C_202E(string P_0)
	{
		return P_0.TrimStart();
	}
static void _200E_200C_202E_206D_206C_202A_202C_202A_206C_202B_206B_206B_202E_206B_200D_202D_202B_200C_200F_206C_200C_206C_206C_206D_206D_206E_206F_200E_200C_206C_200C_200C_202C_200D_202C_202A_202E_206F_200C_206D_202E(IObservatoryCore P_0, Guid P_1)
	{
		P_0.CancelNotification(P_1);
	}
static void _202C_200D_202D_202A_202E_200E_200F_202E_206E_206C_202C_206A_200D_200B_206D_202C_202D_202D_206A_202B_200D_200E_200C_202A_200C_200D_202C_206A_206B_200E_200E_206F_206A_200F_202D_202D_206C_206A_206D_206C_202E(IObservatoryCore P_0, NotificationArgs P_1)
	{
		P_0.UpdateNotification(P_1);
	}
public void StatusChange(Status status)
	{
		//IL_0195: Unknown result type (might be due to invalid IL or missing references)
		//IL_019b: Unknown result type (might be due to invalid IL or missing references)
		//IL_00e8: Unknown result type (might be due to invalid IL or missing references)
		//IL_00ef: Invalid comparison between Unknown and I4
		//IL_035c: Unknown result type (might be due to invalid IL or missing references)
		//IL_0363: Invalid comparison between Unknown and I4
		//IL_0327: Unknown result type (might be due to invalid IL or missing references)
		//IL_037b: Unknown result type (might be due to invalid IL or missing references)
		//IL_0380: Unknown result type (might be due to invalid IL or missing references)
		//IL_03d4: Unknown result type (might be due to invalid IL or missing references)
		//IL_03db: Invalid comparison between Unknown and I4
		//IL_0215: Unknown result type (might be due to invalid IL or missing references)
		//IL_021c: Invalid comparison between Unknown and I4
		try
		{
			if (settings.SetTargetedInFSSWorthMapping)
			{
				goto IL_0010;
			}
			goto IL_0140;
			IL_0010:
			int num = 534004698;
			goto IL_0015;
			IL_0015:
			int key = default;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0x3A2BC78F)) % 25)
				{
				case 0u:
					break;
				default:
					return;
				case 8u:
				{
					int num19;
					int num20;
					if (nextBodyNotification.HasValue)
					{
						num19 = -320655971;
						num20 = num19;
					}
					else
					{
						num19 = -797689309;
						num20 = num19;
					}
					num = num19 ^ (int)(num2 * 2064109627);
					continue;
				}
				case 1u:
					ClosePersistentNotification();
					num = ((int)num2 * -1428124037) ^ -1789872505;
					continue;
				case 18u:
					ClosePersistentNotification();
					num = (int)((num2 * 1696817633) ^ 0x2BE591AC);
					continue;
				case 7u:
				{
					int num13;
					int num14;
					if ((int)_206C_206A_206E_200C_206E_206A_206F_202D_200E_202E_202D_202C_206E_202E_200B_202A_202D_206B_200F_206E_200F_206A_200C_202A_206B_202E_200F_200C_202B_200C_202A_206F_206D_200B_206D_200E_206B_202E_206A_200C_202E(status) != 10)
					{
						num13 = 1968512695;
						num14 = num13;
					}
					else
					{
						num13 = 1385753410;
						num14 = num13;
					}
					num = num13 ^ ((int)num2 * -1686634738);
					continue;
				}
				case 20u:
				{
					int num9;
					int num10;
					if (settings.Notifications.AlwaysPersistent.Popup)
					{
						num9 = -24691151;
						num10 = num9;
					}
					else
					{
						num9 = -1358047374;
						num10 = num9;
					}
					num = num9 ^ (int)(num2 * 198185259);
					continue;
				}
				case 13u:
					goto IL_0140;
				case 10u:
				{
					int num7;
					int num8;
					if (_202E_200F_202B_206C_202B_206F_202B_206A_206B_202C_202B_202A_206E_206D_202C_206B_202C_200E_206B_200D_200D_202E_202E_200F_200C_202A_206F_200C_206F_200E_200D_202C_202C_202C_202A_206C_200B_200F_200E_200D_202E(_206F_206C_200B_202E_206B_202C_202A_202D_206A_202D_206E_200C_200C_206C_206C_200E_206F_202C_200E_200E_202B_206E_206F_206A_202C_206A_200E_202B_206B_200E_202D_200D_200D_200E_206B_206A_200E_206C_202C_206E_202E(status)) != 0)
					{
						num7 = -1367061498;
						num8 = num7;
					}
					else
					{
						num7 = -22239174;
						num8 = num7;
					}
					num = num7 ^ (int)(num2 * 1681739173);
					continue;
				}
				case 4u:
					goto IL_0194;
				case 19u:
				{
					int num15;
					int num16;
					if (nextToVisit.Count <= 0)
					{
						num15 = -582925243;
						num16 = num15;
					}
					else
					{
						num15 = -649724971;
						num16 = num15;
					}
					num = num15 ^ (int)(num2 * 463822656);
					continue;
				}
				case 15u:
				{
					int num25;
					int num26;
					if (current.Bodies[key].IsStar)
					{
						num25 = 1035614448;
						num26 = num25;
					}
					else
					{
						num25 = 1915990902;
						num26 = num25;
					}
					num = num25 ^ ((int)num2 * -513214822);
					continue;
				}
				case 24u:
				{
					int num21;
					int num22;
					if ((int)_206C_206A_206E_200C_206E_206A_206F_202D_200E_202E_202D_202C_206E_202E_200B_202A_202D_206B_200F_206E_200F_206A_200C_202A_206B_202E_200F_200C_202B_200C_202A_206F_206D_200B_206D_200E_206B_202E_206A_200C_202E(status) != 9)
					{
						num21 = 726092955;
						num22 = num21;
					}
					else
					{
						num21 = 1597682751;
						num22 = num21;
					}
					num = num21 ^ ((int)num2 * -320414657);
					continue;
				}
				case 23u:
					UpdateUI();
					num = (int)((num2 * 1162781193) ^ 0x7A2562EE);
					continue;
				case 5u:
				{
					key = _202E_200F_202B_206C_202B_206F_202B_206A_206B_202C_202B_202A_206E_206D_202C_206B_202C_200E_206B_200D_200D_202E_202E_200F_200C_202A_206F_200C_206F_200E_200D_202C_202C_202C_202A_206C_200B_200F_200E_200D_202E(_206F_206C_200B_202E_206B_202C_202A_202D_206A_202D_206E_200C_200C_206C_206C_200E_206F_202C_200E_200E_202B_206E_206F_206A_202C_206A_200E_202B_206B_200E_202D_200D_200D_200E_206B_206A_200E_206C_202C_206E_202E(status));
					int num5;
					int num6;
					if (!current.Bodies.ContainsKey(key))
					{
						num5 = -1591271582;
						num6 = num5;
					}
					else
					{
						num5 = -580309539;
						num6 = num5;
					}
					num = num5 ^ ((int)num2 * -1130765454);
					continue;
				}
				case 16u:
				{
					int num23;
					int num24;
					if (nextToVisit.Count != 0)
					{
						num23 = -837122016;
						num24 = num23;
					}
					else
					{
						num23 = -1378381230;
						num24 = num23;
					}
					num = num23 ^ ((int)num2 * -55532586);
					continue;
				}
				case 6u:
				{
					int num17;
					int num18;
					if (_200E_206F_202B_206A_200F_202B_202D_202B_206B_206B_202C_200B_206C_202B_206A_202E_200C_202D_200E_206A_202A_202C_206D_206C_206F_206E_206F_200F_202D_200B_202A_206A_206B_202D_200E_206D_206D_202C_202A_200E_202E(_206F_206C_200B_202E_206B_202C_202A_202D_206A_202D_206E_200C_200C_206C_206C_200E_206F_202C_200E_200E_202B_206E_206F_206A_202C_206A_200E_202B_206B_200E_202D_200D_200D_200E_206B_206A_200E_206C_202C_206E_202E(status)) != current.SystemAddress)
					{
						num17 = 411353292;
						num18 = num17;
					}
					else
					{
						num17 = 11115362;
						num18 = num17;
					}
					num = num17 ^ (int)(num2 * 35327124);
					continue;
				}
				case 22u:
				{
					int num11;
					int num12;
					if (_206F_206C_200B_202E_206B_202C_202A_202D_206A_202D_206E_200C_200C_206C_206C_200E_206F_202C_200E_200E_202B_206E_206F_206A_202C_206A_200E_202B_206B_200E_202D_200D_200D_200E_206B_206A_200E_206C_202C_206E_202E(status) == null)
					{
						num11 = -1561546348;
						num12 = num11;
					}
					else
					{
						num11 = -112165015;
						num12 = num11;
					}
					num = num11 ^ ((int)num2 * -1060679956);
					continue;
				}
				case 2u:
					num = ((int)num2 * -1225394917) ^ 0x210B8139;
					continue;
				case 12u:
					uiHandler.HandleGuiStatusChange(lastGUIStatus);
					num = (int)(num2 * 47360728) ^ -1158848924;
					continue;
				case 3u:
					UpdatePersistentNotification();
					num = (int)((num2 * 1170134162) ^ 0x20EBDC61);
					continue;
				case 11u:
					goto IL_035b;
				case 14u:
					lastGUIStatus = _206C_206A_206E_200C_206E_206A_206F_202D_200E_202E_202D_202C_206E_202E_200B_202A_202D_206B_200F_206E_200F_206A_200C_202A_206B_202E_200F_200C_202B_200C_202A_206F_206D_200B_206D_200E_206B_202E_206A_200C_202E(status);
					num = 1307101461;
					continue;
				case 9u:
					current.Bodies[key].OverrideIsWorthMapping = true;
					ProcessOrrery(null);
					UpdatePersistentNotification(onlyUpdate: true);
					num = (int)((num2 * 722300980) ^ 0x56385235);
					continue;
				case 21u:
				{
					int num3;
					int num4;
					if ((int)lastGUIStatus == 10)
					{
						num3 = 1023562357;
						num4 = num3;
					}
					else
					{
						num3 = 1590741663;
						num4 = num3;
					}
					num = num3 ^ ((int)num2 * -402207126);
					continue;
				}
				case 17u:
					return;
				}
				break;
				IL_035b:
				int num27;
				if ((int)_206C_206A_206E_200C_206E_206A_206F_202D_200E_202E_202D_202C_206E_202E_200B_202A_202D_206B_200F_206E_200F_206A_200C_202A_206B_202E_200F_200C_202B_200C_202A_206F_206D_200B_206D_200E_206B_202E_206A_200C_202E(status) != 10)
				{
					num = 25224291;
					num27 = num;
				}
				else
				{
					num = 2107378983;
					num27 = num;
				}
				continue;
				IL_0194:
				int num28;
				if (_206C_206A_206E_200C_206E_206A_206F_202D_200E_202E_202D_202C_206E_202E_200B_202A_202D_206B_200F_206E_200F_206A_200C_202A_206B_202E_200F_200C_202B_200C_202A_206F_206D_200B_206D_200E_206B_202E_206A_200C_202E(status) == lastGUIStatus)
				{
					num = 2021116820;
					num28 = num;
				}
				else
				{
					num = 1459273564;
					num28 = num;
				}
			}
			goto IL_0010;
			IL_0140:
			int num29;
			if (settings.Notifications.DSSPersistent.Popup)
			{
				num = 2132189648;
				num29 = num;
			}
			else
			{
				num = 2107378983;
				num29 = num;
			}
			goto IL_0015;
		}
		catch (Exception ex)
		{
			while (true)
			{
				int num30 = 839410475;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num30 ^ 0x3A2BC78F)) % 3)
					{
					case 0u:
						break;
					default:
						return;
					case 2u:
						goto IL_041e;
					case 1u:
						return;
					}
					break;
					IL_041e:
					logger.WriteException("Error in [StatusChange]", ex);
					num30 = 1935409979;
				}
			}
		}
	}
static int _202E_200F_202B_206C_202B_206F_202B_206A_206B_202C_202B_202A_206E_206D_202C_206B_202C_200E_206B_200D_200D_202E_202E_200F_200C_202A_206F_200C_206F_200E_200D_202C_202C_202C_202A_206C_200B_200F_200E_200D_202E(Destination P_0)
	{
		return P_0.Body;
	}
static FocusStatus _206C_206A_206E_200C_206E_206A_206F_202D_200E_202E_202D_202C_206E_202E_200B_202A_202D_206B_200F_206E_200F_206A_200C_202A_206B_202E_200F_200C_202B_200C_202A_206F_206D_200B_206D_200E_206B_202E_206A_200C_202E(Status P_0)
	{
		//IL_0001: Unknown result type (might be due to invalid IL or missing references)
		return P_0.GuiFocus;
	}
static Destination _206F_206C_200B_202E_206B_202C_202A_202D_206A_202D_206E_200C_200C_206C_206C_200E_206F_202C_200E_200E_202B_206E_206F_206A_202C_206A_200E_202B_206B_200E_202D_200D_200D_200E_206B_206A_200E_206C_202C_206E_202E(Status P_0)
	{
		return P_0.Destination;
	}
static ulong _200E_206F_202B_206A_200F_202B_202D_202B_206B_206B_202C_200B_206C_202B_206A_202E_200C_202D_200E_206A_202A_202C_206D_206C_206F_206E_206F_200F_202D_200B_202A_206A_206B_202D_200E_206D_206D_202C_202A_200E_202E(Destination P_0)
	{
		return P_0.System;
	}
public FocusStatus lastGUIStatus;public Guid?nextBodyNotification;public UIHandler uiHandler=new();public Orrery orrery=new();public bool retryProcessOrrery,isReadAll;public List<int>nextToVisit=new(){1};public string ShortName="Evaluator";
public CoreStub Core=new();public Logger logger=new();public EDSystem current=new();public EvalSettings settings=new();public List<string>Trace=>Core.Trace;
public string currentCommander="Source",currentFID="F123";public int currentFIDAsInt=123,lastVisited=-1;public bool OdysseyLoaded,systemIsCompleteOrABodyMapped;public HashSet<int>missingButNeeded=new();public TransportMode transportMode=TransportMode.Ship;
public void AddToOrrery(ScanBaryCentre s){Trace.Add("AddToOrrery:"+s.BodyID);}public void DeleteHecateData(){Trace.Add("DeleteHecateData");}public void ProcessOrrery(int? startbody=null){Trace.Add("ProcessOrrery:"+startbody);}public void RequestUpdatedTotal(bool subtractCurrent=true,bool forceSave=false){Trace.Add("RequestUpdatedTotal:"+subtractCurrent);}public void SendHecate(int bodyid,bool sendMainStar){Trace.Add("SendHecate:"+bodyid+":"+sendMainStar);}public void SetCurrentSystem(string n,ulong id,DateTime d){Trace.Add("SetCurrentSystem:"+id);current=new(){Name=n,SystemAddress=id};}public void UpdateUI(){Trace.Add("UpdateUI");}
public void JournalEvent<TJournal>(TJournal journal)
	{
		//IL_1679: Unknown result type (might be due to invalid IL or missing references)
		//IL_167e: Unknown result type (might be due to invalid IL or missing references)
		//IL_1a0b: Unknown result type (might be due to invalid IL or missing references)
		//IL_1a10: Unknown result type (might be due to invalid IL or missing references)
		//IL_0af5: Unknown result type (might be due to invalid IL or missing references)
		//IL_0afa: Unknown result type (might be due to invalid IL or missing references)
		try
		{
			int value = 0;
			Scan val10 = default;
			FSSBodySignals val = default;
			int value2 = default;
			LoadGame val4 = default;
			StartJump val12 = default;
			FSDJump val22 = default;
			CarrierJump val13 = default;
			SAAScanComplete val5 = default;
			ApproachBody val9 = default;
			Location val6 = default;
			FSSAllBodiesFound val17 = default;
			SAASignalsFound val3 = default;
			FSSDiscoveryScan val8 = default;
			Embark val11 = default;
			ScanBaryCentre val7 = default;
			Signal val2 = default;
			TmpCarrierJumpFlags tmpCarrierJumpFlags = default;
			bool flag = default;
			Signal val16 = default;
			int num49 = default;
			string text = default;
			string theString = default;
			NotificationArgs val18 = default;
			int num70 = default;
			while (true)
			{
				int num = 672851684;
				while (true)
				{
					int num11;
					int num12;
					int num13;
					int num14;
					int num17;
					int num21;
					int num22;
					uint num2;
					int num87;
					int num90;
					int num91;
					switch ((num2 = (uint)(num ^ 0x76D1DE33)) % 53)
					{
					case 43u:
						break;
					case 14u:
						return;
					case 19u:
						UpdateUI();
						num = 1326742492;
						continue;
					case 37u:
						val10 = (Scan)((((object)journal) is Scan) ? ((object)journal) : null);
						if (val10 == null)
						{
							num = (int)(num2 * 893763419) ^ -343415416;
							continue;
						}
						goto IL_0fb1;
					case 24u:
					{
						int num15;
						int num16;
						if (val == null)
						{
							num15 = -1325867762;
							num16 = num15;
						}
						else
						{
							num15 = -916302488;
							num16 = num15;
						}
						num = num15 ^ (int)(num2 * 1258628221);
						continue;
					}
					case 22u:
						value2 = 0;
						num = 350502276;
						continue;
					case 38u:
					{
						int num9;
						int num10;
						if (_202C_206C_202A_206C_200F_202A_200E_202A_200C_202B_202D_206E_206E_200C_206A_206E_200F_200C_200F_206E_200C_200F_206B_202C_202E_206F_206D_206F_206A_206B_206B_206E_206C_200D_206F_202A_206B_202B_200F_206A_202E(_200F_202E_202B_200C_206A_200B_200C_206E_200D_206D_200C_202E_206F_200E_200D_202E_206E_206B_206B_200C_200C_202C_202A_206F_202B_206B_202B_200E_206D_200D_200E_206B_202A_206D_202B_206C_200D_200B_206E_206D_202E(val4)))
						{
							num9 = 816647349;
							num10 = num9;
						}
						else
						{
							num9 = 1878126658;
							num10 = num9;
						}
						num = num9 ^ (int)(num2 * 38048240);
						continue;
					}
					case 52u:
						num = ((int)num2 * -1024985115) ^ -1743237246;
						continue;
					case 48u:
						uiHandler.SetCommander(currentFID, currentCommander);
						num = (int)((num2 * 1845924508) ^ 0x50E66867);
						continue;
					case 31u:
						if (!(journal is MultiSellExplorationData))
						{
							num = (int)((num2 * 394034175) ^ 0x11E6288D);
							continue;
						}
						goto IL_18dc;
					case 51u:
						ClosePersistentNotification();
						num = 158183461;
						continue;
					case 40u:
						currentFID = "F0";
						num = 1721902890;
						continue;
					case 25u:
						if (val12 == null)
						{
							num = ((int)num2 * -319487629) ^ 0x68FE592D;
							continue;
						}
						goto IL_1105;
					case 21u:
						_206B_202C_200D_206E_202D_200B_206F_200E_206D_206F_202E_202C_200B_206F_200C_206D_200E_200D_200E_206B_206D_206C_200B_200B_200E_206B_200E_206C_206D_206A_202A_202A_206F_206D_202E_202C_202E_200C_206F_200C_202E(Core, (IObservatoryPlugin)(object)this, PluginID.Hecate, _206A_200E_200E_200F_202D_202E_202C_202E_206F_200C_202C_206C_206B_200D_200F_202E_202D_200E_206E_206C_200C_206F_206F_206C_200D_206F_206B_202C_200D_202E_206B_206A_206B_206B_200F_206C_206C_206E_200B_200F_202E("GetFID", (Dictionary<string, object>)null, (Guid?)null));
						num = 1466006745;
						continue;
					case 49u:
						val22 = (FSDJump)((((object)journal) is FSDJump) ? ((object)journal) : null);
						if (val22 == null)
						{
							num = ((int)num2 * -1546608276) ^ 0x7A70BA2B;
							continue;
						}
						goto IL_0bf6;
					case 33u:
						val13 = (CarrierJump)((((object)journal) is CarrierJump) ? ((object)journal) : null);
						num = ((int)num2 * -1614362603) ^ 0x58FC6D5E;
						continue;
					case 50u:
						value = 0;
						if (_202C_202E_202D_206F_206F_200E_200F_206A_202C_200C_206C_206E_202D_206C_202C_206B_206B_202C_200F_202C_202B_202B_202E_206D_206B_200C_200E_206A_206E_206A_206B_202A_206C_202D_206D_206F_206A_202D_200D_206F_202E((SAASignalsFound)(object)val) == current.SystemAddress)
						{
							num = (int)(num2 * 838768699) ^ -727770559;
							continue;
						}
						return;
					case 10u:
						new Dictionary<string, object>().Add("Name", _200E_206B_202B_200F_202A_206F_200F_200F_206C_200F_206C_200D_200F_202E_206C_202C_202C_202A_206E_200B_202A_202B_200C_206D_206A_206C_206C_206F_202C_202C_202A_200B_202E_202E_200B_206E_206C_206E_202B_206A_202E(val4));
						num = 2029522450;
						continue;
					case 2u:
						val5 = (SAAScanComplete)((((object)journal) is SAAScanComplete) ? ((object)journal) : null);
						num = (int)((num2 * 301292162) ^ 0x77A430EE);
						continue;
					case 11u:
						return;
					case 8u:
					{
						int num88;
						int num89;
						if (val9 != null)
						{
							num88 = -2106305078;
							num89 = num88;
						}
						else
						{
							num88 = -433741708;
							num89 = num88;
						}
						num = num88 ^ (int)(num2 * 230108982);
						continue;
					}
					case 5u:
						if (val13 == null)
						{
							num = ((int)num2 * -878994783) ^ 0x4F83F3DB;
							continue;
						}
						goto IL_0b9c;
					case 46u:
						currentFIDAsInt = 0;
						num = ((int)num2 * -1283913432) ^ -675222180;
						continue;
					case 12u:
						if (!(journal is Died))
						{
							num = (int)((num2 * 304153572) ^ 0x3A3560AA);
							continue;
						}
						goto IL_178a;
					case 13u:
						return;
					case 35u:
						val12 = (StartJump)((((object)journal) is StartJump) ? ((object)journal) : null);
						num = (int)((num2 * 941395898) ^ 0x7FA4C586);
						continue;
					case 30u:
						val6 = (Location)((((object)journal) is Location) ? ((object)journal) : null);
						num = ((int)num2 * -1128046691) ^ 0x24B13867;
						continue;
					case 15u:
						lastVisited = _202E_200D_200B_202B_206B_206B_200E_206F_200D_202E_202D_200D_202E_202E_206E_206A_200E_200D_202E_200C_200D_206F_202C_200F_200D_206C_206B_202A_202E_200E_206F_200B_206E_200C_202B_206E_202D_200B_202A_206E_202E(val9);
						num = 1853483794;
						continue;
					case 9u:
						val17 = (FSSAllBodiesFound)((((object)journal) is FSSAllBodiesFound) ? ((object)journal) : null);
						if (val17 == null)
						{
							num = ((int)num2 * -424075170) ^ 0x74685000;
							continue;
						}
						goto IL_1da5;
					case 1u:
						if (!(journal is SupercruiseExit))
						{
							num = (int)(num2 * 1613085646) ^ -1844326514;
							continue;
						}
						goto IL_0cd3;
					case 18u:
						if (!(journal is ScanOrganic))
						{
							num = (int)(num2 * 273981709) ^ -1237601997;
							continue;
						}
						goto IL_100f;
					case 42u:
						if (val3 == null)
						{
							num = (int)((num2 * 134393003) ^ 0x174B3F51);
							continue;
						}
						goto IL_09b2;
					case 28u:
						if (val8 == null)
						{
							num = ((int)num2 * -369904778) ^ -1674627641;
							continue;
						}
						goto IL_1769;
					case 36u:
						val11 = (Embark)((((object)journal) is Embark) ? ((object)journal) : null);
						if (val11 == null)
						{
							num = ((int)num2 * -115021329) ^ 0xAFE4DCB;
							continue;
						}
						goto IL_16dd;
					case 20u:
					{
						val4 = (LoadGame)((((object)journal) is LoadGame) ? ((object)journal) : null);
						int num19;
						int num20;
						if (val4 == null)
						{
							num19 = -2091860557;
							num20 = num19;
						}
						else
						{
							num19 = -225347566;
							num20 = num19;
						}
						num = num19 ^ ((int)num2 * -1667303853);
						continue;
					}
					case 39u:
					{
						int num18;
						if (!_202B_206E_206E_202D_202D_206A_202E_200D_200C_206B_206C_206A_206C_206F_206A_206B_200E_200D_200B_206F_202D_206A_200E_202C_200F_206B_202E_200F_200F_206A_206D_206C_200C_206F_206C_206D_200E_206E_200F_206C_202E(Core))
						{
							num = 943126765;
							num18 = num;
						}
						else
						{
							num = 1725702959;
							num18 = num;
						}
						continue;
					}
					case 45u:
						value2 = 0;
						num = ((int)num2 * -583025703) ^ 0x18126BC6;
						continue;
					case 7u:
						int.TryParse(_206C_206E_206D_206C_200D_200D_202A_206D_206F_206B_202C_202E_206D_206B_202D_206D_202A_200C_200B_200B_200C_202D_202D_200D_200D_200C_200B_206F_200E_206B_202D_200C_200C_200D_200E_200F_200B_200D_200E_202E_202E(_200F_202E_202B_200C_206A_200B_200C_206E_200D_206D_200C_202E_206F_200E_200D_202E_206E_206B_206B_200C_200C_202C_202A_206F_202B_206B_202B_200E_206D_200D_200E_206B_202A_206D_202B_206C_200D_200B_206E_206D_202E(val4), "F", ""), out currentFIDAsInt);
						num = 2102382288;
						continue;
					case 32u:
						val = (FSSBodySignals)((((object)journal) is FSSBodySignals) ? ((object)journal) : null);
						num = (int)(num2 * 152688399) ^ -1036310455;
						continue;
					case 6u:
						val9 = (ApproachBody)((((object)journal) is ApproachBody) ? ((object)journal) : null);
						num = ((int)num2 * -1649608258) ^ 0xA4CE9A1;
						continue;
					case 44u:
						if (val6 == null)
						{
							num = (int)((num2 * 1345220909) ^ 0x5AC546DC);
							continue;
						}
						goto IL_0d6e;
					case 41u:
						if (!(journal is Disembark))
						{
							num = (int)((num2 * 839142221) ^ 0x4552051B);
							continue;
						}
						goto IL_17a6;
					case 47u:
						currentFID = _200F_202E_202B_200C_206A_200B_200C_206E_200D_206D_200C_202E_206F_200E_200D_202E_206E_206B_206B_200C_200C_202C_202A_206F_202B_206B_202B_200E_206D_200D_200E_206B_202A_206D_202B_206C_200D_200B_206E_206D_202E(val4);
						num = 87383847;
						continue;
					case 3u:
						val8 = (FSSDiscoveryScan)((((object)journal) is FSSDiscoveryScan) ? ((object)journal) : null);
						num = ((int)num2 * -1471628612) ^ -661144950;
						continue;
					case 23u:
						if (!(journal is SellExplorationData))
						{
							num = ((int)num2 * -1264488965) ^ -1460675823;
							continue;
						}
						goto IL_190b;
					case 17u:
						val7 = (ScanBaryCentre)((((object)journal) is ScanBaryCentre) ? ((object)journal) : null);
						if (val7 == null)
						{
							num = (int)(num2 * 3509591) ^ -310023115;
							continue;
						}
						goto IL_0cb9;
					case 4u:
						if (val5 == null)
						{
							num = ((int)num2 * -880204887) ^ 0x64F19734;
							continue;
						}
						goto IL_1d0b;
					case 27u:
					{
						int num7;
						int num8;
						if (journal is Shutdown)
						{
							num7 = -81898157;
							num8 = num7;
						}
						else
						{
							num7 = -456682713;
							num8 = num7;
						}
						num = num7 ^ (int)(num2 * 1548373001);
						continue;
					}
					case 0u:
						OdysseyLoaded = _202E_206C_206F_206C_200D_200B_206E_202E_200B_200E_206C_202C_206D_200D_202E_206E_200E_206C_202E_206D_200E_200D_206B_206C_202B_202D_200D_202D_200F_206F_202E_200C_202B_200B_206B_202B_202E_202C_202D_202D_202E(val4);
						num = 851293211;
						continue;
					case 26u:
						currentCommander = _200E_206B_202B_200F_202A_206F_200F_200F_206C_200F_206C_200D_200F_202E_206C_202C_202C_202A_206E_200B_202A_202B_200C_206D_206A_206C_206C_206F_202C_202C_202A_200B_202E_202E_200B_206E_206C_206E_202B_206A_202E(val4);
						num = (int)(num2 * 726864919) ^ -2110545859;
						continue;
					case 29u:
						return;
					case 34u:
						val3 = (SAASignalsFound)((((object)journal) is SAASignalsFound) ? ((object)journal) : null);
						num = (int)((num2 * 1465627909) ^ 0x3D42761D);
						continue;
					default:
						{
							using (ImmutableList<Signal>.Enumerator enumerator = _202A_200E_200D_200D_206E_200D_200F_206C_200D_206A_206A_206E_200D_206A_202E_206D_200F_206B_206B_202E_202A_202E_206B_200F_200C_202E_206E_202A_206C_206F_206B_200E_200D_206A_206D_200B_200E_200D_200D_202D_202E((SAASignalsFound)(object)val).GetEnumerator())
							{
								while (true)
								{
									IL_0813:
									int num3;
									int num4;
									if (!enumerator.MoveNext())
									{
										num3 = 1428550714;
										num4 = num3;
									}
									else
									{
										num3 = 751036272;
										num4 = num3;
									}
									while (true)
									{
										switch ((num2 = (uint)(num3 ^ 0x76D1DE33)) % 7)
										{
										case 5u:
											num3 = 751036272;
											continue;
										default:
											goto end_IL_074d;
										case 4u:
										{
											int num6;
											if (!_206D_200C_206D_200F_200B_200F_200E_200E_202C_206B_206C_206C_202E_200D_206E_202D_206A_202B_202D_206B_200D_206D_206B_200E_206A_206D_200E_206F_206C_206E_200F_206B_200F_202E_206F_200D_202C_200B_200B_206B_202E(_202E_202C_206F_206F_200F_202B_206B_200B_200F_202E_206E_202E_206E_200C_200E_206D_206F_202C_200D_206E_200E_206B_202A_206D_206A_206D_202B_200B_200C_202B_202E_202D_206C_202A_202D_202D_202A_202D_206F_206A_202E(val2), "$SAA_SignalType_Geological;"))
											{
												num3 = 1710676577;
												num6 = num3;
											}
											else
											{
												num3 = 1114823855;
												num6 = num3;
											}
											continue;
										}
										case 2u:
											value = _200D_200F_206D_200D_200B_200B_200F_200D_206A_206A_200E_206B_202E_200E_200C_202C_206F_202B_200B_202B_200C_200C_202D_200F_202B_202B_206B_200C_206D_202B_206C_206B_206C_206D_206B_206E_200F_202B_206A_206C_202E(val2);
											num3 = (int)((num2 * 228302517) ^ 0x6BA961D1);
											continue;
										case 3u:
											value2 = _200D_200F_206D_200D_200B_200B_200F_200D_206A_206A_200E_206B_202E_200E_200C_202C_206F_202B_200B_202B_200C_200C_202D_200F_202B_202B_206B_200C_206D_202B_206C_206B_206C_206D_206B_206E_200F_202B_206A_206C_202E(val2);
											num3 = ((int)num2 * -2125509558) ^ 0x63FFCB79;
											continue;
										case 1u:
										{
											val2 = enumerator.Current;
											int num5;
											if (!_206D_200C_206D_200F_200B_200F_200E_200E_202C_206B_206C_206C_202E_200D_206E_202D_206A_202B_202D_206B_200D_206D_206B_200E_206A_206D_200E_206F_206C_206E_200F_206B_200F_202E_206F_200D_202C_200B_200B_206B_202E(_202E_202C_206F_206F_200F_202B_206B_200B_200F_202E_206E_202E_206E_200C_200E_206D_206F_202C_200D_206E_200E_206B_202A_206D_206A_206D_202B_200B_200C_202B_202E_202D_206C_202A_202D_202D_202A_202D_206F_206A_202E(val2), "$SAA_SignalType_Biological;"))
											{
												num3 = 1621699862;
												num5 = num3;
											}
											else
											{
												num3 = 1457894595;
												num5 = num3;
											}
											continue;
										}
										case 6u:
											break;
										case 0u:
											goto end_IL_074d;
										}
										goto IL_0813;
										continue;
										end_IL_074d:
										break;
									}
									break;
								}
							}
							current.SetBioGeocount(_202A_200C_200C_206F_200E_202C_200F_206A_206D_200B_206E_206B_206A_202D_206B_206D_206E_206B_202E_202E_202C_202D_206B_200B_200E_202E_202D_206D_202B_200B_206E_200F_202E_206D_202B_202D_206B_200D_202D_200E_202E((SAASignalsFound)(object)val), value, value2);
							goto IL_085e;
						}
						IL_178a:
						if (transportMode == TransportMode.Ship)
						{
							num11 = 731522708;
							num12 = num11;
						}
						else
						{
							num11 = 1143496466;
							num12 = num11;
						}
						goto IL_1340;
						IL_0d6e:
						if (!_200B_206A_202E_206A_200B_202E_200B_206F_200B_202B_206D_202B_206E_200D_206C_206D_206C_206E_206A_200B_200F_200E_200E_206C_202C_206F_200F_206D_206A_200E_200B_206D_206A_206A_206A_206A_202E_206D_206C_200D_202E(val6))
						{
							num13 = 1874802001;
							num14 = num13;
						}
						else
						{
							num13 = 310179066;
							num14 = num13;
						}
						goto IL_0863;
						IL_17a6:
						transportMode = TransportMode.OnFoot;
						num11 = 1143496466;
						goto IL_1340;
						IL_1769:
						if (!current.Active)
						{
							num11 = 1143496466;
							num17 = num11;
						}
						else
						{
							num11 = 705796919;
							num17 = num11;
						}
						goto IL_1340;
						IL_09b2:
						if (_202C_202E_202D_206F_206F_200E_200F_206A_202C_200C_206C_206E_202D_206C_202C_206B_206B_202C_200F_202C_202B_202B_202E_206D_206B_200C_200E_206A_206E_206A_206B_202A_206C_202D_206D_206F_206A_202D_200D_206F_202E(val3) == current.SystemAddress)
						{
							num13 = 1639543302;
							goto IL_0863;
						}
						return;
						IL_0cd3:
						ClosePersistentNotification();
						num13 = 38240310;
						goto IL_0863;
						IL_0fb1:
						if (OdysseyLoaded)
						{
							num13 = 1468788246;
							num21 = num13;
						}
						else
						{
							num13 = 1765232678;
							num21 = num13;
						}
						goto IL_0863;
						IL_16dd:
						if (!_200C_200E_200B_200B_200D_206F_202A_206D_202B_200F_206C_206A_206D_200D_200C_200B_200D_202A_206E_206E_200B_206B_200D_202E_202A_200D_202C_206D_202A_206E_202E_200F_202C_202C_200D_202D_206C_206E_200C_202E_202E((Disembark)(object)val11))
						{
							num11 = 1998811116;
							num22 = num11;
						}
						else
						{
							num11 = 683968780;
							num22 = num11;
						}
						goto IL_1340;
						IL_0cb9:
						if (current.Active)
						{
							num13 = 1858939871;
							goto IL_0863;
						}
						return;
						IL_0863:
						while (true)
						{
							switch ((num2 = (uint)(num13 ^ 0x76D1DE33)) % 64)
							{
							case 14u:
								break;
							case 35u:
								RequestUpdatedTotal(subtractCurrent: false, forceSave: false);
								num13 = ((int)num2 * -1117096027) ^ 0x43D2CA64;
								continue;
							case 59u:
								UpdateUI();
								return;
							case 54u:
								goto IL_09b2;
							case 25u:
								return;
							case 31u:
								OdysseyLoaded = true;
								num13 = (int)(num2 * 1051423240) ^ -2066976530;
								continue;
							case 9u:
								transportMode = TransportMode.SRV;
								num13 = ((int)num2 * -1630891194) ^ -473517695;
								continue;
							case 26u:
								return;
							case 19u:
								if (_200D_206A_206D_206F_206F_206C_206E_202C_206C_206F_202C_202C_206C_206B_202A_206F_206D_202E_206B_206F_202E_206A_206C_202C_206B_206E_206B_206F_200B_206D_200C_200B_206F_200D_200B_200E_202E_206A_202B_206F_202E(val6) != 0)
								{
									num13 = 1980089536;
									continue;
								}
								return;
							case 61u:
							{
								bool flag2 = current.AddBody(val10, OdysseyLoaded);
								if (flag2 && settings.Notifications.BodiesWorthMapping.Any && !_202B_206E_206E_202D_202D_206A_202E_200D_200C_206B_206C_206A_206C_206F_206A_206B_200E_200D_200B_206F_202D_206A_200E_202C_200F_206B_202E_200F_200F_206A_206D_206C_200C_206F_206C_206D_200E_206E_200F_206C_202E(Core))
								{
									string title = Utils.NotificationBodyName(_200D_202E_206F_200F_206E_200C_206E_206B_206E_200E_206E_200C_206A_206B_202C_202A_206D_206B_206E_200E_200C_202A_206C_200F_202D_202C_200B_202D_202A_200C_202A_206E_206E_206D_200F_200F_200D_202C_206F_202C_202E(val10), _202D_206D_206F_200B_206F_206B_206C_206B_200E_202E_206F_200E_202B_206D_200D_202D_200C_200C_200E_200D_200D_200F_206B_206D_202A_202B_202A_200B_200D_200F_200B_200B_200D_200C_202A_202B_206E_200E_200F_202B_202E((ScanBaryCentre)(object)val10));
									string titleSsml = Utils.NotificationBodyName(_200D_202E_206F_200F_206E_200C_206E_206B_206E_200E_206E_200C_206A_206B_202C_202A_206D_206B_206E_200E_200C_202A_206C_200F_202D_202C_200B_202D_202A_200C_202A_206E_206E_206D_200F_200F_200D_202C_206F_202C_202E(val10), _202D_206D_206F_200B_206F_206B_206C_206B_200E_202E_206F_200E_202B_206D_200D_202D_200C_200C_200E_200D_200D_200F_206B_206D_202A_202B_202A_200B_200D_200F_200B_200B_200D_200C_202A_202B_206E_200E_200F_202B_202E((ScanBaryCentre)(object)val10), isTTS: true);
									NotificationArgs val14 = _206D_200B_202C_202D_206C_200C_206D_202E_202A_200C_206C_206E_206C_200E_200F_206C_200C_200E_202D_200C_206F_200E_202C_200D_202D_200D_200E_206F_206B_206B_202E_202D_202E_200C_206E_200E_200F_206F_200E_200D_202E();
									val14.Title = title;
									val14.TitleSsml = titleSsml;
									val14.Detail = "Meets Mapping Criteria";
									val14.Rendering = settings.Notifications.BodiesWorthMapping.Rendering;
									val14.CoalescingId = _200B_200C_200D_202C_200D_202B_200E_200B_202E_200D_200C_200D_200B_206B_202E_206B_206A_206B_202A_206E_206C_206E_206C_200D_206D_206B_206C_200E_202D_200F_200E_202E_202B_202A_200C_202D_200D_200F_206D_202E_202E((ScanBaryCentre)(object)val10);
									val14.Sender = ShortName;
									NotificationArgs val15 = val14;
									_200C_200E_206A_200D_202B_202D_200E_200B_202B_206A_200D_200B_200D_200B_200F_206A_206F_206B_202D_200E_202A_200E_206C_202A_206D_200B_200B_200C_202C_200E_202B_200C_206D_206F_200C_200F_200E_206A_200C_202E(Core, val15);
								}
								if (!flag2)
								{
									num13 = 1015030643;
									continue;
								}
								goto case 46u;
							}
							case 15u:
								UpdateUI();
								num13 = ((int)num2 * -1019311270) ^ 0x7F3A2FC7;
								continue;
							case 53u:
								value2 = 0;
								value = 0;
								num13 = (int)((num2 * 579559013) ^ 0x78B702AB);
								continue;
							case 34u:
								goto IL_0b6c;
							case 4u:
								num13 = (int)(num2 * 1429164058) ^ -2129305400;
								continue;
							case 56u:
								goto IL_0b9c;
							case 11u:
							{
								int num27;
								int num28;
								if (tmpCarrierJumpFlags.Docked)
								{
									num27 = -264345125;
									num28 = num27;
								}
								else
								{
									num27 = -1074417565;
									num28 = num27;
								}
								num13 = num27 ^ ((int)num2 * -787323805);
								continue;
							}
							case 43u:
								missingButNeeded = orrery.GetMissingBodies;
								num13 = 1590976700;
								continue;
							case 55u:
								goto IL_0bf6;
							case 50u:
								return;
							case 38u:
							{
								int num37;
								int num38;
								if (_206B_200F_202E_200C_206C_200B_200C_200F_200F_202A_200C_200D_200B_200B_206F_202D_206D_206E_202D_202C_206B_206A_202A_200E_206C_200E_202D_206D_200F_200E_202B_200C_202E_202C_200E_200F_206C_200B_202B_206D_202E(val10) > 0f)
								{
									num37 = -1520142726;
									num38 = num37;
								}
								else
								{
									num37 = -1159701440;
									num38 = num37;
								}
								num13 = num37 ^ (int)(num2 * 371045065);
								continue;
							}
							case 6u:
							{
								int num33;
								int num34;
								if (!_202C_206C_202A_206C_200F_202A_200E_202A_200C_202B_202D_206E_206E_200C_206A_206E_200F_200C_200F_206E_200C_200F_206B_202C_202E_206F_206D_206F_206A_206B_206B_206E_206C_200D_206F_202A_206B_202B_200F_206A_202E(_206F_202D_202D_200D_206C_200D_202B_206C_202C_206A_206A_200F_202C_202C_200D_202D_202B_202D_202C_200C_200B_202C_202C_202E_206B_200C_202D_206C_206D_206C_202E_202B_200F_206F_206E_200B_206B_206D_200B_206A_202E(val10)))
								{
									num33 = -415792925;
									num34 = num33;
								}
								else
								{
									num33 = -162328845;
									num34 = num33;
								}
								num13 = num33 ^ ((int)num2 * -678373889);
								continue;
							}
							case 40u:
								if (_202D_202A_206D_202A_202E_202D_206F_200C_202E_200F_200D_200B_202D_202E_206E_206A_202D_206C_200E_202B_200C_200C_202E_206F_200E_206F_200D_202D_206C_200D_206F_202B_206D_206C_200E_202E_202A_202B_206A_202E_202E((ScanBaryCentre)(object)val10) == current.SystemAddress)
								{
									num13 = (int)(num2 * 1586723498) ^ -853630427;
									continue;
								}
								return;
							case 12u:
								goto IL_0cb9;
							case 23u:
								goto IL_0cd3;
							case 39u:
							{
								int num31;
								int num32;
								if (!_202E_200E_206C_202E_202C_200B_206A_206B_202E_206F_200C_200F_200E_206F_206B_202D_202B_206F_202B_202E_200C_206D_202B_200B_206D_200F_200C_202D_206D_206E_206F_206B_200D_200D_202A_206B_202C_206B_202D_200D_202E(val10))
								{
									num31 = -584880607;
									num32 = num31;
								}
								else
								{
									num31 = -1235158366;
									num32 = num31;
								}
								num13 = num31 ^ (int)(num2 * 392890737);
								continue;
							}
							case 22u:
								if (_206D_200C_206D_200F_200B_200F_200E_200E_202C_206B_206C_206C_202E_200D_206E_202D_206A_202B_202D_206B_200D_206D_206B_200E_206A_206D_200E_206F_206C_206E_200F_206B_200F_202E_206F_200D_202C_200B_200B_206B_202E(_206B_206F_202D_206F_202C_206E_202B_202E_202A_206B_206B_206A_206D_206E_206F_200F_202A_206E_206B_206F_206D_206D_202D_202E_200E_200B_202A_200F_202B_206C_202C_200E_202C_200C_200B_206E_202D_200D_206B_200D_202E(val12), "Supercruise"))
								{
									num13 = 136549342;
									continue;
								}
								return;
							case 32u:
								transportMode = TransportMode.Ship;
								num13 = 872790560;
								continue;
							case 45u:
								if (settings.Notifications.AlwaysPersistent.Popup)
								{
									num13 = ((int)num2 * -1320904915) ^ -1768968861;
									continue;
								}
								return;
							case 2u:
								goto IL_0d6e;
							case 37u:
								if (current.Active)
								{
									num13 = 188667291;
									continue;
								}
								return;
							case 52u:
								SendHecate(_200B_200C_200D_202C_200D_202B_200E_200B_202E_200D_200C_200D_200B_206B_202E_206B_206A_206B_202A_206E_206C_206E_206C_200D_206D_206B_206C_200E_202D_200F_200E_202E_202B_202A_200C_202D_200D_200F_206D_202E_202E((ScanBaryCentre)(object)val10), sendMainStar: true);
								num13 = (int)(num2 * 370552732) ^ -93784856;
								continue;
							case 18u:
							{
								int num35;
								int num36;
								if (!missingButNeeded.Contains(_200B_200C_200D_202C_200D_202B_200E_200B_202E_200D_200C_200D_200B_206B_202E_206B_206A_206B_202A_206E_206C_206E_206C_200D_206D_206B_206C_200E_202D_200F_200E_202E_202B_202A_200C_202D_200D_200F_206D_202E_202E((ScanBaryCentre)(object)val10)))
								{
									num35 = -2008671233;
									num36 = num35;
								}
								else
								{
									num35 = -534311839;
									num36 = num35;
								}
								num13 = num35 ^ (int)(num2 * 1289427374);
								continue;
							}
							case 42u:
								flag = current.Bodies.ContainsKey(_200B_200C_200D_202C_200D_202B_200E_200B_202E_200D_200C_200D_200B_206B_202E_206B_206A_206B_202A_206E_206C_206E_206C_200D_206D_206B_206C_200E_202D_200F_200E_202E_202B_202A_200C_202D_200D_200F_206D_202E_202E((ScanBaryCentre)(object)val10));
								AddToOrrery(val10);
								num13 = 884824078;
								continue;
							case 29u:
								goto IL_0e22;
							case 5u:
								return;
							case 60u:
								ClosePersistentNotification();
								if (!_202B_206E_206E_202D_202D_206A_202E_200D_200C_206B_206C_206A_206C_206F_206A_206B_200E_200D_200B_206F_202D_206A_200E_202C_200F_206B_202E_200F_200F_206A_206D_206C_200C_206F_206C_206D_200E_206E_200F_206C_202E(Core))
								{
									num13 = ((int)num2 * -957003116) ^ 0x4E3F6D20;
									continue;
								}
								return;
							case 10u:
								UpdateUI();
								return;
							case 57u:
								if (nextToVisit.Count > 0)
								{
									num13 = ((int)num2 * -1135403225) ^ 0x14AB33C2;
									continue;
								}
								return;
							case 33u:
								return;
							case 17u:
								if (tmpCarrierJumpFlags.OnFoot)
								{
									num13 = ((int)num2 * -941970604) ^ 0x72D6E88E;
									continue;
								}
								return;
							case 13u:
								return;
							case 41u:
								if (_202E_202B_200F_202A_200C_200C_200B_202A_206B_200D_206B_202C_200E_200B_206B_206F_206E_206A_206D_202E_202D_200D_206B_206C_202D_206F_206E_200D_206C_206D_202A_202B_206E_206B_206D_206B_200E_202B_202A_202C_202E((FSDJump)(object)val13) != 0)
								{
									num13 = 748510760;
									continue;
								}
								return;
							case 16u:
								SetCurrentSystem(_202C_206E_206D_200C_206B_200C_200D_206D_202B_202E_202B_200E_202B_200F_206E_206F_206F_206B_206A_200B_206D_200E_200B_206E_206A_200B_202B_202A_202C_206D_200F_202A_200B_200F_200C_206F_200C_202E_200D_206E_202E((FSDJump)(object)val13), _202E_202B_200F_202A_200C_200C_200B_202A_206B_200D_206B_202C_200E_200B_206B_206F_206E_206A_206D_202E_202D_200D_206B_206C_202D_206F_206E_200D_206C_206D_202A_202B_206E_206B_206D_206B_200E_202B_202A_202C_202E((FSDJump)(object)val13), _206A_202B_202B_206E_206D_206B_202C_202C_202D_202D_200E_200C_202E_206E_206E_206B_202B_202A_202C_202A_202B_200F_206E_202E_206A_200D_202B_200F_206C_206D_206B_202D_202B_206C_206B_206D_206F_206D_206C_206E_202E((JournalBase)(object)val13));
								UpdateUI();
								num13 = (int)((num2 * 764125782) ^ 0x41B4FF2);
								continue;
							case 44u:
								if (_202D_202A_206D_202A_202E_202D_206F_200C_202E_200F_200D_200B_202D_202E_206E_206A_202D_206C_200E_202B_200C_200C_202E_206F_200E_206F_200D_202D_206C_200D_206F_202B_206D_206C_200E_202E_202A_202B_206A_202E_202E(val7) == current.SystemAddress)
								{
									num13 = (int)((num2 * 1594397976) ^ 0x6637CF94);
									continue;
								}
								return;
							case 30u:
								UpdatePersistentNotification();
								num13 = (int)(num2 * 1547019929) ^ -435822832;
								continue;
							case 3u:
								goto IL_0fb1;
							case 27u:
								if (_202E_202B_200F_202A_200C_200C_200B_202A_206B_200D_206B_202C_200E_200B_206B_206F_206E_206A_206D_202E_202D_200D_206B_206C_202D_206F_206E_200D_206C_206D_202A_202B_206E_206B_206D_206B_200E_202B_202A_202C_202E((FSDJump)(object)val13) != current.SystemAddress)
								{
									num13 = (int)(num2 * 1302793909) ^ -1100852236;
									continue;
								}
								return;
							case 36u:
								goto IL_0ff7;
							case 8u:
								goto IL_100f;
							case 24u:
								return;
							case 58u:
								if (!_202C_206C_202A_206C_200F_202A_200E_202A_200C_202B_202D_206E_206E_200C_206A_206E_200F_200C_200F_206E_200C_200F_206B_202C_202E_206F_206D_206F_206A_206B_206B_206E_206C_200D_206F_202A_206B_202B_200F_206A_202E(_202D_200E_202D_206D_206F_202E_200F_202C_200B_202C_200C_202D_200D_200F_202A_202E_202A_206F_200D_200D_202E_206D_202C_202B_206E_202C_200E_206D_200C_206C_200B_206E_206F_200D_206B_202C_200E_202D_200E_202E(val10)))
								{
									num13 = ((int)num2 * -346629610) ^ 0x40654D65;
									continue;
								}
								return;
							case 1u:
							{
								int num29;
								int num30;
								if (_202C_206C_202A_206C_200F_202A_200E_202A_200C_202B_202D_206E_206E_200C_206A_206E_200F_200C_200F_206E_200C_200F_206B_202C_202E_206F_206D_206F_206A_206B_206B_206E_206C_200D_206F_202A_206B_202B_200F_206A_202E(_202A_202B_206D_202D_206F_202E_206E_202D_206C_206D_202B_206F_206B_200F_202A_202A_206A_200B_206A_200C_202A_202C_202C_206F_202B_206E_200B_202A_206C_206E_200C_206C_206C_202B_200D_206A_202D_202B_206A_202D_202E(val10)))
								{
									num29 = 1112617015;
									num30 = num29;
								}
								else
								{
									num29 = 1301358223;
									num30 = num29;
								}
								num13 = num29 ^ ((int)num2 * -1600465695);
								continue;
							}
							case 51u:
								if (_200D_206A_206D_206F_206F_206C_206E_202C_206C_206F_202C_202C_206C_206B_202A_206F_206D_202E_206B_206F_202E_206A_206C_202C_206B_206E_206B_206F_200B_206D_200C_200B_206F_200D_200B_200E_202E_206A_202B_206F_202E(val6) != current.SystemAddress)
								{
									num13 = ((int)num2 * -274090458) ^ -1876447090;
									continue;
								}
								return;
							case 63u:
								return;
							case 0u:
							{
								int num25;
								int num26;
								if (flag)
								{
									num25 = 1251891075;
									num26 = num25;
								}
								else
								{
									num25 = 1454894881;
									num26 = num25;
								}
								num13 = num25 ^ ((int)num2 * -822736145);
								continue;
							}
							case 7u:
								AddToOrrery(val7);
								num13 = ((int)num2 * -600232452) ^ -858428339;
								continue;
							case 28u:
								goto IL_1105;
							case 21u:
							{
								int num23;
								int num24;
								if (_202C_206C_202A_206C_200F_202A_200E_202A_200C_202B_202D_206E_206E_200C_206A_206E_200F_200C_200F_206E_200C_200F_206B_202C_202E_206F_206D_206F_206A_206B_206B_206E_206C_200D_206F_202A_206B_202B_200F_206A_202E(_206F_202D_202D_200D_206C_200D_202B_206C_202C_206A_206A_200F_202C_202C_200D_202D_202B_202D_202C_200C_200B_202C_202C_202E_206B_200C_202D_206C_206D_206C_202E_202B_200F_206F_206E_200B_206B_206D_200B_206A_202E(val10)))
								{
									num23 = -1996749279;
									num24 = num23;
								}
								else
								{
									num23 = -1809463483;
									num24 = num23;
								}
								num13 = num23 ^ ((int)num2 * -973980645);
								continue;
							}
							case 62u:
								num13 = ((int)num2 * -1654311071) ^ 0x323AD31E;
								continue;
							case 20u:
								transportMode = TransportMode.OnFoot;
								num13 = ((int)num2 * -2075172230) ^ -1243277755;
								continue;
							case 48u:
								goto IL_118b;
							case 47u:
								SetCurrentSystem(_202A_202B_206E_200D_202C_206C_200F_206C_202A_202C_202C_202C_202A_206E_206D_202B_202C_206B_202C_202E_200D_200C_206D_202D_202A_206E_206E_200B_200B_200D_200B_206F_202B_200F_206F_200E_202C_202A_202B_202B_202E(val6), _200D_206A_206D_206F_206F_206C_206E_202C_206C_206F_202C_202C_206C_206B_202A_206F_206D_202E_206B_206F_202E_206A_206C_202C_206B_206E_206B_206F_200B_206D_200C_200B_206F_200D_200B_200E_202E_206A_202B_206F_202E(val6), _206A_202B_202B_206E_206D_206B_202C_202C_202D_202D_200E_200C_202E_206E_206E_206B_202B_202A_202C_202A_202B_200F_206E_202E_206A_200D_202B_200F_206C_206D_206B_202D_202B_206C_206B_206D_206F_206D_206C_206E_202E((JournalBase)(object)val6));
								num13 = ((int)num2 * -1082393153) ^ -420284327;
								continue;
							case 46u:
								ProcessOrrery(null);
								num13 = 1283147863;
								continue;
							default:
								goto IL_11ef;
							}
							break;
							IL_118b:
							int num39;
							if (!retryProcessOrrery)
							{
								num13 = 1283147863;
								num39 = num13;
							}
							else
							{
								num13 = 1393602269;
								num39 = num13;
							}
							continue;
							IL_0e22:
							int num40;
							if (!_202D_202B_200D_200B_206C_202A_200C_200D_202A_200B_202D_200B_202E_206F_202B_206C_206F_200E_200C_206F_206E_206F_206A_206A_206C_200F_200B_202E_200D_206E_206A_206D_202D_200B_200F_202E_206D_200E_200F_206B_202E(_202A_202B_206D_202D_206F_202E_206E_202D_206C_206D_202B_206F_206B_200F_202A_202A_206A_200B_206A_200C_202A_202C_202C_206F_202B_206E_200B_202A_206C_206E_200C_206C_206C_202B_200D_206A_202D_202B_206A_202D_202E(val10), "None"))
							{
								num13 = 1468788246;
								num40 = num13;
							}
							else
							{
								num13 = 1302515220;
								num40 = num13;
							}
							continue;
							IL_0b6c:
							int num41;
							if (!_206C_200B_206C_202D_206D_200E_202C_200D_206A_206D_202C_200B_202A_206C_206C_200D_202C_202B_200D_206A_206A_200B_206B_206F_206A_206A_206C_206B_206C_200E_206B_206F_206A_206F_206E_202A_206D_202B_200B_206D_202E(val6))
							{
								num13 = 1616880915;
								num41 = num13;
							}
							else
							{
								num13 = 873422119;
								num41 = num13;
							}
							continue;
							IL_0ff7:
							int num42;
							if (flag)
							{
								num13 = 1825760088;
								num42 = num13;
							}
							else
							{
								num13 = 1582592775;
								num42 = num13;
							}
						}
						goto IL_085e;
						IL_11ef:
						using (ImmutableList<Signal>.Enumerator enumerator = _202A_200E_200D_200D_206E_200D_200F_206C_200D_206A_206A_206E_200D_206A_202E_206D_200F_206B_206B_202E_202A_202E_206B_200F_200C_202E_206E_202A_206C_206F_206B_200E_200D_206A_206D_200B_200E_200D_200D_202D_202E(val3).GetEnumerator())
						{
							while (true)
							{
								IL_12c2:
								int num43;
								int num44;
								if (!enumerator.MoveNext())
								{
									num43 = 653896340;
									num44 = num43;
								}
								else
								{
									num43 = 816785004;
									num44 = num43;
								}
								while (true)
								{
									switch ((num2 = (uint)(num43 ^ 0x76D1DE33)) % 9)
									{
									case 7u:
										num43 = 816785004;
										continue;
									default:
										goto end_IL_1207;
									case 5u:
										val16 = enumerator.Current;
										num43 = 808272426;
										continue;
									case 6u:
										value = _200D_200F_206D_200D_200B_200B_200F_200D_206A_206A_200E_206B_202E_200E_200C_202C_206F_202B_200B_202B_200C_200C_202D_200F_202B_202B_206B_200C_206D_202B_206C_206B_206C_206D_206B_206E_200F_202B_206A_206C_202E(val16);
										num43 = ((int)num2 * -763830035) ^ 0xE014ED7;
										continue;
									case 4u:
										value2 = _200D_200F_206D_200D_200B_200B_200F_200D_206A_206A_200E_206B_202E_200E_200C_202C_206F_202B_200B_202B_200C_200C_202D_200F_202B_202B_206B_200C_206D_202B_206C_206B_206C_206D_206B_206E_200F_202B_206A_206C_202E(val16);
										num43 = (int)((num2 * 550240041) ^ 0x1AD84FA6);
										continue;
									case 8u:
									{
										int num46;
										if (_206D_200C_206D_200F_200B_200F_200E_200E_202C_206B_206C_206C_202E_200D_206E_202D_206A_202B_202D_206B_200D_206D_206B_200E_206A_206D_200E_206F_206C_206E_200F_206B_200F_202E_206F_200D_202C_200B_200B_206B_202E(_202E_202C_206F_206F_200F_202B_206B_200B_200F_202E_206E_202E_206E_200C_200E_206D_206F_202C_200D_206E_200E_206B_202A_206D_206A_206D_202B_200B_200C_202B_202E_202D_206C_202A_202D_202D_202A_202D_206F_206A_202E(val16), "$SAA_SignalType_Biological;"))
										{
											num43 = 367994414;
											num46 = num43;
										}
										else
										{
											num43 = 2089584758;
											num46 = num43;
										}
										continue;
									}
									case 2u:
										num43 = ((int)num2 * -1752822312) ^ 0x69E88E46;
										continue;
									case 0u:
										break;
									case 3u:
									{
										int num45;
										if (_206D_200C_206D_200F_200B_200F_200E_200E_202C_206B_206C_206C_202E_200D_206E_202D_206A_202B_202D_206B_200D_206D_206B_200E_206A_206D_200E_206F_206C_206E_200F_206B_200F_202E_206F_200D_202C_200B_200B_206B_202E(_202E_202C_206F_206F_200F_202B_206B_200B_200F_202E_206E_202E_206E_200C_200E_206D_206F_202C_200D_206E_200E_206B_202A_206D_206A_206D_202B_200B_200C_202B_202E_202D_206C_202A_202D_202D_202A_202D_206F_206A_202E(val16), "$SAA_SignalType_Geological;"))
										{
											num43 = 1927243243;
											num45 = num43;
										}
										else
										{
											num43 = 1561883198;
											num45 = num43;
										}
										continue;
									}
									case 1u:
										goto end_IL_1207;
									}
									goto IL_12c2;
									continue;
									end_IL_1207:
									break;
								}
								break;
							}
						}
						current.SetBioGeocount(_202A_200C_200C_206F_200E_202C_200F_206A_206D_200B_206E_206B_206A_202D_206B_206D_206E_206B_202E_202E_202C_202D_206B_200B_200E_202E_202D_206D_202B_200B_206E_200F_202E_206D_202B_202D_206B_200D_202D_200E_202E(val3), value, value2);
						goto IL_133b;
						IL_085e:
						num13 = 1364976057;
						goto IL_0863;
						IL_133b:
						num11 = 1097549753;
						goto IL_1340;
						IL_1340:
						while (true)
						{
							switch ((num2 = (uint)(num11 ^ 0x76D1DE33)) % 61)
							{
							case 47u:
								break;
							default:
								return;
							case 27u:
								systemIsCompleteOrABodyMapped = true;
								SendHecate(50000, sendMainStar: false);
								ProcessOrrery(null);
								UpdateUI();
								num11 = (int)((num2 * 793548441) ^ 0x31FDA1B6);
								continue;
							case 54u:
								current.Bodies[_200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(val5)].IsMappedEfficiently = _206B_206B_206A_206C_202C_200D_206D_200C_200B_202C_200F_200F_202E_202C_200D_206C_206E_206C_206F_206E_206F_206B_206D_202D_206F_202C_206E_200F_206F_206D_206F_202B_200C_206C_206F_200F_200F_206A_202E_200C_202E(val5) <= _206F_202B_200D_200F_202E_202B_202E_206B_206E_202A_206B_206A_206C_200E_202A_202C_206F_206D_202C_202A_200B_200D_206D_202E_200B_202A_200B_202E_206A_206D_206A_200D_202C_200D_200B_200C_206A_200B_206B_206A_202E(val5);
								num11 = 1586993316;
								continue;
							case 9u:
								UpdateUI();
								num11 = 1940623383;
								continue;
							case 21u:
							{
								int num52;
								int num53;
								if (current.SystemAddress != _202C_202A_200D_206E_202C_200C_206E_206E_202A_200F_200F_202D_200D_200B_200D_206B_200D_206A_206C_202C_200D_206A_200C_206E_206C_200B_206E_206F_206F_202A_206E_200B_202E_206D_206B_200E_200E_200E_200C_200C_202E(val5))
								{
									num52 = 12342098;
									num53 = num52;
								}
								else
								{
									num52 = 1068497614;
									num53 = num52;
								}
								num11 = num52 ^ ((int)num2 * -1712589882);
								continue;
							}
							case 20u:
								ProcessOrrery(_200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(val5));
								num11 = (int)((num2 * 454334889) ^ 0x4A02FA7A);
								continue;
							case 23u:
							{
								num49 = nextToVisit[0];
								int num66;
								int num67;
								if (num49 > -1)
								{
									num66 = -922840137;
									num67 = num66;
								}
								else
								{
									num66 = -369030484;
									num67 = num66;
								}
								num11 = num66 ^ (int)(num2 * 1634065167);
								continue;
							}
							case 50u:
								nextToVisit.RemoveAt(0);
								num11 = ((int)num2 * -1920322240) ^ 0x3E5BCC30;
								continue;
							case 32u:
							{
								int num77;
								int num78;
								if (!settings.Notifications.EfficientRoute.Any)
								{
									num77 = -1720071466;
									num78 = num77;
								}
								else
								{
									num77 = -1266381527;
									num78 = num77;
								}
								num11 = num77 ^ (int)(num2 * 299011130);
								continue;
							}
							case 13u:
								goto IL_15a8;
							case 43u:
								num11 = (int)(num2 * 1654796504) ^ -926467974;
								continue;
							case 12u:
							{
								int num62;
								int num63;
								if (nextToVisit.Count <= 0)
								{
									num62 = 1021860438;
									num63 = num62;
								}
								else
								{
									num62 = 352632171;
									num63 = num62;
								}
								num11 = num62 ^ ((int)num2 * -974320002);
								continue;
							}
							case 25u:
								SendHecate(_200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(val5), sendMainStar: true);
								num11 = ((int)num2 * -395611409) ^ 0x4FDC424B;
								continue;
							case 4u:
								DeleteHecateData();
								num11 = ((int)num2 * -1188448820) ^ -1238888528;
								continue;
							case 1u:
							{
								NotificationArgs val19 = _206D_200B_202C_202D_206C_200C_206D_202E_202A_200C_206C_206E_206C_200E_200F_206C_200C_200E_202D_200C_206F_200E_202C_200D_202D_200D_200E_206F_206B_206B_202E_202D_202E_200C_206E_200E_200F_206F_200E_200D_202E();
								val19.Title = "Mapping Complete";
								val19.Detail = text;
								val19.DetailSsml = Utils.SSMLify(theString);
								val19.Rendering = settings.Notifications.EfficientRoute.Rendering;
								val19.Sender = ShortName;
								val18 = val19;
								num11 = 2064832588;
								continue;
							}
							case 49u:
							{
								int num81;
								int num82;
								if (current.Bodies.ContainsKey(num70))
								{
									num81 = 356041260;
									num82 = num81;
								}
								else
								{
									num81 = 86764154;
									num82 = num81;
								}
								num11 = num81 ^ (int)(num2 * 849511892);
								continue;
							}
							case 16u:
								UpdatePersistentNotification(onlyUpdate: true);
								num11 = 1441292263;
								continue;
							case 41u:
								goto IL_16dd;
							case 48u:
							{
								int num73;
								int num74;
								if (nextToVisit.Count <= 0)
								{
									num73 = -100691147;
									num74 = num73;
								}
								else
								{
									num73 = -1279709214;
									num74 = num73;
								}
								num11 = num73 ^ ((int)num2 * -2084700633);
								continue;
							}
							case 0u:
								num11 = (int)((num2 * 1450287007) ^ 0x5FE0EC36);
								continue;
							case 3u:
							{
								int num68;
								int num69;
								if (_202C_202C_206F_206F_206B_202A_206E_200B_206A_202C_200D_206A_200D_206D_200C_200B_206F_202C_206E_200D_200D_200D_206F_206E_202B_200E_206E_206E_200F_200C_202A_200F_200F_206E_202E_202D_202B_206D_202B_202B_202E(val8) != current.SystemAddress)
								{
									num68 = 715208086;
									num69 = num68;
								}
								else
								{
									num68 = 1969341069;
									num69 = num68;
								}
								num11 = num68 ^ ((int)num2 * -1993961119);
								continue;
							}
							case 53u:
								goto IL_1769;
							case 11u:
								goto IL_178a;
							case 31u:
								goto IL_17a6;
							case 36u:
								goto IL_17b7;
							case 5u:
								transportMode = TransportMode.Ship;
								num11 = 1891535737;
								continue;
							case 22u:
							{
								int num58;
								int num59;
								if (_200E_200B_200B_200E_202D_200D_202D_206F_206F_200B_202B_200C_206E_202D_206A_202B_206D_206E_206C_206C_206E_200B_206C_206B_200E_206E_200D_206B_200D_206E_206D_202D_200B_202B_200E_206A_206D_202B_200E_206A_202E(val17) == current.SystemAddress)
								{
									num58 = -287130965;
									num59 = num58;
								}
								else
								{
									num58 = -843340690;
									num59 = num58;
								}
								num11 = num58 ^ ((int)num2 * -2052443636);
								continue;
							}
							case 39u:
							{
								int num54;
								int num55;
								if (nextToVisit.Count <= 0)
								{
									num54 = 1756775336;
									num55 = num54;
								}
								else
								{
									num54 = 2135599575;
									num55 = num54;
								}
								num11 = num54 ^ ((int)num2 * -251376430);
								continue;
							}
							case 28u:
								current.TotalBodies = _200E_206B_202A_200B_200F_200B_206D_206D_202D_206E_202A_200F_202A_206C_206F_202C_206C_200B_202A_206F_200F_206F_206E_202C_200C_206C_206D_200E_202A_206B_202D_202A_202A_206D_206B_206D_202C_202C_202B_202C_202E(val8);
								num11 = (int)((num2 * 1871145755) ^ 0x3BAA8271);
								continue;
							case 38u:
							{
								int num79;
								int num80;
								if (!current.Bodies.ContainsKey(_200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(val5)))
								{
									num79 = -1892400230;
									num80 = num79;
								}
								else
								{
									num79 = -1470162157;
									num80 = num79;
								}
								num11 = num79 ^ (int)(num2 * 507915048);
								continue;
							}
							case 2u:
								text = "No more bodies to map";
								num11 = 2001558557;
								continue;
							case 6u:
								goto IL_18c0;
							case 18u:
								goto IL_18dc;
							case 44u:
								SendHecate(50001, sendMainStar: false);
								num11 = (int)((num2 * 1572270034) ^ 0x597D0F7);
								continue;
							case 8u:
								goto IL_190b;
							case 33u:
								transportMode = TransportMode.SRV;
								num11 = ((int)num2 * -1264416793) ^ -334910005;
								continue;
							case 56u:
								theString = text;
								num11 = ((int)num2 * -1062164741) ^ 0x1ED57943;
								continue;
							case 45u:
								goto IL_194c;
							case 14u:
								lastVisited = _200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(val5);
								current.Bodies[_200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(val5)].IsMapped = true;
								num11 = ((int)num2 * -1020519526) ^ -157989681;
								continue;
							case 51u:
							{
								int num75;
								int num76;
								if (settings.Notifications.DSSPersistent.Popup)
								{
									num75 = -1201218846;
									num76 = num75;
								}
								else
								{
									num75 = -1065165297;
									num76 = num75;
								}
								num11 = num75 ^ ((int)num2 * -910126109);
								continue;
							}
							case 40u:
								UpdatePersistentNotification();
								num11 = ((int)num2 * -234998656) ^ -1859296604;
								continue;
							case 58u:
							{
								num70 = nextToVisit[0];
								int num71;
								int num72;
								if (num70 > -1)
								{
									num71 = 1493083101;
									num72 = num71;
								}
								else
								{
									num71 = 558489678;
									num72 = num71;
								}
								num11 = num71 ^ (int)(num2 * 265980721);
								continue;
							}
							case 29u:
								UpdateUI();
								num11 = (int)(num2 * 876024662) ^ -762147506;
								continue;
							case 42u:
								theString = _202E_206A_206B_202E_206A_200C_206C_202D_206D_206A_202C_200B_206F_200D_200E_206F_206E_202C_206F_206D_206B_206F_200C_202E_206D_200F_200F_200E_202A_202B_200B_200D_206C_206A_206F_200D_200B_200B_200D_202D_202E("Next body to map is ", _206E_200E_200C_202E_200F_200F_200B_202D_202D_206E_206B_202D_206E_206A_206A_202D_206D_202B_200D_206F_206B_206C_202E_202E_200E_202C_202B_200F_202A_206A_200F_206B_202D_200E_206F_206E_202E_202D_206B_202B_202E(Utils.NotificationBodyName(current.Bodies[num49].BodyName, current.Name, isTTS: true, includeSSMLtag: false), 5));
								num11 = 1640115548;
								continue;
							case 10u:
							{
								int num64;
								int num65;
								if (settings.Notifications.EfficientRoute.Any)
								{
									num64 = 287590030;
									num65 = num64;
								}
								else
								{
									num64 = 1150976439;
									num65 = num64;
								}
								num11 = num64 ^ (int)(num2 * 690062932);
								continue;
							}
							case 60u:
								systemIsCompleteOrABodyMapped = true;
								num11 = (int)(num2 * 833944268) ^ -186768122;
								continue;
							case 34u:
								text = _202E_206A_206B_202E_206A_200C_206C_202D_206D_206A_202C_200B_206F_200D_200E_206F_206E_202C_206F_206D_206B_206F_200C_202E_206D_200F_200F_200E_202A_202B_200B_200D_206C_206A_206F_200D_200B_200B_200D_202D_202E("Next body to map is ", _206E_200E_200C_202E_200F_200F_200B_202D_202D_206E_206B_202D_206E_206A_206A_202D_206D_202B_200D_206F_206B_206C_202E_202E_200E_202C_202B_200F_202A_206A_200F_206B_202D_200E_206F_206E_202E_202D_206B_202B_202E(Utils.NotificationBodyName(current.Bodies[num49].BodyName, current.Name), 5));
								num11 = 72417798;
								continue;
							case 37u:
								current.Bodies[_200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(val5)].UpdateValues();
								num11 = ((int)num2 * -1223800781) ^ -494526562;
								continue;
							case 26u:
							{
								int num60;
								int num61;
								if (settings.Notifications.AlwaysPersistent.Popup)
								{
									num60 = -246083846;
									num61 = num60;
								}
								else
								{
									num60 = -982091854;
									num61 = num60;
								}
								num11 = num60 ^ ((int)num2 * -461083611);
								continue;
							}
							case 35u:
								num11 = (int)(num2 * 347132836) ^ -1649810430;
								continue;
							case 46u:
								num11 = (int)(num2 * 1677703090) ^ -1908801492;
								continue;
							case 30u:
							{
								int num56;
								int num57;
								if (current.FullyMappedBonus > 0)
								{
									num56 = -907484844;
									num57 = num56;
								}
								else
								{
									num56 = -2015576695;
									num57 = num56;
								}
								num11 = num56 ^ ((int)num2 * -151486752);
								continue;
							}
							case 57u:
								num11 = (int)(num2 * 1172651472) ^ -1160662734;
								continue;
							case 15u:
							{
								int num50;
								int num51;
								if (current.Bodies.ContainsKey(num49))
								{
									num50 = 839638897;
									num51 = num50;
								}
								else
								{
									num50 = 1298101012;
									num51 = num50;
								}
								num11 = num50 ^ ((int)num2 * -737701830);
								continue;
							}
							case 59u:
								goto IL_1d0b;
							case 24u:
								num11 = (int)(num2 * 437142483) ^ -2071986201;
								continue;
							case 55u:
								num11 = ((int)num2 * -1513462217) ^ 0x284A9FBC;
								continue;
							case 52u:
								_200C_200E_206A_200D_202B_202D_200E_200B_202B_206A_200D_200B_200D_200B_200F_206A_206F_206B_202D_200E_202A_200E_206C_202A_206D_200B_200B_200C_202C_200E_202B_200C_206D_206F_200C_200F_200E_206A_200C_202E(Core, val18);
								num11 = ((int)num2 * -1560583213) ^ 0x72ADAF91;
								continue;
							case 17u:
							{
								int num47;
								int num48;
								if (_200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(val5) != nextToVisit[0])
								{
									num47 = 738210760;
									num48 = num47;
								}
								else
								{
									num47 = 841400031;
									num48 = num47;
								}
								num11 = num47 ^ ((int)num2 * -102898054);
								continue;
							}
							case 19u:
								goto IL_1da5;
							case 7u:
								return;
							}
							break;
							IL_194c:
							string detail = _202E_206A_206B_202E_206A_200C_206C_202D_206D_206A_202C_200B_206F_200D_200E_206F_206E_202C_206F_206D_206B_206F_200C_202E_206D_200F_200F_200E_202A_202B_200B_200D_206C_206A_206F_200D_200B_200B_200D_202D_202E("First body to map is ", _206E_200E_200C_202E_200F_200F_200B_202D_202D_206E_206B_202D_206E_206A_206A_202D_206D_202B_200D_206F_206B_206C_202E_202E_200E_202C_202B_200F_202A_206A_200F_206B_202D_200E_206F_206E_202E_202D_206B_202B_202E(Utils.NotificationBodyName(current.Bodies[num70].BodyName, current.Name), 5));
							string theString2 = _202E_206A_206B_202E_206A_200C_206C_202D_206D_206A_202C_200B_206F_200D_200E_206F_206E_202C_206F_206D_206B_206F_200C_202E_206D_200F_200F_200E_202A_202B_200B_200D_206C_206A_206F_200D_200B_200B_200D_202D_202E("First body to map is ", _206E_200E_200C_202E_200F_200F_200B_202D_202D_206E_206B_202D_206E_206A_206A_202D_206D_202B_200D_206F_206B_206C_202E_202E_200E_202C_202B_200F_202A_206A_200F_206B_202D_200E_206F_206E_202E_202D_206B_202B_202E(Utils.NotificationBodyName(current.Bodies[num70].BodyName, current.Name, isTTS: true, includeSSMLtag: false), 5));
							NotificationArgs val20 = _206D_200B_202C_202D_206C_200C_206D_202E_202A_200C_206C_206E_206C_200E_200F_206C_200C_200E_202D_200C_206F_200E_202C_200D_202D_200D_200E_206F_206B_206B_202E_202D_202E_200C_206E_200E_200F_206F_200E_200D_202E();
							val20.Title = "System Discovery Complete";
							val20.Detail = detail;
							val20.DetailSsml = Utils.SSMLify(theString2);
							val20.Rendering = settings.Notifications.EfficientRoute.Rendering;
							val20.Sender = ShortName;
							NotificationArgs val21 = val20;
							_200C_200E_206A_200D_202B_202D_200E_200B_202B_206A_200D_200B_200D_200B_200F_206A_206F_206B_202D_200E_202A_200E_206C_202A_206D_200B_200B_200C_202C_200E_202B_200C_206D_206F_200C_200F_200E_206A_200C_202E(Core, val21);
							int num83;
							if (!settings.Notifications.AlwaysPersistent.Popup)
							{
								num11 = 1143496466;
								num83 = num11;
							}
							else
							{
								num11 = 232189812;
								num83 = num11;
							}
							continue;
							IL_17b7:
							int num84;
							if (_200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(val5) != nextToVisit[0])
							{
								num11 = 1887940336;
								num84 = num11;
							}
							else
							{
								num11 = 1847049964;
								num84 = num11;
							}
							continue;
							IL_15a8:
							int num85;
							if (!isReadAll)
							{
								num11 = 1429029495;
								num85 = num11;
							}
							else
							{
								num11 = 1441292263;
								num85 = num11;
							}
							continue;
							IL_18c0:
							int num86;
							if (!isReadAll)
							{
								num11 = 1865025273;
								num86 = num11;
							}
							else
							{
								num11 = 1143496466;
								num86 = num11;
							}
						}
						goto IL_133b;
						IL_1da5:
						if (current.Active)
						{
							num11 = 412629771;
							num87 = num11;
						}
						else
						{
							num11 = 617614030;
							num87 = num11;
						}
						goto IL_1340;
						IL_0bf6:
						SetCurrentSystem(_202C_206E_206D_200C_206B_200C_200D_206D_202B_202E_202B_200E_202B_200F_206E_206F_206F_206B_206A_200B_206D_200E_200B_206E_206A_200B_202B_202A_202C_206D_200F_202A_200B_200F_200C_206F_200C_202E_200D_206E_202E(val22), _202E_202B_200F_202A_200C_200C_200B_202A_206B_200D_206B_202C_200E_200B_206B_206F_206E_206A_206D_202E_202D_200D_206B_206C_202D_206F_206E_200D_206C_206D_202A_202B_206E_206B_206D_206B_200E_202B_202A_202C_202E(val22), _206A_202B_202B_206E_206D_206B_202C_202C_202D_202D_200E_200C_202E_206E_206E_206B_202B_202A_202C_202A_202B_200F_206E_202E_206A_200D_202B_200F_206C_206D_206B_202D_202B_206C_206B_206D_206F_206D_206C_206E_202E((JournalBase)(object)val22));
						UpdateUI();
						num13 = 1722090218;
						goto IL_0863;
						IL_1d0b:
						if (!current.Active)
						{
							num11 = 1143496466;
							num90 = num11;
						}
						else
						{
							num11 = 1142351187;
							num90 = num11;
						}
						goto IL_1340;
						IL_1105:
						if (_206D_200C_206D_200F_200B_200F_200E_200E_202C_206B_206C_206C_202E_200D_206E_202D_206A_202B_202D_206B_200D_206D_206B_200E_206A_206D_200E_206F_206C_206E_200F_206B_200F_202E_206F_200D_202C_200B_200B_206B_202E(_206B_206F_202D_206F_202C_206E_202B_202E_202A_206B_206B_206A_206D_206E_206F_200F_202A_206E_206B_206F_206D_206D_202D_202E_200E_200B_202A_200F_202B_206C_202C_200E_202C_200C_200B_206E_202D_200D_206B_200D_202E(val12), "Hyperspace"))
						{
							num13 = 1731629327;
							num91 = num13;
						}
						else
						{
							num13 = 1350736485;
							num91 = num13;
						}
						goto IL_0863;
						IL_0b9c:
						tmpCarrierJumpFlags = JsonSerializer.Deserialize<TmpCarrierJumpFlags>(_206D_206E_202B_200C_200E_202D_202E_202D_200B_202D_200D_202D_206A_206C_202B_206C_202E_202C_200C_202B_202B_206D_206E_200E_200D_202A_206D_200C_206A_202E_202D_206C_202D_206F_202D_202D_206C_200F_202D_206E_202E((JournalBase)(object)val13));
						num13 = 1221858488;
						goto IL_0863;
						IL_100f:
						OdysseyLoaded = true;
						num13 = 827451084;
						goto IL_0863;
						IL_190b:
						DeleteHecateData();
						num11 = 1143496466;
						goto IL_1340;
						IL_18dc:
						DeleteHecateData();
						num11 = 139840004;
						goto IL_1340;
					}
					break;
				}
			}
		}
		catch (Exception ex)
		{
			logger.WriteException(_200F_206F_202C_206E_202A_206D_206A_200F_202C_200F_206F_206A_206D_202D_202E_206D_202D_200E_206E_200E_202D_206C_200D_202A_200E_206B_200B_206B_202C_202C_200E_202B_206A_206F_202E_200F_200D_206A_206F_200F_202E("[JournalEvent - ", _206E_202B_206D_200C_200C_206D_200F_200E_206E_206E_200C_206F_206B_200D_200F_202E_206C_206E_202D_200F_200D_206C_200C_202D_200B_202B_200C_206B_206A_200D_206C_206C_206C_200D_206F_200B_200E_206D_202E((JournalBase)(object)journal), "]"), ex);
		}
	}
static string _206C_206E_206D_206C_200D_200D_202A_206D_206F_206B_202C_202E_206D_206B_202D_206D_202A_200C_200B_200B_200C_202D_202D_200D_200D_200C_200B_206F_200E_206B_202D_200C_200C_200D_200E_200F_200B_200D_200E_202E_202E(string P_0, string P_1, string P_2)
	{
		return P_0.Replace(P_1, P_2);
	}
static bool _202E_200E_206C_202E_202C_200B_206A_206B_202E_206F_200C_200F_200E_206F_206B_202D_202B_206F_202B_202E_200C_206D_202B_200B_206D_200F_200C_202D_206D_206E_206F_206B_200D_200D_202A_206B_202C_206B_202D_200D_202E(Scan P_0)
	{
		return P_0.Landable;
	}
static string _200E_206B_202B_200F_202A_206F_200F_200F_206C_200F_206C_200D_200F_202E_206C_202C_202C_202A_206E_200B_202A_202B_200C_206D_206A_206C_206C_206F_202C_202C_202A_200B_202E_202E_200B_206E_206C_206E_202B_206A_202E(LoadGame P_0)
	{
		return P_0.Commander;
	}
static DateTime _206A_202B_202B_206E_206D_206B_202C_202C_202D_202D_200E_200C_202E_206E_206E_206B_202B_202A_202C_202A_202B_200F_206E_202E_206A_200D_202B_200F_206C_206D_206B_202D_202B_206C_206B_206D_206F_206D_206C_206E_202E(JournalBase P_0)
	{
		return P_0.TimestampDateTime;
	}
static bool _202C_206C_202A_206C_200F_202A_200E_202A_200C_202B_202D_206E_206E_200C_206A_206E_200F_200C_200F_206E_200C_200F_206B_202C_202E_206F_206D_206F_206A_206B_206B_206E_206C_200D_206F_202A_206B_202B_200F_206A_202E(string P_0)
	{
		return string.IsNullOrEmpty(P_0);
	}
static string _202E_202C_206F_206F_200F_202B_206B_200B_200F_202E_206E_202E_206E_200C_200E_206D_206F_202C_200D_206E_200E_206B_202A_206D_206A_206D_202B_200B_200C_202B_202E_202D_206C_202A_202D_202D_202A_202D_206F_206A_202E(Signal P_0)
	{
		return P_0.Type;
	}
static bool _200B_206A_202E_206A_200B_202E_200B_206F_200B_202B_206D_202B_206E_200D_206C_206D_206C_206E_206A_200B_200F_200E_200E_206C_202C_206F_200F_206D_206A_200E_200B_206D_206A_206A_206A_206A_202E_206D_206C_200D_202E(Location P_0)
	{
		return P_0.InSRV;
	}
static ulong _202D_202A_206D_202A_202E_202D_206F_200C_202E_200F_200D_200B_202D_202E_206E_206A_202D_206C_200E_202B_200C_200C_202E_206F_200E_206F_200D_202D_206C_200D_206F_202B_206D_206C_200E_202E_202A_202B_206A_202E_202E(ScanBaryCentre P_0)
	{
		return P_0.SystemAddress;
	}
static ulong _202E_202B_200F_202A_200C_200C_200B_202A_206B_200D_206B_202C_200E_200B_206B_206F_206E_206A_206D_202E_202D_200D_206B_206C_202D_206F_206E_200D_206C_206D_202A_202B_206E_206B_206D_206B_200E_202B_202A_202C_202E(FSDJump P_0)
	{
		return P_0.SystemAddress;
	}
static int _202E_200D_200B_202B_206B_206B_200E_206F_200D_202E_202D_200D_202E_202E_206E_206A_200E_200D_202E_200C_200D_206F_202C_200F_200D_206C_206B_202A_202E_200E_206F_200B_206E_200C_202B_206E_202D_200B_202A_206E_202E(ApproachBody P_0)
	{
		return P_0.BodyID;
	}
static int _206B_206B_206A_206C_202C_200D_206D_200C_200B_202C_200F_200F_202E_202C_200D_206C_206E_206C_206F_206E_206F_206B_206D_202D_206F_202C_206E_200F_206F_206D_206F_202B_200C_206C_206F_200F_200F_206A_202E_200C_202E(SAAScanComplete P_0)
	{
		return P_0.ProbesUsed;
	}
static ImmutableList<Signal> _202A_200E_200D_200D_206E_200D_200F_206C_200D_206A_206A_206E_200D_206A_202E_206D_200F_206B_206B_202E_202A_202E_206B_200F_200C_202E_206E_202A_206C_206F_206B_200E_200D_206A_206D_200B_200E_200D_200D_202D_202E(SAASignalsFound P_0)
	{
		return P_0.Signals;
	}
static string _200D_202E_206F_200F_206E_200C_206E_206B_206E_200E_206E_200C_206A_206B_202C_202A_206D_206B_206E_200E_200C_202A_206C_200F_202D_202C_200B_202D_202A_200C_202A_206E_206E_206D_200F_200F_200D_202C_206F_202C_202E(Scan P_0)
	{
		return P_0.BodyName;
	}
static string _206F_202D_202D_200D_206C_200D_202B_206C_202C_206A_206A_200F_202C_202C_200D_202D_202B_202D_202C_200C_200B_202C_202C_202E_206B_200C_202D_206C_206D_206C_202E_202B_200F_206F_206E_200B_206B_206D_200B_206A_202E(Scan P_0)
	{
		return P_0.PlanetClass;
	}
static ulong _202C_202E_202D_206F_206F_200E_200F_206A_202C_200C_206C_206E_202D_206C_202C_206B_206B_202C_200F_202C_202B_202B_202E_206D_206B_200C_200E_206A_206E_206A_206B_202A_206C_202D_206D_206F_206A_202D_200D_206F_202E(SAASignalsFound P_0)
	{
		return P_0.SystemAddress;
	}
static string _206E_200E_200C_202E_200F_200F_200B_202D_202D_206E_206B_202D_206E_206A_206A_202D_206D_202B_200D_206F_206B_206C_202E_202E_200E_202C_202B_200F_202A_206A_200F_206B_202D_200E_206F_206E_202E_202D_206B_202B_202E(string P_0, int P_1)
	{
		return P_0.Substring(P_1);
	}
static Guid _200C_200E_206A_200D_202B_202D_200E_200B_202B_206A_200D_200B_200D_200B_200F_206A_206F_206B_202D_200E_202A_200E_206C_202A_206D_200B_200B_200C_202C_200E_202B_200C_206D_206F_200C_200F_200E_206A_200C_202E(IObservatoryCore P_0, NotificationArgs P_1)
	{
		return P_0.SendNotification(P_1);
	}
static int _200D_200F_206D_200D_200B_200B_200F_200D_206A_206A_200E_206B_202E_200E_200C_202C_206F_202B_200B_202B_200C_200C_202D_200F_202B_202B_206B_200C_206D_202B_206C_206B_206C_206D_206B_206E_200F_202B_206A_206C_202E(Signal P_0)
	{
		return P_0.Count;
	}
static string _206B_206F_202D_206F_202C_206E_202B_202E_202A_206B_206B_206A_206D_206E_206F_200F_202A_206E_206B_206F_206D_206D_202D_202E_200E_200B_202A_200F_202B_206C_202C_200E_202C_200C_200B_206E_202D_200D_206B_200D_202E(StartJump P_0)
	{
		return P_0.JumpType;
	}
static ulong _200E_200B_200B_200E_202D_200D_202D_206F_206F_200B_202B_200C_206E_202D_206A_202B_206D_206E_206C_206C_206E_200B_206C_206B_200E_206E_200D_206B_200D_206E_206D_202D_200B_202B_200E_206A_206D_202B_200E_206A_202E(FSSAllBodiesFound P_0)
	{
		return P_0.SystemAddress;
	}
static bool _202E_206C_206F_206C_200D_200B_206E_202E_200B_200E_206C_202C_206D_200D_202E_206E_200E_206C_202E_206D_200E_200D_206B_206C_202B_202D_200D_202D_200F_206F_202E_200C_202B_200B_206B_202B_202E_202C_202D_202D_202E(LoadGame P_0)
	{
		return P_0.Odyssey;
	}
static int _200D_200F_200B_206B_206B_202B_202A_206D_200E_206D_202A_202B_202D_206D_200E_202A_200B_200B_202D_206D_202D_200E_202E_206B_206F_206E_200E_200B_202D_206E_200E_206F_200F_206D_206D_206B_206A_206C_200C_202E(SAAScanComplete P_0)
	{
		return P_0.BodyID;
	}
static int _202A_200C_200C_206F_200E_202C_200F_206A_206D_200B_206E_206B_206A_202D_206B_206D_206E_206B_202E_202E_202C_202D_206B_200B_200E_202E_202D_206D_202B_200B_206E_200F_202E_206D_202B_202D_206B_200D_202D_200E_202E(SAASignalsFound P_0)
	{
		return P_0.BodyID;
	}
static bool _202B_206E_206E_202D_202D_206A_202E_200D_200C_206B_206C_206A_206C_206F_206A_206B_200E_200D_200B_206F_202D_206A_200E_202C_200F_206B_202E_200F_200F_206A_206D_206C_200C_206F_206C_206D_200E_206E_200F_206C_202E(IObservatoryCore P_0)
	{
		return P_0.IsLogMonitorBatchReading;
	}
static bool _206D_200C_206D_200F_200B_200F_200E_200E_202C_206B_206C_206C_202E_200D_206E_202D_206A_202B_202D_206B_200D_206D_206B_200E_206A_206D_200E_206F_206C_206E_200F_206B_200F_202E_206F_200D_202C_200B_200B_206B_202E(string P_0, string P_1)
	{
		return P_0 == P_1;
	}
static string _202A_202B_206E_200D_202C_206C_200F_206C_202A_202C_202C_202C_202A_206E_206D_202B_202C_206B_202C_202E_200D_200C_206D_202D_202A_206E_206E_200B_200B_200D_200B_206F_202B_200F_206F_200E_202C_202A_202B_202B_202E(Location P_0)
	{
		return P_0.StarSystem;
	}
static string _200F_202E_202B_200C_206A_200B_200C_206E_200D_206D_200C_202E_206F_200E_200D_202E_206E_206B_206B_200C_200C_202C_202A_206F_202B_206B_202B_200E_206D_200D_200E_206B_202A_206D_202B_206C_200D_200B_206E_206D_202E(LoadGame P_0)
	{
		return P_0.FID;
	}
static ulong _202C_202C_206F_206F_206B_202A_206E_200B_206A_202C_200D_206A_200D_206D_200C_200B_206F_202C_206E_200D_200D_200D_206F_206E_202B_200E_206E_206E_200F_200C_202A_200F_200F_206E_202E_202D_202B_206D_202B_202B_202E(FSSDiscoveryScan P_0)
	{
		return P_0.SystemAddress;
	}
static bool _202D_202B_200D_200B_206C_202A_200C_200D_202A_200B_202D_200B_202E_206F_202B_206C_206F_200E_200C_206F_206E_206F_206A_206A_206C_200F_200B_202E_200D_206E_206A_206D_202D_200B_200F_202E_206D_200E_200F_206B_202E(string P_0, string P_1)
	{
		return P_0 != P_1;
	}
static void _206B_202C_200D_206E_202D_200B_206F_200E_206D_206F_202E_202C_200B_206F_200C_206D_200E_200D_200E_206B_206D_206C_200B_200B_200E_206B_200E_206C_206D_206A_202A_202A_206F_206D_202E_202C_202E_200C_206F_200C_202E(IObservatoryCore P_0, IObservatoryPlugin P_1, Guid P_2, PluginMessage P_3)
	{
		P_0.SendPluginMessage(P_1, P_2, P_3);
	}
static NotificationArgs _206D_200B_202C_202D_206C_200C_206D_202E_202A_200C_206C_206E_206C_200E_200F_206C_200C_200E_202D_200C_206F_200E_202C_200D_202D_200D_200E_206F_206B_206B_202E_202D_202E_200C_206E_200E_200F_206F_200E_200D_202E()
	{
		//IL_0000: Unknown result type (might be due to invalid IL or missing references)
		//IL_0006: Expected Obj, but got Unknown
		return new NotificationArgs();
	}
static string _202D_206D_206F_200B_206F_206B_206C_206B_200E_202E_206F_200E_202B_206D_200D_202D_200C_200C_200E_200D_200D_200F_206B_206D_202A_202B_202A_200B_200D_200F_200B_200B_200D_200C_202A_202B_206E_200E_200F_202B_202E(ScanBaryCentre P_0)
	{
		return P_0.StarSystem;
	}
static string _202C_206E_206D_200C_206B_200C_200D_206D_202B_202E_202B_200E_202B_200F_206E_206F_206F_206B_206A_200B_206D_200E_200B_206E_206A_200B_202B_202A_202C_206D_200F_202A_200B_200F_200C_206F_200C_202E_200D_206E_202E(FSDJump P_0)
	{
		return P_0.StarSystem;
	}
static bool _206C_200B_206C_202D_206D_200E_202C_200D_206A_206D_202C_200B_202A_206C_206C_200D_202C_202B_200D_206A_206A_200B_206B_206F_206A_206A_206C_206B_206C_200E_206B_206F_206A_206F_206E_202A_206D_202B_200B_206D_202E(Location P_0)
	{
		return P_0.OnFoot;
	}
static float _206B_200F_202E_200C_206C_200B_200C_200F_200F_202A_200C_200D_200B_200B_206F_202D_206D_206E_202D_202C_206B_206A_202A_200E_206C_200E_202D_206D_200F_200E_202B_200C_202E_202C_200E_200F_206C_200B_202B_206D_202E(Scan P_0)
	{
		return P_0.SurfacePressure;
	}
static string _202A_202B_206D_202D_206F_202E_206E_202D_206C_206D_202B_206F_206B_200F_202A_202A_206A_200B_206A_200C_202A_202C_202C_206F_202B_206E_200B_202A_206C_206E_200C_206C_206C_202B_200D_206A_202D_202B_206A_202D_202E(Scan P_0)
	{
		return P_0.AtmosphereType;
	}
static int _200B_200C_200D_202C_200D_202B_200E_200B_202E_200D_200C_200D_200B_206B_202E_206B_206A_206B_202A_206E_206C_206E_206C_200D_206D_206B_206C_200E_202D_200F_200E_202E_202B_202A_200C_202D_200D_200F_206D_202E_202E(ScanBaryCentre P_0)
	{
		return P_0.BodyID;
	}
static bool _200C_200E_200B_200B_200D_206F_202A_206D_202B_200F_206C_206A_206D_200D_200C_200B_200D_202A_206E_206E_200B_206B_200D_202E_202A_200D_202C_206D_202A_206E_202E_200F_202C_202C_200D_202D_206C_206E_200C_202E_202E(Disembark P_0)
	{
		return P_0.SRV;
	}
static PluginMessage _206A_200E_200E_200F_202D_202E_202C_202E_206F_200C_202C_206C_206B_200D_200F_202E_202D_200E_206E_206C_200C_206F_206F_206C_200D_206F_206B_202C_200D_202E_206B_206A_206B_206B_200F_206C_206C_206E_200B_200F_202E(string P_0, Dictionary<string, object> P_1, Guid? P_2)
	{
		//IL_0003: Unknown result type (might be due to invalid IL or missing references)
		//IL_0009: Expected Obj, but got Unknown
		return new PluginMessage(P_0, P_1, P_2);
	}
static ulong _202C_202A_200D_206E_202C_200C_206E_206E_202A_200F_200F_202D_200D_200B_200D_206B_200D_206A_206C_202C_200D_206A_200C_206E_206C_200B_206E_206F_206F_202A_206E_200B_202E_206D_206B_200E_200E_200E_200C_200C_202E(SAAScanComplete P_0)
	{
		return P_0.SystemAddress;
	}
static int _200E_206B_202A_200B_200F_200B_206D_206D_202D_206E_202A_200F_202A_206C_206F_202C_206C_200B_202A_206F_200F_206F_206E_202C_200C_206C_206D_200E_202A_206B_202D_202A_202A_206D_206B_206D_202C_202C_202B_202C_202E(FSSDiscoveryScan P_0)
	{
		return P_0.BodyCount;
	}
static string _200F_206F_202C_206E_202A_206D_206A_200F_202C_200F_206F_206A_206D_202D_202E_206D_202D_200E_206E_200E_202D_206C_200D_202A_200E_206B_200B_206B_202C_202C_200E_202B_206A_206F_202E_200F_200D_206A_206F_200F_202E(string P_0, string P_1, string P_2)
	{
		return P_0 + P_1 + P_2;
	}
static string _206D_206E_202B_200C_200E_202D_202E_202D_200B_202D_200D_202D_206A_206C_202B_206C_202E_202C_200C_202B_202B_206D_206E_200E_200D_202A_206D_200C_206A_202E_202D_206C_202D_206F_202D_202D_206C_200F_202D_206E_202E(JournalBase P_0)
	{
		return P_0.Json;
	}
static string _206E_202B_206D_200C_200C_206D_200F_200E_206E_206E_200C_206F_206B_200D_200F_202E_206C_206E_202D_200F_200D_206C_200C_202D_200B_202B_200C_206B_206A_200D_206C_206C_206C_200D_206F_200B_200E_206D_202E(JournalBase P_0)
	{
		return P_0.Event;
	}
static ulong _200D_206A_206D_206F_206F_206C_206E_202C_206C_206F_202C_202C_206C_206B_202A_206F_206D_202E_206B_206F_202E_206A_206C_202C_206B_206E_206B_206F_200B_206D_200C_200B_206F_200D_200B_200E_202E_206A_202B_206F_202E(Location P_0)
	{
		return P_0.SystemAddress;
	}
static string _202E_206A_206B_202E_206A_200C_206C_202D_206D_206A_202C_200B_206F_200D_200E_206F_206E_202C_206F_206D_206B_206F_200C_202E_206D_200F_200F_200E_202A_202B_200B_200D_206C_206A_206F_200D_200B_200B_200D_202D_202E(string P_0, string P_1)
	{
		return P_0 + P_1;
	}
static int _206F_202B_200D_200F_202E_202B_202E_206B_206E_202A_206B_206A_206C_200E_202A_202C_206F_206D_202C_202A_200B_200D_206D_202E_200B_202A_200B_202E_206A_206D_206A_200D_202C_200D_200B_200C_206A_200B_206B_206A_202E(SAAScanComplete P_0)
	{
		return P_0.EfficiencyTarget;
	}
static string _202D_200E_202D_206D_206F_202E_200F_202C_200B_202C_200C_202D_200D_200F_202A_202E_202A_206F_200D_200D_202E_206D_202C_202B_206E_202C_200E_206D_200C_206C_200B_206E_206F_200D_206B_202C_200E_202D_200E_202E(Scan P_0)
	{
		return P_0.StarType;
	}
}
public class Program{public static void Main(){var cases=new List<object>();foreach(var type in new Type[]{typeof(ApproachBody),typeof(CarrierJump),typeof(Died),typeof(Disembark),typeof(Embark),typeof(FSDJump),typeof(FSSAllBodiesFound),typeof(FSSBodySignals),typeof(FSSDiscoveryScan),typeof(LoadGame),typeof(Location),typeof(MultiSellExplorationData),typeof(SAAScanComplete),typeof(SAASignalsFound),typeof(Scan),typeof(ScanBaryCentre),typeof(ScanOrganic),typeof(SellExplorationData),typeof(Shutdown),typeof(StartJump),typeof(SupercruiseExit)})for(int flags=0;flags<256;flags++){
 var e=new Evaluator(); e.Core.IsLogMonitorBatchReading=(flags&2)!=0; e.isReadAll=(flags&4)!=0;e.transportMode=(flags&8)!=0?TransportMode.SRV:TransportMode.Ship;
 var j=(JournalBase)Activator.CreateInstance(type);j.Event=type.Name;j.SystemAddress=(flags&1)!=0?99UL:42UL;j.OnFoot=(flags&16)!=0;j.Docked=(flags&32)!=0;j.SRV=(flags&8)!=0;j.InSRV=(flags&8)!=0;j.Odyssey=(flags&16)!=0;j.JumpType=(flags&128)!=0?"Supercruise":"Hyperspace";
 j.Json=JsonSerializer.Serialize(new{j.OnFoot,j.Docked});j.Signals=ImmutableList.Create(new Signal{Type="$SAA_SignalType_Biological;",Count=3},new Signal{Type="$SAA_SignalType_Geological;",Count=2});
 e.current.Bodies[1].IsMapped=(flags&64)!=0;
 e.JournalEvent(j);
 cases.Add(new{Event=type.Name,Flags=flags,Actions=e.Trace,Current=e.current.SystemAddress,Transport=(int)e.transportMode,Odyssey=e.OdysseyLoaded,LastVisited=e.lastVisited,Complete=e.systemIsCompleteOrABodyMapped,Active=e.current.Active,Total=e.current.TotalBodies,Mapped=e.current.Bodies[1].IsMapped,Efficient=e.current.Bodies[1].IsMappedEfficiently});
 }var statusCases=new List<object>();for(int previous=0;previous<=10;previous++)for(int focus=0;focus<=10;focus++)for(int flags=0;flags<64;flags++){
 var e=new Evaluator();e.lastGUIStatus=(FocusStatus)previous;e.nextBodyNotification=(flags&1)!=0?new Guid("00000000-0000-0000-0000-000000000001"):null;e.settings.Notifications.AlwaysPersistent.Popup=(flags&2)!=0;e.settings.Notifications.DSSPersistent.Popup=(flags&4)!=0;e.settings.SetTargetedInFSSWorthMapping=(flags&8)!=0;e.current.Bodies[1].IsStar=(flags&16)!=0;
 e.StatusChange(new Status{GuiFocus=(FocusStatus)focus,Destination=new Destination{System=(flags&32)!=0?99UL:42UL,Body=1}});
 statusCases.Add(new{previous,focus,flags,Actions=e.Trace,Override=e.current.Bodies[1].OverrideIsWorthMapping});
 }var persistent=new List<object>();for(int flags=0;flags<128;flags++){
 var e=new Evaluator();e.nextBodyNotification=(flags&1)!=0?new Guid("00000000-0000-0000-0000-000000000001"):null;e.settings.Notifications.AlwaysPersistent.Popup=(flags&2)!=0;e.settings.Notifications.DSSPersistent.Popup=(flags&4)!=0;e.Core.IsLogMonitorBatchReading=(flags&8)!=0;e.lastGUIStatus=(flags&16)!=0?(FocusStatus)10:(FocusStatus)0;if((flags&32)!=0)e.nextToVisit.Clear();e.UpdatePersistentNotification((flags&64)!=0);persistent.Add(new{flags,Actions=e.Trace});
 }var journalNotifications=new List<object>();foreach(var type in new Type[]{typeof(FSSAllBodiesFound),typeof(SAAScanComplete),typeof(StartJump),typeof(SupercruiseExit),typeof(Shutdown)})for(int flags=0;flags<64;flags++){
 var e=new Evaluator();e.settings.Notifications.AlwaysPersistent.Popup=(flags&1)!=0;e.settings.Notifications.DSSPersistent.Popup=(flags&2)!=0;e.settings.Notifications.EfficientRoute.Any=(flags&4)!=0;e.isReadAll=(flags&8)!=0;e.nextBodyNotification=(flags&16)!=0?new Guid("00000000-0000-0000-0000-000000000001"):null;
 var j=(JournalBase)Activator.CreateInstance(type);j.Event=type.Name;if((flags&32)!=0)j.JumpType="Supercruise";e.JournalEvent(j);journalNotifications.Add(new{Event=type.Name,flags,Actions=e.Trace});
 }Console.WriteLine(JsonSerializer.Serialize(new{journal=cases,status=statusCases,persistent,journalNotifications}));}}