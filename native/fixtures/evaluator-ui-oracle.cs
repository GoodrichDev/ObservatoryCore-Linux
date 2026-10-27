using System;using System.Collections;using System.Collections.Generic;using System.Linq;using System.Runtime.CompilerServices;using System.Text.Json;using Observatory.Framework.Files.Journal;using EDJP.Observatory;
namespace Evaluator;
public class FakeCore {public bool IsLogMonitorBatchReading=>false;public void ExecuteOnUIThread(Action action){} }
public class UIHandler {
 public List<OutputLine> Rows=new();public List<object>Headers=new(); public void ResetPage(){Rows.Clear();Headers.Clear();} public void AddButtonsAndOpenContentDiv(){} public void AddHTMLAlerts(){} public void OpenHTMLTable(bool wide){} public void AddHTMLTableHeaders(long total){} public void AddHTMLSystemDetails(string s,int c,int m,int e,int b,UIFilter f,bool show){Headers.Add(new{Name=s,Current=show});} public void AddHTMLBodyDetails(OutputLine line,string row,bool show){Rows.Add(line);} public void AddHTMLTableFooter(){} public void CloseHTMLTable(){} public void CloseContentDiv(){} public void SetScrollTo(string page,string line){}
}
public class Evaluator {
 public static StubSettings EvalSettings=new(); public StubSettings settings=EvalSettings; public FakeCore Core=new();public List<EDSystem> systems=new();public UIHandler uiHandler=new();public EDSystem current=new(); public UIFilter UIFilter;public List<int> nextToVisit=new();public long UnsoldTotal;private void UpdateUIShowPage(){} private void ForceWebClientsToRefresh(){}
public void UpdateUI()
	{
		if (_202B_206E_206E_202D_202D_206A_202E_200D_200C_206B_206C_206A_206C_206F_206A_206B_200E_200D_200B_206F_202D_206A_200E_202C_200F_206B_202E_200F_200F_206A_206D_206C_200C_206F_206C_206D_200E_206E_200F_206C_202E(Core))
		{
			while (true)
			{
				uint num;
				switch ((num = 795415457u) % 3)
				{
				case 0u:
					continue;
				case 2u:
					return;
				}
				break;
			}
		}
		List<EDSystem> list = new List<EDSystem>(systems);
		uiHandler.ResetPage();
		uiHandler.AddButtonsAndOpenContentDiv();
		uiHandler.AddHTMLAlerts();
		uiHandler.OpenHTMLTable(settings.ExpandOutput);
		uiHandler.AddHTMLTableHeaders(UnsoldTotal + current.TotalValue);
		using (List<EDSystem>.Enumerator enumerator = list.GetEnumerator())
		{
			bool flag = default;
			int num3 = default;
			OutputLine outputLine = default;
			Body body = default;
			while (enumerator.MoveNext())
			{
				while (true)
				{
					EDSystem eDSystem = enumerator.Current;
					int num2 = 1038367458;
					while (true)
					{
						uint num;
						switch ((num = (uint)(num2 ^ 0x3DCA0E0D)) % 7)
						{
						case 2u:
							num2 = 1663814764;
							continue;
						case 3u:
							break;
						case 0u:
							goto IL_00f1;
						case 1u:
							goto IL_010e;
						case 5u:
							uiHandler.AddHTMLSystemDetails(eDSystem.Name, eDSystem.TotalValue, eDSystem.PotentialMaxValue, eDSystem.TotalBodies, eDSystem.Bodies.Count(), UIFilter, flag);
							num3 = 1;
							num2 = (int)((num * 746605267) ^ 0x23F0015);
							continue;
						case 6u:
							flag = eDSystem.SystemAddress == current.SystemAddress;
							num2 = (int)(num * 890602799) ^ -933913289;
							continue;
						default:
							goto IL_019b;
						}
						break;
						IL_019b:
						IEnumerator<Body> enumerator2 = (from i in eDSystem.Bodies
							orderby i.Key
							select i.Value).GetEnumerator();
						try
						{
							while (true)
							{
								IL_0283:
								int num4;
								int num5;
								if (!_202D_206F_202A_206D_206D_200F_200F_206D_206A_202D_202D_206F_206F_200D_202D_200E_202D_200E_200D_206D_202E_200B_202C_200E_202B_206A_200F_206A_200E_202A_206A_206D_200C_206C_206D_206A_202E_200B_206A_206B_202E((IEnumerator)enumerator2))
								{
									num4 = 1907431907;
									num5 = num4;
								}
								else
								{
									num4 = 2047091239;
									num5 = num4;
								}
								while (true)
								{
									int num10;
									switch ((num = (uint)(num4 ^ 0x3DCA0E0D)) % 12)
									{
									case 0u:
										num4 = 2047091239;
										continue;
									default:
										goto end_IL_01fa;
									case 5u:
									{
										UIHandler uIHandler = uiHandler;
										OutputLine q = outputLine;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(3, 1);
										defaultInterpolatedStringHandler.AppendLiteral("row");
										defaultInterpolatedStringHandler.AppendFormatted(num3);
										uIHandler.AddHTMLBodyDetails(q, defaultInterpolatedStringHandler.ToStringAndClear(), flag);
										num4 = 189713364;
										continue;
									}
									case 3u:
										break;
									case 4u:
									{
										string body2 = _202C_206B_200E_206C_202E_202D_202D_200D_202D_206D_200F_202E_206D_202D_206F_206F_202D_200B_202E_200E_206C_206F_206B_200D_200D_200B_202D_200D_206B_206E_200D_200D_200E_206F_206D_200F_200B_206D_206C_202C_202E(_206C_206E_206D_206C_200D_200D_202A_206D_206F_206B_202C_202E_206D_206B_202D_206D_202A_200C_200B_200B_200C_202D_202D_200D_200D_200C_200B_206F_200E_206B_202D_200C_200C_200D_200E_200F_200B_200D_200E_202E_202E(body.BodyName, eDSystem.Name, ""));
										outputLine = new OutputLine
										{
											IsStar = body.IsStar,
											IsLandable = body.IsLandable,
											Body = body2,
											Type = body.BodyType,
											Terraformable = (!body.IsStar && body.IsTerraformable),
											Distance = body.Distance,
											Gravity = body.Gravity,
											CurrentValue = eDSystem.GetBodyValue(body.BodyID, showMax: false),
											PotentialMaxValue = eDSystem.GetBodyValue(body.BodyID, showMax: true),
											Done = (!body.IsStar && body.IsMapped),
											Map = (body.WorthMapping || body.WorthVisiting),
											Order = body.VisitOrder,
											IsNextToVisit = (body.VisitOrder > 0 && nextToVisit.Count > 0 && body.BodyID == nextToVisit[0]),
											Extra = body.Extra,
											Volcanism = (body.HasVolcanism ? body.Volcanism : ""),
											ID64 = current.SystemAddress,
											BodyID = body.BodyID,
											WasDiscovered = body.WasDiscovered,
											WasMapped = body.WasMapped,
											WasFootfalled = body.WasFootfalled
										};
										int num16;
										if (UIFilter != UIFilter.MappingOnly)
										{
											num4 = 1816911096;
											num16 = num4;
										}
										else
										{
											num4 = 1300206591;
											num16 = num4;
										}
										continue;
									}
									case 11u:
									{
										int num8;
										int num9;
										if (!outputLine.Done)
										{
											num8 = 438911508;
											num9 = num8;
										}
										else
										{
											num8 = 1877863366;
											num9 = num8;
										}
										num4 = num8 ^ ((int)num * -697831254);
										continue;
									}
									case 2u:
									{
										int num13;
										int num14;
										if (!outputLine.Map)
										{
											num13 = 1960006438;
											num14 = num13;
										}
										else
										{
											num13 = 1489832432;
											num14 = num13;
										}
										num4 = num13 ^ ((int)num * -1213518940);
										continue;
									}
									case 1u:
										num10 = 1;
										goto IL_04a0;
									case 9u:
										if (num3 == 1)
										{
											num10 = 2;
											goto IL_04a0;
										}
										num4 = (int)(num * 693906906) ^ -1734552162;
										continue;
									case 6u:
									{
										body = enumerator2.Current;
										int num15;
										if (eDSystem.GetBodyValue(body.BodyID, showMax: true) < settings.MinimumBodyValue)
										{
											num4 = 1521457002;
											num15 = num4;
										}
										else
										{
											num4 = 1378873005;
											num15 = num4;
										}
										continue;
									}
									case 7u:
									{
										int num11;
										int num12;
										if (!eDSystem.ShowInInterface(body.BodyID))
										{
											num11 = -946076202;
											num12 = num11;
										}
										else
										{
											num11 = -288014274;
											num12 = num11;
										}
										num4 = num11 ^ (int)(num * 1764477429);
										continue;
									}
									case 8u:
									{
										int num6;
										int num7;
										if (UIFilter == UIFilter.All)
										{
											num6 = 2124680109;
											num7 = num6;
										}
										else
										{
											num6 = 889491498;
											num7 = num6;
										}
										num4 = num6 ^ (int)(num * 598268832);
										continue;
									}
									case 10u:
										goto end_IL_01fa;
										IL_04a0:
										num3 = num10;
										num4 = 428832042;
										continue;
									}
									goto IL_0283;
									continue;
									end_IL_01fa:
									break;
								}
								break;
							}
						}
						finally
						{
							if (enumerator2 != null)
							{
								while (true)
								{
									IL_0552:
									int num17 = 1354998801;
									while (true)
									{
										switch ((num = (uint)(num17 ^ 0x3DCA0E0D)) % 3)
										{
										case 2u:
											break;
										default:
											goto end_IL_0557;
										case 1u:
											goto IL_0575;
										case 0u:
											goto end_IL_0557;
										}
										goto IL_0552;
										IL_0575:
										_202B_202B_202E_200C_202B_202E_206A_200D_206B_200B_202E_202E_202B_202D_202B_202D_206B_200D_202C_202C_200D_206A_202D_206F_206E_206C_206F_202A_206F_200C_206A_200E_202B_202E_206E_200B_206B_202E_206A_200E_202E((IDisposable)enumerator2);
										num17 = ((int)num * -1876869552) ^ 0x7128C7A7;
										continue;
										end_IL_0557:
										break;
									}
									break;
								}
							}
						}
						goto end_IL_00e2;
						IL_010e:
						if (eDSystem.TotalValue <= 0)
						{
							goto end_IL_00e2;
						}
						num2 = ((int)num * -2106580374) ^ -963086167;
						continue;
						IL_00f1:
						if (eDSystem.SystemAddress == 0)
						{
							goto end_IL_00e2;
						}
						num2 = ((int)num * -1756490224) ^ 0x10A542C9;
					}
					continue;
					end_IL_00e2:
					break;
				}
			}
		}
		uiHandler.AddHTMLTableFooter();
		uiHandler.CloseHTMLTable();
		while (true)
		{
			int num18 = 273250248;
			while (true)
			{
				uint num;
				switch ((num = (uint)(num18 ^ 0x3DCA0E0D)) % 4)
				{
				case 3u:
					break;
				default:
					return;
				case 1u:
					uiHandler.CloseContentDiv();
					uiHandler.SetScrollTo("main", "currentsystem");
					num18 = 1424015081;
					continue;
				case 0u:
					_202D_202E_202A_200F_202B_200D_200D_202D_206F_202C_206D_200B_202A_206C_200B_202A_206F_206C_202C_206F_202D_202A_206B_206B_206C_206D_206A_202B_202E_202A_202B_200C_206E_202C_200D_202B_206A_200C_202B_202A_202E(Core, (Action)UpdateUIShowPage);
					ForceWebClientsToRefresh();
					num18 = (int)((num * 80453575) ^ 0x64CFC2A7);
					continue;
				case 2u:
					return;
				}
				break;
			}
		}
	}
static void _202B_202B_202E_200C_202B_202E_206A_200D_206B_200B_202E_202E_202B_202D_202B_202D_206B_200D_202C_202C_200D_206A_202D_206F_206E_206C_206F_202A_206F_200C_206A_200E_202B_202E_206E_200B_206B_202E_206A_200E_202E(IDisposable P_0)
	{
		P_0.Dispose();
	}
static bool _202B_206E_206E_202D_202D_206A_202E_200D_200C_206B_206C_206A_206C_206F_206A_206B_200E_200D_200B_206F_202D_206A_200E_202C_200F_206B_202E_200F_200F_206A_206D_206C_200C_206F_206C_206D_200E_206E_200F_206C_202E(FakeCore P_0)
	{
		return P_0.IsLogMonitorBatchReading;
	}
static string _202C_206B_200E_206C_202E_202D_202D_200D_202D_206D_200F_202E_206D_202D_206F_206F_202D_200B_202E_200E_206C_206F_206B_200D_200D_200B_202D_200D_206B_206E_200D_200D_200E_206F_206D_200F_200B_206D_206C_202C_202E(string P_0)
	{
		return P_0.TrimStart();
	}
static void _202D_202E_202A_200F_202B_200D_200D_202D_206F_202C_206D_200B_202A_206C_200B_202A_206F_206C_202C_206F_202D_202A_206B_206B_206C_206D_206A_202B_202E_202A_202B_200C_206E_202C_200D_202B_206A_200C_202B_202A_202E(FakeCore P_0, Action P_1)
	{
		P_0.ExecuteOnUIThread(P_1);
	}
static bool _202D_206F_202A_206D_206D_200F_200F_206D_206A_202D_202D_206F_206F_200D_202D_200E_202D_200E_200D_206D_202E_200B_202C_200E_202B_206A_200F_206A_200E_202A_206A_206D_200C_206C_206D_206A_202E_200B_206A_206B_202E(IEnumerator P_0)
	{
		return P_0.MoveNext();
	}
static string _206C_206E_206D_206C_200D_200D_202A_206D_206F_206B_202C_202E_206D_206B_202D_206D_202A_200C_200B_200B_200C_202D_202D_200D_200D_200C_200B_206F_200E_206B_202D_200C_200C_200D_200E_200F_200B_200D_200E_202E_202E(string P_0, string P_1, string P_2)
	{
		return P_0.Replace(P_1, P_2);
	}
}
