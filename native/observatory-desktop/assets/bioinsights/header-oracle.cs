using System;using System.Text;using System.Globalization;using System.Text.Json;using System.Collections.Generic;
class Options { public bool ShowHeaders; public bool ShowTotal;public bool ShowBodyFields; }
class OriginalHeader {
 Options bioInsightsSettings=new();StringBuilder currentPage=new();string currentCmdr="Reference Commander";
 NumberFormatInfo nfi=new(){NumberGroupSeparator=" ",NumberDecimalDigits=0};
public void AddHTMLTableHeaders(bool isReadAllRequired, long total)
	{
        StringBuilder stringBuilder = default;
		if (bioInsightsSettings.ShowHeaders)
		{
			goto IL_0010;
		}
		goto IL_00bc;
		IL_0010:
		int num = -512226580;
		goto IL_0015;
		IL_0015:
		
		while (true)
		{
			uint num2;
			switch ((num2 = (uint)(num ^ -161843476)) % 22)
			{
			case 8u:
				break;
			default:
				return;
			case 18u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th class=\"border\">Genus</th>");
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=2 class=\"border\">Species</th>");
				num = -268931338;
				continue;
			case 19u:
				goto IL_00bc;
			case 7u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=2 class=\"extra border\">Volcanism</th>");
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th class=\"extra border\"> Gravity</th>");
				num = -201948253;
				continue;
			case 1u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=3 class=\"extra border\">Atmosphere Type</th>");
				num = -488538705;
				continue;
			case 6u:
			{
				int num5;
				int num6;
				if (!bioInsightsSettings.ShowHeaders)
				{
					num5 = -416473925;
					num6 = num5;
				}
				else
				{
					num5 = -2003147409;
					num6 = num5;
				}
				num = num5 ^ (int)(num2 * 345627966);
				continue;
			}
			case 20u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=2 class=\"border\">Status</th>");
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=2 class=\"border\">Variants</th>");
				num = -2140099728;
				continue;
			case 12u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E("<tr><th class=\"banner\" colspan=11>CMDR ", currentCmdr, "</th></tr>"));
				num = -1906409393;
				continue;
			case 2u:
				num = -480994578;
				continue;
			case 9u:
				goto IL_01e1;
			case 0u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<tr><td>&nbsp;</td>");
				num = -1648224213;
				continue;
			case 16u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th class=\"left border\">Value</th>");
				num = -810207218;
				continue;
			case 10u:
				stringBuilder = currentPage;
				num = ((int)num2 * -1134431491) ^ 0x6893B203;
				continue;
			case 13u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=2 class=\"extra border\">Pressure</th>");
				num = -856706513;
				continue;
			case 14u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=2 class=\"left border\">Dist</th></tr>");
				num = -677249337;
				continue;
			case 4u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<tr><td>&nbsp;</td>");
				num = -506327224;
				continue;
			case 17u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<tr><th class=\"body\" colspan=4>Body</th>");
				num = -961416817;
				continue;
			case 21u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th class=\"bodytype\" colspan=1>Type</th>");
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=1 class=\"signals\">Signals</th>");
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=3 class=\"body\">Radiant Star</th>");
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=2 class=\"value\">Body Value</th></tr>");
				num = -1565826719;
				continue;
			case 5u:
			{
				int num3;
				int num4;
				if (bioInsightsSettings.ShowBodyFields)
				{
					num3 = -676994496;
					num4 = num3;
				}
				else
				{
					num3 = -902504020;
					num4 = num3;
				}
				num = num3 ^ ((int)num2 * -1171450022);
				continue;
			}
			case 11u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<th colspan=2 class=\"extra border\">Temp</th>");
				num = -778300610;
				continue;
			case 3u:
				_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(currentPage, "<tr><td colspan=11>&nbsp;</td>");
				num = -572913796;
				continue;
			case 15u:
				return;
			}
			break;
			IL_01e1:
			StringBuilder stringBuilder2 = stringBuilder;
			StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(63, 1, stringBuilder);
			appendInterpolatedStringHandler.AppendLiteral("<tr><th class=\"banner right\" colspan=11>Unsold Data: ");
			appendInterpolatedStringHandler.AppendFormatted(total.ToString("n", nfi));
			appendInterpolatedStringHandler.AppendLiteral("</th></tr>");
			_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder2, ref appendInterpolatedStringHandler);
			int num7;
			if (bioInsightsSettings.ShowHeaders)
			{
				num = -572913796;
				num7 = num;
			}
			else
			{
				num = -1022745059;
				num7 = num;
			}
		}
		goto IL_0010;
		IL_00bc:
		int num8;
		if (bioInsightsSettings.ShowTotal)
		{
			num = -2071128650;
			num8 = num;
		}
		else
		{
			num = -572913796;
			num8 = num;
		}
		goto IL_0015;
	}

	

	static string _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(string P_0, string P_1, string P_2)
	{
		return P_0 + P_1 + P_2;
	}

	static StringBuilder _206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(StringBuilder P_0, string P_1)
	{
		return P_0.Append(P_1);
	}

	static StringBuilder _200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(StringBuilder P_0, ref StringBuilder.AppendInterpolatedStringHandler P_1)
	{
		return P_0.Append(ref P_1);
	}
 static void Main() {var results=new List<object>();foreach(bool headers in new[]{false,true})foreach(bool fields in new[]{false,true})foreach(bool total in new[]{false,true}) {
 var h=new OriginalHeader();h.bioInsightsSettings=new Options{ShowHeaders=headers,ShowBodyFields=fields,ShowTotal=total};h.AddHTMLTableHeaders(false,123456789);
 results.Add(new{show_headers=headers,show_body_fields=fields,show_total=total,html=h.currentPage.ToString()});}Console.WriteLine(JsonSerializer.Serialize(results));}
}
