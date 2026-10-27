using System;using System.Collections.Generic;using System.Reflection;using System.Runtime.CompilerServices;using EDJP.Observatory;using Observatory.Framework;using Observatory.Framework.Interfaces;namespace Evaluator;public class Evaluator:BaseWorker,IObservatoryPlugin{
public FakeCore Core=new();public EvaluatorSettings settings=new();public EDSystem current=new("",0,DateTime.MinValue,new());public List<EDSystem>systems=new();public bool isPreReading=false;public HashSet<int> missingButNeeded=new();public int lastVisited;public bool retryProcessOrrery,systemIsCompleteOrABodyMapped;public Orrery orrery=new();public UIHandler uiHandler=new();public List<int>nextToVisit=new();public int currentFIDAsInt=123;public long UnsoldTotal;public void Delete()=>DeleteHecateData();public bool hasAllKeys(Dictionary<string,object>d,List<string>keys){foreach(var k in keys)if(!d.ContainsKey(k))return false;return true;}
public void SetCurrentSystem(string theSys, ulong theId64, DateTime theDT)
	{
		EDSystem item = new EDSystem(theSys, theId64, theDT, logger);
		while (true)
		{
			int num = -1055426642;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -288773840)) % 13)
				{
				case 0u:
					break;
				default:
					return;
				case 7u:
					missingButNeeded = new HashSet<int>();
					lastVisited = -1;
					retryProcessOrrery = false;
					num = (int)((num2 * 311568254) ^ 0x42C6EF19);
					continue;
				case 6u:
				{
					current = item;
					int num7;
					int num8;
					if (!_202B_206E_206E_202D_202D_206A_202E_200D_200C_206B_206C_206A_206C_206F_206A_206B_200E_200D_200B_206F_202D_206A_200E_202C_200F_206B_202E_200F_200F_206A_206D_206C_200C_206F_206C_206D_200E_206E_200F_206C_202E(Core))
					{
						num7 = 223953522;
						num8 = num7;
					}
					else
					{
						num7 = 171879952;
						num8 = num7;
					}
					num = num7 ^ ((int)num2 * -678787939);
					continue;
				}
				case 5u:
				{
					int num4;
					int num5;
					if (!isPreReading)
					{
						num4 = 1887200587;
						num5 = num4;
					}
					else
					{
						num4 = 822497789;
						num5 = num4;
					}
					num = num4 ^ (int)(num2 * 1471551919);
					continue;
				}
				case 4u:
				{
					Dictionary<string, object> dictionary = new Dictionary<string, object>
					{
						{
							"FID",
							currentFIDAsInt
						},
						{
							"ID64",
							theId64
						},
						{
							"DateTime",
							theDT
						},
						{
							"Name",
							theSys
						}
					};
					_206B_202C_200D_206E_202D_200B_206F_200E_206D_206F_202E_202C_200B_206F_200C_206D_200E_200D_200E_206B_206D_206C_200B_200B_200E_206B_200E_206C_206D_206A_202A_202A_206F_206D_202E_202C_202E_200C_206F_200C_202E(Core, (IObservatoryPlugin)(object)this, PluginID.Hecate, _206A_200E_200E_200F_202D_202E_202C_202E_206F_200C_202C_206C_206B_200D_200F_202E_202D_200E_206E_206C_200C_206F_206F_206C_200D_206F_206B_202C_200D_202E_206B_206A_206B_206B_200F_206C_206C_206E_200B_200F_202E("EvaluatorSysAdd", dictionary, (Guid?)null));
					int num6;
					if (_202B_206E_206E_202D_202D_206A_202E_200D_200C_206B_206C_206A_206C_206F_206A_206B_200E_200D_200B_206F_202D_206A_200E_202C_200F_206B_202E_200F_200F_206A_206D_206C_200C_206F_206C_206D_200E_206E_200F_206C_202E(Core))
					{
						num = -931439082;
						num6 = num;
					}
					else
					{
						num = -1799986763;
						num6 = num;
					}
					continue;
				}
				case 10u:
					systems = new List<EDSystem>();
					num = ((int)num2 * -1303612188) ^ 0x6F216BC;
					continue;
				case 11u:
					systemIsCompleteOrABodyMapped = false;
					num = ((int)num2 * -2043456880) ^ 0x11135A72;
					continue;
				case 3u:
					systems.Add(item);
					nextToVisit = new List<int>();
					num = -504312650;
					continue;
				case 1u:
				{
					int num3;
					if (systems.Count <= _206D_202E_206A_202A_206C_206A_202C_206D_202B_200B_206E_200D_206C_200E_200D_200B_202E_206D_206F_202C_200B_200C_206E_200D_206F_200C_206D_206E_200E_202A_202C_200F_200C_206B_200C_202A_206A_202B_202D_200C_202E(settings.MaxSystems, 1))
					{
						num = -1580664456;
						num3 = num;
					}
					else
					{
						num = -974794802;
						num3 = num;
					}
					continue;
				}
				case 9u:
					orrery = new Orrery();
					num = ((int)num2 * -1106320426) ^ -954184557;
					continue;
				case 2u:
					uiHandler.ExpireAlerts();
					num = ((int)num2 * -369827769) ^ 0x217762E6;
					continue;
				case 8u:
					systems.RemoveAt(0);
					num = (int)((num2 * 1514665451) ^ 0x5D0D5852);
					continue;
				case 12u:
					return;
				}
				break;
			}
		}
	}
