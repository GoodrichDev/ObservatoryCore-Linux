using System;using System.Collections.Generic;using System.Text.Json;
enum FocusStatus{Default=0}
[Flags]enum StatusFlags:long{None=0}
class Status{public StatusFlags Flags{get;set;}}
class Popout{public bool? Visible;public void SetVisibility(bool vis){Visible=vis;}}
class BioInsightsSettings{
public bool PopoutAlwaysShow {get;set;}
public bool PopoutShowNoFocus {get;set;}
public bool PopoutShowLeftPanel {get;set;}
public bool PopoutShowRightPanel {get;set;}
public bool PopoutShowRolePanel {get;set;}
public bool PopoutShowSysMap {get;set;}
public bool PopoutShowFSSMode {get;set;}
public bool PopoutShowDSSMode {get;set;}
public bool PopoutHideUnlessSupercruiseOrSrv {get;set;}
public bool PopoutHidePopulated {get;set;}
}
class Original {
bool poppedout=true;Popout fpop=new();BioInsightsSettings bioInsightsSettings=new();
public void HandleGuiStatusChange(FocusStatus theStatus, FocusStatus lastStatus, Status status, long population)
	{
		//IL_0136: Unknown result type (might be due to invalid IL or missing references)
		//IL_0138: Invalid comparison between Unknown and I4
		//IL_020e: Unknown result type (might be due to invalid IL or missing references)
		//IL_0210: Invalid comparison between Unknown and I4
		//IL_02aa: Unknown result type (might be due to invalid IL or missing references)
		//IL_02ac: Invalid comparison between Unknown and I4
		//IL_01b8: Unknown result type (might be due to invalid IL or missing references)
		//IL_01ba: Invalid comparison between Unknown and I4
		//IL_014c: Unknown result type (might be due to invalid IL or missing references)
		//IL_014f: Invalid comparison between Unknown and I4
		//IL_0089: Unknown result type (might be due to invalid IL or missing references)
		//IL_030f: Unknown result type (might be due to invalid IL or missing references)
		//IL_01a1: Unknown result type (might be due to invalid IL or missing references)
		//IL_01a4: Invalid comparison between Unknown and I4
		//IL_0328: Unknown result type (might be due to invalid IL or missing references)
		if (!poppedout)
		{
			return;
		}
		bool flag = default;
		while (true)
		{
			int num = -1278123250;
			while (true)
			{
				uint num2;
				int num4;
				switch ((num2 = (uint)(num ^ -1239407753)) % 25)
				{
				case 18u:
					break;
				default:
					return;
				case 10u:
					if ((int)theStatus != 0)
					{
						num = ((int)num2 * -1273442469) ^ -1219746336;
						continue;
					}
					goto IL_0156;
				case 4u:
				{
					int num18;
					if (bioInsightsSettings.PopoutShowSysMap)
					{
						num = -1395777706;
						num18 = num;
					}
					else
					{
						num = -320958013;
						num18 = num;
					}
					continue;
				}
				case 6u:
					flag &= population == 0;
					num = ((int)num2 * -1950291169) ^ -558535833;
					continue;
				case 24u:
				{
					int num17;
					if (!bioInsightsSettings.PopoutShowRightPanel)
					{
						num = -1105781651;
						num17 = num;
					}
					else
					{
						num = -1402282430;
						num17 = num;
					}
					continue;
				}
				case 5u:
				{
					int num13;
					if (flag)
					{
						num = -1465617781;
						num13 = num;
					}
					else
					{
						num = -1349278380;
						num13 = num;
					}
					continue;
				}
				case 21u:
				{
					int num10;
					if (bioInsightsSettings.PopoutShowRolePanel)
					{
						num = -1436630778;
						num10 = num;
					}
					else
					{
						num = -1259383921;
						num10 = num;
					}
					continue;
				}
				case 0u:
					if ((int)theStatus != 2)
					{
						num = ((int)num2 * -232545787) ^ 0x52884833;
						continue;
					}
					goto IL_0156;
				case 9u:
					num4 = (((int)theStatus == 9) ? 1 : 0);
					goto IL_0157;
				case 8u:
					fpop.SetVisibility(flag);
					num = -965207897;
					continue;
				case 11u:
				{
					int num15;
					int num16;
					if (bioInsightsSettings.PopoutAlwaysShow)
					{
						num15 = -1805342870;
						num16 = num15;
					}
					else
					{
						num15 = -151017082;
						num16 = num15;
					}
					num = num15 ^ ((int)num2 * -1114250266);
					continue;
				}
				case 17u:
					if ((int)theStatus != 10)
					{
						num = (int)(num2 * 1751957959) ^ -1184766526;
						continue;
					}
					goto IL_0156;
				case 3u:
					if ((int)theStatus != 4)
					{
						num = (int)((num2 * 204319550) ^ 0x753601D1);
						continue;
					}
					goto IL_0156;
				case 23u:
				{
					int num6;
					int num7;
					if (flag)
					{
						num6 = 1562381702;
						num7 = num6;
					}
					else
					{
						num6 = 1176968072;
						num7 = num6;
					}
					num = num6 ^ ((int)num2 * -1650490649);
					continue;
				}
				case 14u:
				{
					int num3;
					if (!bioInsightsSettings.PopoutShowLeftPanel)
					{
						num = -866973520;
						num3 = num;
					}
					else
					{
						num = -130345584;
						num3 = num;
					}
					continue;
				}
				case 1u:
					if ((int)theStatus != 1)
					{
						num = (int)((num2 * 778702855) ^ 0x7BC5201E);
						continue;
					}
					goto IL_0156;
				case 7u:
					if (!bioInsightsSettings.PopoutShowFSSMode)
					{
						num4 = 0;
						goto IL_0157;
					}
					num = -1168904751;
					continue;
				case 12u:
				{
					int num14;
					if (bioInsightsSettings.PopoutShowDSSMode)
					{
						num = -343185337;
						num14 = num;
					}
					else
					{
						num = -1787345006;
						num14 = num;
					}
					continue;
				}
				case 13u:
					fpop.SetVisibility(vis: true);
					return;
				case 20u:
				{
					int num11;
					int num12;
					if (bioInsightsSettings.PopoutHidePopulated)
					{
						num11 = -1344308522;
						num12 = num11;
					}
					else
					{
						num11 = -1198132584;
						num12 = num11;
					}
					num = num11 ^ ((int)num2 * -1604275059);
					continue;
				}
				case 2u:
					if ((int)theStatus != 7)
					{
						num = ((int)num2 * -604989152) ^ -1250317085;
						continue;
					}
					goto IL_0156;
				case 15u:
				{
					int num8;
					int num9;
					if (!bioInsightsSettings.PopoutHideUnlessSupercruiseOrSrv)
					{
						num8 = 1669359561;
						num9 = num8;
					}
					else
					{
						num8 = 1186850720;
						num9 = num8;
					}
					num = num8 ^ ((int)num2 * -1567235307);
					continue;
				}
				case 19u:
				{
					int num5;
					if (bioInsightsSettings.PopoutShowNoFocus)
					{
						num = -1563741630;
						num5 = num;
					}
					else
					{
						num = -734208713;
						num5 = num;
					}
					continue;
				}
				case 16u:
					flag &= _202B_202B_206B_200F_200D_202B_200D_206E_206D_202A_206C_200E_202B_202C_206D_206D_206F_202E_200D_200C_202D_206D_200B_206A_200F_200F_206E_200F_202A_206E_206E_206F_202B_202B_206E_206F_200B_200B_202C_200F_202E((Enum)(object)_200D_202C_206B_200B_202D_200F_202C_200B_202E_206D_200C_200B_206F_202B_200C_200E_206D_202B_206B_200B_206D_200D_206B_206E_200D_200D_206F_200C_206B_200E_200D_200F_200E_200B_200F_200C_206A_200C_202B_202A_202E(status), (Enum)(object)(StatusFlags)16) || _202B_202B_206B_200F_200D_202B_200D_206E_206D_202A_206C_200E_202B_202C_206D_206D_206F_202E_200D_200C_202D_206D_200B_206A_200F_200F_206E_200F_202A_206E_206E_206F_202B_202B_206E_206F_200B_200B_202C_200F_202E((Enum)(object)_200D_202C_206B_200B_202D_200F_202C_200B_202E_206D_200C_200B_206F_202B_200C_200E_206D_202B_206B_200B_206D_200D_206B_206E_200D_200D_206F_200C_206B_200E_200D_200F_200E_200B_200F_200C_206A_200C_202B_202A_202E(status), (Enum)(object)(StatusFlags)67108864);
					num = -944862931;
					continue;
				case 22u:
					return;
					IL_0156:
					num4 = 1;
					goto IL_0157;
					IL_0157:
					flag = (byte)num4 != 0;
					num = -1407288348;
					continue;
				}
				break;
			}
		}
	}
static StatusFlags _200D_202C_206B_200B_202D_200F_202C_200B_202E_206D_200C_200B_206F_202B_200C_200E_206D_202B_206B_200B_206D_200D_206B_206E_200D_200D_206F_200C_206B_200E_200D_200F_200E_200B_200F_200C_206A_200C_202B_202A_202E(Status P_0)
	{
		//IL_0001: Unknown result type (might be due to invalid IL or missing references)
		return P_0.Flags;
	}
static bool _202B_202B_206B_200F_200D_202B_200D_206E_206D_202A_206C_200E_202B_202C_206D_206D_206F_202E_200D_200C_202D_206D_200B_206A_200F_200F_206E_200F_202A_206E_206E_206F_202B_202B_206E_206F_200B_200B_202C_200F_202E(Enum P_0, Enum P_1)
	{
		return P_0.HasFlag(P_1);
	}
static void Main(){var results=new List<object>();for(int mask=0;mask<1024;mask++){var o=new Original();o.bioInsightsSettings.PopoutAlwaysShow=(mask & (1<<0))!=0;o.bioInsightsSettings.PopoutShowNoFocus=(mask & (1<<1))!=0;o.bioInsightsSettings.PopoutShowLeftPanel=(mask & (1<<2))!=0;o.bioInsightsSettings.PopoutShowRightPanel=(mask & (1<<3))!=0;o.bioInsightsSettings.PopoutShowRolePanel=(mask & (1<<4))!=0;o.bioInsightsSettings.PopoutShowSysMap=(mask & (1<<5))!=0;o.bioInsightsSettings.PopoutShowFSSMode=(mask & (1<<6))!=0;o.bioInsightsSettings.PopoutShowDSSMode=(mask & (1<<7))!=0;o.bioInsightsSettings.PopoutHideUnlessSupercruiseOrSrv=(mask & (1<<8))!=0;o.bioInsightsSettings.PopoutHidePopulated=(mask & (1<<9))!=0;
var strings=new List<string>();foreach(long population in new long[]{0,1})foreach(long status in new long[]{0,16,67108864}){string output="";for(int focus=0;focus<12;focus++){o.fpop.Visible=null;o.HandleGuiStatusChange((FocusStatus)focus,(FocusStatus)0,new Status{Flags=(StatusFlags)status},population);output+=o.fpop.Visible==true?"1":o.fpop.Visible==false?"0":"?";}strings.Add(output);}results.Add(new{mask,visible=strings});}Console.WriteLine(JsonSerializer.Serialize(results));}}