public void HandleEvaluatorSetTotal(MessageSender sender, Dictionary<string, object> payload)
	{
		bool flag = default;

		if (!hasAllKeys(payload, new List<string> { "Total" }))
		{
			goto IL_0023;
		}
		goto IL_011a;
		IL_0023:
		int num = 2013140133;
		goto IL_0028;
		IL_0028:
		while (true)
		{
			uint num2;
			switch ((num2 = (uint)(num ^ 0x62FB99B4)) % 9)
			{
			case 2u:
				break;
			default:
				return;
			case 3u:
				goto IL_0061;
			case 0u:
			{
				int num5;
				int num6;
				if (!flag)
				{
					num5 = -1785566210;
					num6 = num5;
				}
				else
				{
					num5 = -661129173;
					num6 = num5;
				}
				num = num5 ^ ((int)num2 * -1037136367);
				continue;
			}
			case 4u:
				UnsoldTotal -= current.TotalValue;
				num = ((int)num2 * -1559237839) ^ -604546600;
				continue;
			case 6u:
			{
				int num3;
				int num4;
				if (UnsoldTotal >= current.TotalValue)
				{
					num3 = 863459919;
					num4 = num3;
				}
				else
				{
					num3 = 1761955235;
					num4 = num3;
				}
				num = num3 ^ (int)(num2 * 53068163);
				continue;
			}
			case 8u:
				goto IL_011a;
			case 7u:
				base.logger.WriteLog("HandleEvaluatorSetTotal", _206D_202C_202C_200C_202C_200E_200F_206E_206E_202E_200C_206B_202A_200F_206E_206D_202A_200E_206A_206A_200D_202D_206A_200D_200D_206D_206E_206E_202D_200C_200F_206F_202C_206C_200F_206A_202C_206F_206D_202E("called with missing keys by ", _200E_206F_200B_202E_202D_200E_206C_206A_202E_200F_206C_200F_206D_200D_200B_200F_200B_202A_206E_206F_200D_202C_200F_200B_206D_200F_206A_200C_202B_206B_202C_200C_206E_202C_202B_202B_206D_200F_206A_206E_202E(sender), " v", _200C_206F_200F_206F_200E_206B_206D_206F_206B_206A_206F_206F_200D_200E_202B_202A_206E_200F_200C_202D_202D_202C_202A_200C_206D_202D_206B_202C_200D_202A_206B_200B_202A_200B_200D_202D_202A_200B_200D_200D_202E(sender)), LogLevel.Verbose);
				return;
			case 1u:
			{
				Logger logger = base.logger;
				DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(26, 2);
				defaultInterpolatedStringHandler.AppendLiteral("Evaluator total set to ");
				defaultInterpolatedStringHandler.AppendFormatted(UnsoldTotal);
				defaultInterpolatedStringHandler.AppendLiteral(" [");
				defaultInterpolatedStringHandler.AppendFormatted(flag);
				defaultInterpolatedStringHandler.AppendLiteral("]");
				logger.WriteLog(defaultInterpolatedStringHandler.ToStringAndClear(), LogLevel.Verbose);
				num = 1352317563;
				continue;
			}
			case 5u:
				return;
			}
			break;
		}
		goto IL_0023;
		IL_007b:
		int num7;
		flag = (byte)num7 != 0;
		UnsoldTotal = (long)payload["Total"];
		num = 1110868970;
		goto IL_0028;
		IL_011a:
		if (!payload.ContainsKey("SubtractCurrent"))
		{
			num7 = 0;
			goto IL_007b;
		}
		num = 674545126;
		goto IL_0028;
		IL_0061:
		num7 = (((bool)payload["SubtractCurrent"]) ? 1 : 0);
		goto IL_007b;
	}
private void DeleteHecateData()
	{
		Dictionary<string, object> dictionary = new Dictionary<string, object> { 
		{
			"FID",
			currentFIDAsInt
		} };
		_206B_202C_200D_206E_202D_200B_206F_200E_206D_206F_202E_202C_200B_206F_200C_206D_200E_200D_200E_206B_206D_206C_200B_200B_200E_206B_200E_206C_206D_206A_202A_202A_206F_206D_202E_202C_202E_200C_206F_200C_202E(Core, (IObservatoryPlugin)(object)this, PluginID.Hecate, _206A_200E_200E_200F_202D_202E_202C_202E_206F_200C_202C_206C_206B_200D_200F_202E_202D_200E_206E_206C_200C_206F_206F_206C_200D_206F_206B_202C_200D_202E_206B_206A_206B_206B_200F_206C_206C_206E_200B_200F_202E("EvaluatorDelete", dictionary, (Guid?)null));
		while (true)
		{
			int num = 947319139;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0x4D9F6673)) % 3)
				{
				case 2u:
					break;
				default:
					return;
				case 1u:
					goto IL_0071;
				case 0u:
					return;
				}
				break;
				IL_0071:
				UnsoldTotal = 0L;
				num = ((int)num2 * -1122525621) ^ 0x290C89A9;
			}
		}
	}
static string _200C_206F_200F_206F_200E_206B_206D_206F_206B_206A_206F_206F_200D_200E_202B_202A_206E_200F_200C_202D_202D_202C_202A_200C_206D_202D_206B_202C_200D_202A_206B_200B_202A_200B_200D_202D_202A_200B_200D_200D_202E(MessageSender P_0)
	{
		return P_0.Version;
	}
static void _206B_202C_200D_206E_202D_200B_206F_200E_206D_206F_202E_202C_200B_206F_200C_206D_200E_200D_200E_206B_206D_206C_200B_200B_200E_206B_200E_206C_206D_206A_202A_202A_206F_206D_202E_202C_202E_200C_206F_200C_202E(IObservatoryCore P_0, IObservatoryPlugin P_1, Guid P_2, PluginMessage P_3)
	{
		P_0.SendPluginMessage(P_1, P_2, P_3);
	}
static PluginMessage _206A_200E_200E_200F_202D_202E_202C_202E_206F_200C_202C_206C_206B_200D_200F_202E_202D_200E_206E_206C_200C_206F_206F_206C_200D_206F_206B_202C_200D_202E_206B_206A_206B_206B_200F_206C_206C_206E_200B_200F_202E(string P_0, Dictionary<string, object> P_1, Guid? P_2)
	{
		//IL_0003: Unknown result type (might be due to invalid IL or missing references)
		//IL_0009: Expected Obj, but got Unknown
		return new PluginMessage(P_0, P_1, P_2);
	}
static string _200E_206F_200B_202E_202D_200E_206C_206A_202E_200F_206C_200F_206D_200D_200B_200F_200B_202A_206E_206F_200D_202C_200F_200B_206D_200F_206A_200C_202B_206B_202C_200C_206E_202C_202B_202B_206D_200F_206A_206E_202E(MessageSender P_0)
	{
		return P_0.ShortName;
	}
static string _206D_202C_202C_200C_202C_200E_200F_206E_206E_202E_200C_206B_202A_200F_206E_206D_202A_200E_206A_206A_200D_202D_206A_200D_200D_206D_206E_206E_202D_200C_200F_206F_202C_206C_200F_206A_202C_206F_206D_202E(string P_0, string P_1, string P_2, string P_3)
	{
		return P_0 + P_1 + P_2 + P_3;
	}
static bool _202B_206E_206E_202D_202D_206A_202E_200D_200C_206B_206C_206A_206C_206F_206A_206B_200E_200D_200B_206F_202D_206A_200E_202C_200F_206B_202E_200F_200F_206A_206D_206C_200C_206F_206C_206D_200E_206E_200F_206C_202E(IObservatoryCore P_0)
	{
		return P_0.IsLogMonitorBatchReading;
	}
static int _206D_202E_206A_202A_206C_206A_202C_206D_202B_200B_206E_200D_206C_200E_200D_200B_202E_206D_206F_202C_200B_200C_206E_200D_206F_200C_206D_206E_200E_202A_202C_200F_200C_206B_200C_202A_206A_202B_202D_200C_202E(int P_0, int P_1)
	{
		return Math.Max(P_0, P_1);
	}
}