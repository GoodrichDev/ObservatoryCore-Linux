using System;using System.Text;using System.Globalization;using System.Runtime.CompilerServices;using System.Collections.Generic;using EDJP.Observatory;namespace Evaluator;public class UIHandler{public List<OutputSymbol> SymbolList=new(){OutputSymbol.Mapped,OutputSymbol.Terraformable,OutputSymbol.IsHighValue,OutputSymbol.IsLandable,OutputSymbol.HasBiologicals,OutputSymbol.BioInsightsPersonalCodex,OutputSymbol.BioInsightsGalacticCodex,OutputSymbol.BioInsightsMarked,OutputSymbol.BioInsightsHighValue,OutputSymbol.HasVolcanism,OutputSymbol.WasDiscovered,OutputSymbol.WasMapped,OutputSymbol.WasFootfalled,OutputSymbol.PluginWorthVisiting,OutputSymbol.NextBody};public void AddButtonsAndOpenContentDiv(StringBuilder p){} public void CloseContentDiv(StringBuilder p){} public void SavePageAndMaybeUpdate(string name,StringBuilder p,bool showIfCurrent){currentPage=p;}public StringBuilder currentPage=new();public string currentCmdr="Oracle";public string BaseWSURL="http://localhost/evaluator";public EvaluatorSettings evaluatorsettings=new();public EvaluatorSettings uisettings=>evaluatorsettings;public Theme theme=new();public NumberFormatInfo nfi=new(){NumberGroupSeparator=" ",NumberDecimalDigits=0};public string GetSymbol(OutputSymbol s,string text=null)=>"["+s+(text==null?"":":"+text)+"]";public static Dictionary<string, string> PlanetLookup = new Dictionary<string, string>
	{
		{
			"High metal content body",
			"HMC"
		},
		{
			"Metal rich body",
			"MR"
		},
		{
			"Rocky body",
			"ROCKY"
		},
		{
			"Icy body",
			"ICY"
		},
		{
			"Rocky ice body",
			"RIB"
		},
		{
			"Sudarsky class I gas giant",
			"GG1"
		},
		{
			"Sudarsky class II gas giant",
			"GG2"
		},
		{
			"Sudarsky class III gas giant",
			"GG3"
		},
		{
			"Sudarsky class IV gas giant",
			"GG4"
		},
		{
			"Sudarsky class V gas giant",
			"GG5"
		},
		{
			"Water world",
			"WW"
		},
		{
			"Gas giant with water based life",
			"GGWL"
		},
		{
			"Gas giant with ammonia based life",
			"GGAL"
		},
		{
			"Helium rich gas giant",
			"HRGG"
		},
		{
			"Helium gas giant",
			"HGG"
		},
		{
			"Earthlike body",
			"ELW"
		},
		{
			"Water giant",
			"WG"
		},
		{
			"Ammonia world",
			"AW"
		}
	};

	
public void OpenHTMLTable(bool wide)
	{
		StringBuilder stringBuilder = currentPage;
		StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(28, 1, stringBuilder);
		appendInterpolatedStringHandler.AppendLiteral("<div class=\"scroll\"><table ");
		appendInterpolatedStringHandler.AppendFormatted(wide ? "width=\"100%\"" : "");
		appendInterpolatedStringHandler.AppendLiteral(">");
		_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder, ref appendInterpolatedStringHandler);
	}
public void CloseHTMLTable()
	{
		_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<tr><td>&nbsp;</td></tr></table></div><span id=\"theend\"></span><br />");
	}
public void AddHTMLTableFooter()
	{
		if (((EvaluatorSettings)uisettings).ShowHeaders)
		{
			goto IL_0012;
		}
		goto IL_0057;
		IL_0012:
		int num = -2046079622;
		goto IL_0017;
		IL_0017:
		while (true)
		{
			uint num2;
			switch ((num2 = (uint)(num ^ -1061398573)) % 4)
			{
			case 0u:
				break;
			case 1u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<tr><th class=\"banner border\" colspan=10>&nbsp;</th>");
				num = -1765591943;
				continue;
			case 2u:
				goto IL_0057;
			default:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<tr><td colspan=10>&nbsp;</td></tr>");
				return;
			}
			break;
		}
		goto IL_0012;
		IL_0057:
		_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<tr><td colspan=10>&nbsp;</td></tr>");
		num = -1277834560;
		goto IL_0017;
	}
public void AddHTMLTableHeaders(long total)
	{
		StringBuilder stringBuilder = default;

		if (evaluatorsettings.ShowHeaders)
		{
			goto IL_0010;
		}
		goto IL_0356;
		IL_0010:
		int num = 793444668;
		goto IL_0015;
		IL_0015:
		while (true)
		{
			uint num2;
			switch ((num2 = (uint)(num ^ 0x66AA3804)) % 18)
			{
			case 9u:
				break;
			default:
				return;
			case 6u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, _202B_202B_200B_206F_206B_200C_206B_202B_206E_202E_202B_200D_206B_202C_206D_202A_202D_206C_206D_206A_206F_200C_200D_200C_206C_206C_202E_206D_200C_200F_206B_206B_206A_200E_200F_200E_200F_202B_206E_200C_202E("<tr><th class=\"bold border banner\" colspan=10>CMDR ", currentCmdr, "</th></tr>"));
				num = 682390243;
				continue;
			case 7u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border\">Type</th>");
				num = 998034673;
				continue;
			case 13u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border\">Attributes</th>");
				num = 171718728;
				continue;
			case 4u:
			{
				StringBuilder stringBuilder2 = stringBuilder;
				StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(81, 1, stringBuilder);
				appendInterpolatedStringHandler.AppendLiteral("<tr><th class=\"bold border banner right\" colspan=10>Unsold Data (Max): ");
				appendInterpolatedStringHandler.AppendFormatted(total.ToString("n", nfi));
				appendInterpolatedStringHandler.AppendLiteral("</th></tr>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder2, ref appendInterpolatedStringHandler);
				num = 841568296;
				continue;
			}
			case 11u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th colspan=3 class=\"bold border right\" title=\"Estimated current total value of system\">System Value</th>");
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th colspan=1 class=\"bold border right\" title=\"Estimated maximum value of system if all bodies worth mapping are mapped\">Potential Value</th></tr>");
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<tr><td class=\"indent\">&nbsp;</td>");
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border\">Body</th>");
				num = 1568124913;
				continue;
			case 0u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border\">Gravity</th>");
				num = 31671162;
				continue;
			case 12u:
			{
				int num3;
				int num4;
				if (evaluatorsettings.ShowHeaders)
				{
					num3 = -945799302;
					num4 = num3;
				}
				else
				{
					num3 = -943469448;
					num4 = num3;
				}
				num = num3 ^ ((int)num2 * -1899383684);
				continue;
			}
			case 2u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<tr><td colspan=11>&nbsp;</td>");
				num = 1087461930;
				continue;
			case 5u:
				stringBuilder = currentPage;
				num = ((int)num2 * -377836449) ^ -229536077;
				continue;
			case 8u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border\" colspan=2>Visit / Map</th>");
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border right\" title=\"Estimated current value of body\">Current</th>");
				num = 1698267273;
				continue;
			case 14u:
				goto IL_027d;
			case 3u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border\" colspan=1 title=\"Show mapped only | Default | Show all\">Filter</th>");
				num = 1801633877;
				continue;
			case 10u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border\">Distance</th>");
				num = 682524588;
				continue;
			case 16u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<tr><th class=\"bold border left\" colspan=4>System</th>");
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border\" colspan=1 title=\"Scanned bodies (FSS) / Total bodies\">Progress</th>");
				num = 1765979797;
				continue;
			case 1u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<th class=\"bold border right\" title=\"Estimated maximum value of body if mapped\">Mapped</th>");
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "</tr>");
				num = 1694515635;
				continue;
			case 17u:
				goto IL_0356;
			case 15u:
				return;
			}
			break;
			IL_027d:
			int num5;
			if (!evaluatorsettings.ShowHeaders)
			{
				num = 1694515635;
				num5 = num;
			}
			else
			{
				num = 1770764990;
				num5 = num;
			}
		}
		goto IL_0010;
		IL_0356:
		int num6;
		if (evaluatorsettings.ShowTotal)
		{
			num = 1669427273;
			num6 = num;
		}
		else
		{
			num = 1087461930;
			num6 = num;
		}
		goto IL_0015;
	}
public void AddHTMLSystemDetails(string system, int cvalue, int mvalue, int expectedBodies, int currentBodies, UIFilter uih, bool showFilter)
	{
		StringBuilder stringBuilder = default;

		if (showFilter)
		{
			goto IL_0007;
		}
		goto IL_0248;
		IL_0007:
		int num = 1634059974;
		goto IL_000c;
		IL_000c:
		StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler;
		while (true)
		{
			uint num2;
			switch ((num2 = (uint)(num ^ 0x513F7DD4)) % 18)
			{
			case 10u:
				break;
			default:
				return;
			case 8u:
				num = (int)((num2 * 148547734) ^ 0x2912C89A);
				continue;
			case 12u:
			{
				StringBuilder stringBuilder7 = stringBuilder;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(46, 1, stringBuilder);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"header right w100\" colspan=3 >");
				appendInterpolatedStringHandler.AppendFormatted(cvalue.ToString("n", nfi));
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder7, ref appendInterpolatedStringHandler);
				num = 1118588471;
				continue;
			}
			case 6u:
			{
				stringBuilder = currentPage;
				StringBuilder stringBuilder8 = stringBuilder;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(39, 3, stringBuilder);
				appendInterpolatedStringHandler.AppendLiteral("<a class=\"filter\" href=\"");
				appendInterpolatedStringHandler.AppendFormatted(BaseWSURL);
				appendInterpolatedStringHandler.AppendLiteral("/filter/");
				appendInterpolatedStringHandler.AppendFormatted(2);
				appendInterpolatedStringHandler.AppendLiteral("/\">");
				appendInterpolatedStringHandler.AppendFormatted(GetSymbol((uih == UIFilter.All) ? OutputSymbol.FilterAllSelected : OutputSymbol.FilterAll));
				appendInterpolatedStringHandler.AppendLiteral("</a>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder8, ref appendInterpolatedStringHandler);
				num = 2017643866;
				continue;
			}
			case 15u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<td class=\"header center\">&nbsp;</td>");
				num = 1893603018;
				continue;
			case 3u:
			{
				stringBuilder = currentPage;
				StringBuilder stringBuilder6 = stringBuilder;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(46, 1, stringBuilder);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"header right w100\" colspan=1 >");
				appendInterpolatedStringHandler.AppendFormatted(mvalue.ToString("n", nfi));
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder6, ref appendInterpolatedStringHandler);
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "</tr>");
				num = 1137285669;
				continue;
			}
			case 13u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<td class=\"header center\">");
				stringBuilder = currentPage;
				num = 141465851;
				continue;
			case 1u:
				goto IL_0248;
			case 0u:
				num = (int)((num2 * 1560044092) ^ 0x55731675);
				continue;
			case 2u:
				goto IL_02ad;
			case 14u:
				stringBuilder = currentPage;
				num = 1489713806;
				continue;
			case 5u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "<td class=\"header\">&nbsp;</td>");
				num = 741366637;
				continue;
			case 17u:
			{
				StringBuilder stringBuilder4 = stringBuilder;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(39, 3, stringBuilder);
				appendInterpolatedStringHandler.AppendLiteral("<a class=\"filter\" href=\"");
				appendInterpolatedStringHandler.AppendFormatted(BaseWSURL);
				appendInterpolatedStringHandler.AppendLiteral("/filter/");
				appendInterpolatedStringHandler.AppendFormatted(1);
				appendInterpolatedStringHandler.AppendLiteral("/\">");
				appendInterpolatedStringHandler.AppendFormatted(GetSymbol((uih == UIFilter.MappingOnly) ? OutputSymbol.FilterMappedSelected : OutputSymbol.FilterMapped));
				appendInterpolatedStringHandler.AppendLiteral("</a>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder4, ref appendInterpolatedStringHandler);
				stringBuilder = currentPage;
				StringBuilder stringBuilder5 = stringBuilder;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(39, 3, stringBuilder);
				appendInterpolatedStringHandler.AppendLiteral("<a class=\"filter\" href=\"");
				appendInterpolatedStringHandler.AppendFormatted(BaseWSURL);
				appendInterpolatedStringHandler.AppendLiteral("/filter/");
				appendInterpolatedStringHandler.AppendFormatted(0);
				appendInterpolatedStringHandler.AppendLiteral("/\">");
				appendInterpolatedStringHandler.AppendFormatted(GetSymbol((uih == UIFilter.Default) ? OutputSymbol.FilterDefaultSelected : OutputSymbol.FilterDefault));
				appendInterpolatedStringHandler.AppendLiteral("</a>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder5, ref appendInterpolatedStringHandler);
				num = 1633835600;
				continue;
			}
			case 11u:
			{
				stringBuilder = currentPage;
				StringBuilder stringBuilder3 = stringBuilder;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(35, 3, stringBuilder);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"header center ");
				appendInterpolatedStringHandler.AppendFormatted((currentBodies == expectedBodies) ? "BodiesAllScanned" : "BodiesNotAllScanned");
				appendInterpolatedStringHandler.AppendLiteral("\">");
				appendInterpolatedStringHandler.AppendFormatted(currentBodies);
				appendInterpolatedStringHandler.AppendLiteral(" / ");
				appendInterpolatedStringHandler.AppendFormatted(expectedBodies);
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder3, ref appendInterpolatedStringHandler);
				num = 700859022;
				continue;
			}
			case 16u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "</td>");
				num = 1960196524;
				continue;
			case 9u:
				goto IL_04ef;
			case 4u:
			{
				stringBuilder = currentPage;
				StringBuilder stringBuilder2 = stringBuilder;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(77, 1, stringBuilder);
				appendInterpolatedStringHandler.AppendLiteral("<tr><td class=\"header headerfix roundleft\" colspan=4 id=\"currentsystem\">");
				appendInterpolatedStringHandler.AppendFormatted(system);
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder2, ref appendInterpolatedStringHandler);
				num = 1882645526;
				continue;
			}
			case 7u:
				return;
			}
			break;
			IL_04ef:
			int num3;
			if (!showFilter)
			{
				num = 1134724213;
				num3 = num;
			}
			else
			{
				num = 262393413;
				num3 = num;
			}
			continue;
			IL_02ad:
			int num4;
			if (expectedBodies <= 0)
			{
				num = 512496081;
				num4 = num;
			}
			else
			{
				num = 1605378419;
				num4 = num;
			}
		}
		goto IL_0007;
		IL_0248:
		stringBuilder = currentPage;
		StringBuilder stringBuilder9 = stringBuilder;
		appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(58, 1, stringBuilder);
		appendInterpolatedStringHandler.AppendLiteral("<tr><td class=\"header headerfix roundleft\" colspan=4>");
		appendInterpolatedStringHandler.AppendFormatted(system);
		appendInterpolatedStringHandler.AppendLiteral("</td>");
		_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder9, ref appendInterpolatedStringHandler);
		num = 1882645526;
		goto IL_000c;
	}
public void AddHTMLBodyDetails(OutputLine q, string row, bool showcheckbox)
	{
		OutputSymbol sym = default;
		StringBuilder stringBuilder2 = default;

		if (q.IsNextToVisit)
		{
			goto IL_000b;
		}
		goto IL_058e;
		IL_000b:
		int num = -572233491;
		goto IL_0010;
		IL_0010:
		while (true)
		{
			uint num2;
			StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler;
			switch ((num2 = (uint)(num ^ -1688737442)) % 44)
			{
			case 16u:
				break;
			default:
				return;
			case 36u:
				goto IL_00d6;
			case 35u:
				goto IL_017e;
			case 31u:
				goto IL_019a;
			case 6u:
				stringBuilder2 = currentPage;
				num = (int)((num2 * 1587227791) ^ 0x1C7DA5F8);
				continue;
			case 23u:
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(currentPage, "</tr>");
				num = -2077526265;
				continue;
			case 1u:
				stringBuilder2 = currentPage;
				num = -1119017572;
				continue;
			case 33u:
			{
				stringBuilder2 = currentPage;
				StringBuilder stringBuilder12 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(34, 1, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral("\" colspan=2>&nbsp;</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder12, ref appendInterpolatedStringHandler);
				num = -889517570;
				continue;
			}
			case 32u:
				sym = OutputSymbol.Alert;
				num = (int)(num2 * 58139798) ^ -145483036;
				continue;
			case 8u:
			{
				stringBuilder2 = currentPage;
				StringBuilder stringBuilder7 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(43, 6, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral("\"><a href=\"");
				appendInterpolatedStringHandler.AppendFormatted(BaseWSURL);
				appendInterpolatedStringHandler.AppendLiteral("/toggle/");
				appendInterpolatedStringHandler.AppendFormatted(q.ID64);
				appendInterpolatedStringHandler.AppendLiteral("/");
				appendInterpolatedStringHandler.AppendFormatted(q.BodyID);
				appendInterpolatedStringHandler.AppendLiteral("/");
				appendInterpolatedStringHandler.AppendFormatted(q.Map ? "0" : "1");
				appendInterpolatedStringHandler.AppendLiteral("\">");
				appendInterpolatedStringHandler.AppendFormatted(GetSymbol((!q.Map) ? OutputSymbol.NotWorthMapping : OutputSymbol.WorthMapping));
				appendInterpolatedStringHandler.AppendLiteral("</a></td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder7, ref appendInterpolatedStringHandler);
				stringBuilder2 = currentPage;
				StringBuilder stringBuilder8 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(27, 2, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral(" left w30\">");
				string value2;
				if (q.Order != 0)
				{
					DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(54, 1);
					defaultInterpolatedStringHandler.AppendLiteral("<span title=\"Visit Order\" class=\"visitorder\">[");
					defaultInterpolatedStringHandler.AppendFormatted(q.Order);
					defaultInterpolatedStringHandler.AppendLiteral("]</span>");
					value2 = defaultInterpolatedStringHandler.ToStringAndClear();
				}
				else
				{
					value2 = "&nbsp;";
				}
				appendInterpolatedStringHandler.AppendFormatted(value2);
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder8, ref appendInterpolatedStringHandler);
				num = -889517570;
				continue;
			}
			case 15u:
			{
				int num7;
				int num8;
				if (q.Gravity >= 2.0)
				{
					num7 = 775805757;
					num8 = num7;
				}
				else
				{
					num7 = 320432821;
					num8 = num7;
				}
				num = num7 ^ ((int)num2 * -757664717);
				continue;
			}
			case 37u:
				num = (int)(num2 * 759096088) ^ -251402900;
				continue;
			case 10u:
				row = _200D_202B_206E_200B_200B_202A_202D_206A_206F_200E_202B_202B_202A_202E_206C_202B_206B_202A_202D_202D_202B_202A_200D_202D_202B_206A_202C_202B_202C_206D_202B_202B_206E_200E_202E_206A_200F_206D_206D_206A_202E(row, " highlight");
				num = -129624770;
				continue;
			case 34u:
				sym = OutputSymbol.Warn1;
				num = ((int)num2 * -1367334780) ^ 0x267D03F;
				continue;
			case 3u:
			{
				StringBuilder stringBuilder6 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(28, 2, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral(" right w50\">");
				appendInterpolatedStringHandler.AppendFormatted(GetSymbol(sym, _200D_202B_206E_200B_200B_202A_202D_206A_206F_200E_202B_202B_202A_202E_206C_202B_206B_202A_202D_202D_202B_202A_200D_202D_202B_206A_202C_202B_202C_206D_202B_202B_206E_200E_202E_206A_200F_206D_206D_206A_202E(q.Gravity.ToString("0.##"), " G")));
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder6, ref appendInterpolatedStringHandler);
				num = -1280625477;
				continue;
			}
			case 19u:
			{
				int num9;
				int num10;
				if (!theme.Settings.UseHighlightRow)
				{
					num9 = 1836322454;
					num10 = num9;
				}
				else
				{
					num9 = 2119815748;
					num10 = num9;
				}
				num = num9 ^ ((int)num2 * -584052296);
				continue;
			}
			case 4u:
				goto IL_058e;
			case 30u:
				goto IL_059f;
			case 12u:
				goto IL_061d;
			case 40u:
			{
				stringBuilder2 = currentPage;
				StringBuilder stringBuilder4 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(44, 1, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral(" left w50\" colspan=2\">&nbsp;</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder4, ref appendInterpolatedStringHandler);
				num = -889517570;
				continue;
			}
			case 27u:
				stringBuilder2 = currentPage;
				num = ((int)num2 * -857314176) ^ -1728355436;
				continue;
			case 17u:
			{
				stringBuilder2 = currentPage;
				StringBuilder stringBuilder15 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(63, 3, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral("\" colspan=2><span class=\"STAR\" title=\"");
				appendInterpolatedStringHandler.AppendFormatted(StarTypeLookup(q.Type));
				appendInterpolatedStringHandler.AppendLiteral("\">");
				appendInterpolatedStringHandler.AppendFormatted(StarTypeLookup(q.Type));
				appendInterpolatedStringHandler.AppendLiteral("</span></td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder15, ref appendInterpolatedStringHandler);
				num = -1842914660;
				continue;
			}
			case 28u:
			{
				StringBuilder stringBuilder14 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(38, 2, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral(" left w50\" colspan=2\">");
				appendInterpolatedStringHandler.AppendFormatted(GetSymbol(OutputSymbol.Mapped));
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder14, ref appendInterpolatedStringHandler);
				num = -889517570;
				continue;
			}
			case 9u:
				sym = OutputSymbol.OK;
				num = (int)((num2 * 1786786815) ^ 0x3226CB77);
				continue;
			case 38u:
			{
				StringBuilder stringBuilder13 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(33, 1, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<tr><td style='width:32px;'>");
				appendInterpolatedStringHandler.AppendFormatted(q.IsNextToVisit ? GetSymbol(OutputSymbol.NextBody) : "&nbsp;&nbsp;&nbsp;&nbsp;");
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder13, ref appendInterpolatedStringHandler);
				num = -2042682595;
				continue;
			}
			case 18u:
			{
				StringBuilder stringBuilder11 = currentPage;
				string[] array = new string[8];
				DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(18, 8);
				defaultInterpolatedStringHandler.AppendLiteral("<td class=\"");
				defaultInterpolatedStringHandler.AppendFormatted(row);
				defaultInterpolatedStringHandler.AppendLiteral(" w100\">");
				defaultInterpolatedStringHandler.AppendFormatted(q.IsLandable ? GetSymbol(OutputSymbol.IsLandable) : "");
				defaultInterpolatedStringHandler.AppendFormatted(q.Extra.IsHighValue ? GetSymbol(OutputSymbol.IsHighValue) : "");
				defaultInterpolatedStringHandler.AppendFormatted(q.Terraformable ? GetSymbol(OutputSymbol.Terraformable) : "");
				string value3;
				if (q.WasMapped)
				{
					value3 = GetSymbol(OutputSymbol.WasMapped);
				}
				else
				{
					value3 = (q.WasDiscovered ? GetSymbol(OutputSymbol.WasDiscovered) : "");
				}
				defaultInterpolatedStringHandler.AppendFormatted(value3);
				defaultInterpolatedStringHandler.AppendFormatted(q.WasFootfalled ? GetSymbol(OutputSymbol.WasFootfalled) : "");
				defaultInterpolatedStringHandler.AppendFormatted((q.Extra.BioCount > 0) ? GetSymbol(OutputSymbol.HasBiologicals) : "");
				defaultInterpolatedStringHandler.AppendFormatted(_200C_202E_200B_206F_206E_200C_200F_202A_200D_206E_206C_202A_200E_200F_200F_200F_200B_202E_200D_200D_202A_206F_206A_206F_206C_206F_200C_206C_202B_200D_200F_202B_200D_200B_206D_206E_202C_200B_202C_206C_202E(q.Volcanism, "") ? GetSymbol(OutputSymbol.HasVolcanism) : "");
				array[0] = defaultInterpolatedStringHandler.ToStringAndClear();
				array[1] = "&nbsp;";
				array[2] = (q.Extra.BioInsightsPersonalCodex ? GetSymbol(OutputSymbol.BioInsightsPersonalCodex) : "");
				array[3] = (q.Extra.BioInsightsMarked ? GetSymbol(OutputSymbol.BioInsightsMarked) : "");
				array[4] = (q.Extra.BioInsightsGalacticCodex ? GetSymbol(OutputSymbol.BioInsightsGalacticCodex) : "");
				array[5] = (q.Extra.BioInsightsHighValue ? GetSymbol(OutputSymbol.BioInsightsHighValue) : "");
				array[6] = (q.Extra.PluginWorthVisiting ? GetSymbol(OutputSymbol.PluginWorthVisiting) : "");
				array[7] = "</td>";
				_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder11, _202E_206A_206C_200D_202B_202C_200B_200C_200C_206F_200F_206A_202E_206C_200D_206B_202D_202D_200F_202C_200D_202B_206F_200B_202C_206D_202A_200C_202E_200E_206E_206D_202A_206E_202E_206C_206A_200B_202E_202C_202E(array));
				num = -452418818;
				continue;
			}
			case 42u:
			{
				StringBuilder stringBuilder10 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(22, 2, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral(" w50\">");
				appendInterpolatedStringHandler.AppendFormatted(q.Type);
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder10, ref appendInterpolatedStringHandler);
				num = -1842914660;
				continue;
			}
			case 0u:
				stringBuilder2 = currentPage;
				num = -267550178;
				continue;
			case 21u:
				stringBuilder2 = currentPage;
				num = (int)(num2 * 377198112) ^ -1083517330;
				continue;
			case 7u:
				sym = OutputSymbol.Warn0;
				num = ((int)num2 * -963156362) ^ -802397655;
				continue;
			case 5u:
				num = ((int)num2 * -1524130307) ^ 0x58578633;
				continue;
			case 2u:
				goto IL_0b00;
			case 13u:
			{
				stringBuilder2 = currentPage;
				StringBuilder stringBuilder9 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(28, 1, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral(" w50\">&nbsp;</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder9, ref appendInterpolatedStringHandler);
				num = -1534576030;
				continue;
			}
			case 26u:
				goto IL_0b6f;
			case 14u:
				stringBuilder2 = currentPage;
				num = -1067191919;
				continue;
			case 11u:
				goto IL_0ba5;
			case 29u:
				num = ((int)num2 * -167514249) ^ -1487511253;
				continue;
			case 39u:
			{
				int num5;
				int num6;
				if (q.Gravity >= 1.0)
				{
					num5 = 881955469;
					num6 = num5;
				}
				else
				{
					num5 = 2006276877;
					num6 = num5;
				}
				num = num5 ^ (int)(num2 * 1541852608);
				continue;
			}
			case 25u:
			{
				string value = PlanetLookup[q.Type];
				stringBuilder2 = currentPage;
				StringBuilder stringBuilder5 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(41, 2, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral("\"><span class=\"");
				appendInterpolatedStringHandler.AppendFormatted(value);
				appendInterpolatedStringHandler.AppendLiteral("\" ></span></td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder5, ref appendInterpolatedStringHandler);
				num = -1842914660;
				continue;
			}
			case 22u:
			{
				int num3;
				int num4;
				if (q.Gravity < 0.5)
				{
					num3 = -946974445;
					num4 = num3;
				}
				else
				{
					num3 = -2043661117;
					num4 = num3;
				}
				num = num3 ^ (int)(num2 * 1250013663);
				continue;
			}
			case 43u:
				goto IL_0cb8;
			case 24u:
			{
				stringBuilder2 = currentPage;
				StringBuilder stringBuilder3 = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(24, 2, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral(" right\">");
				appendInterpolatedStringHandler.AppendFormatted(q.CurrentValue.ToString("n", nfi));
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder3, ref appendInterpolatedStringHandler);
				num = -1290750517;
				continue;
			}
			case 20u:
			{
				StringBuilder stringBuilder = stringBuilder2;
				appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(32, 2, stringBuilder2);
				appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
				appendInterpolatedStringHandler.AppendFormatted(row);
				appendInterpolatedStringHandler.AppendLiteral(" right padleft\">");
				appendInterpolatedStringHandler.AppendFormatted(q.PotentialMaxValue.ToString("n", nfi));
				appendInterpolatedStringHandler.AppendLiteral("</td>");
				_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder, ref appendInterpolatedStringHandler);
				num = -1661095163;
				continue;
			}
			case 41u:
				return;
			}
			break;
			IL_0cb8:
			int num11;
			if (showcheckbox)
			{
				num = -831549970;
				num11 = num;
			}
			else
			{
				num = -1299404542;
				num11 = num;
			}
			continue;
			IL_0b00:
			int num12;
			if (q.IsStar)
			{
				num = -452418818;
				num12 = num;
			}
			else
			{
				num = -431573308;
				num12 = num;
			}
			continue;
			IL_059f:
			StringBuilder stringBuilder16 = stringBuilder2;
			appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(18, 2, stringBuilder2);
			appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
			appendInterpolatedStringHandler.AppendFormatted(row);
			appendInterpolatedStringHandler.AppendLiteral("\">");
			appendInterpolatedStringHandler.AppendFormatted(q.Body);
			appendInterpolatedStringHandler.AppendLiteral("</td>");
			_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder16, ref appendInterpolatedStringHandler);
			int num13;
			if (!q.IsStar)
			{
				num = -580465731;
				num13 = num;
			}
			else
			{
				num = -451972305;
				num13 = num;
			}
			continue;
			IL_0ba5:
			int num14;
			if (q.Gravity >= 1.0)
			{
				num = -1869844767;
				num14 = num;
			}
			else
			{
				num = -269104080;
				num14 = num;
			}
			continue;
			IL_017e:
			int num15;
			if (q.Done)
			{
				num = -886096792;
				num15 = num;
			}
			else
			{
				num = -705326411;
				num15 = num;
			}
			continue;
			IL_061d:
			int num16;
			if (!q.IsStar)
			{
				num = -73103679;
				num16 = num;
			}
			else
			{
				num = -721651729;
				num16 = num;
			}
			continue;
			IL_0b6f:
			int num17;
			if (q.Gravity >= 2.0)
			{
				num = -1330929418;
				num17 = num;
			}
			else
			{
				num = -2112695660;
				num17 = num;
			}
			continue;
			IL_00d6:
			StringBuilder stringBuilder17 = stringBuilder2;
			appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(31, 2, stringBuilder2);
			appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
			appendInterpolatedStringHandler.AppendFormatted(row);
			appendInterpolatedStringHandler.AppendLiteral(" right w50\">");
			appendInterpolatedStringHandler.AppendFormatted(_206D_206E_200E_200F_200B_206B_206F_202D_206D_200E_202D_200E_202C_202E_206B_206C_200B_206F_202A_202C_200D_202B_206C_202E_202C_200E_200E_202B_200F_202D_200E_202E_200B_202C_206A_200D_202D_206B_202D_206F_202E(q.Distance, 0).ToString("n", nfi));
			appendInterpolatedStringHandler.AppendLiteral(" LS</td>");
			_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder17, ref appendInterpolatedStringHandler);
			int num18;
			if (q.Gravity > 0.0)
			{
				num = -1094591381;
				num18 = num;
			}
			else
			{
				num = -2035147513;
				num18 = num;
			}
			continue;
			IL_019a:
			int num19;
			if (PlanetLookup.ContainsKey(q.Type))
			{
				num = -1887943625;
				num19 = num;
			}
			else
			{
				num = -20999293;
				num19 = num;
			}
		}
		goto IL_000b;
		IL_058e:
		stringBuilder2 = currentPage;
		num = -856619152;
		goto IL_0010;
	}
public static string StarTypeLookup(string theScanType)
	{
		string result = theScanType;
		char c = default;
		int num9 = default;
		while (true)
		{
			int num = 620699999;
			while (true)
			{
				uint num2;
				int num5;
				int num8;
				int num13;
				int num18;
				int num23;
				int num31;
				int num46;
				int num54;
				int num58;
				int num61;
				int num69;
				switch ((num2 = (uint)(num ^ 0x15C2D9C1)) % 118)
				{
				case 72u:
					break;
				case 46u:
				{
					int num65;
					int num66;
					if (c == 'T')
					{
						num65 = 898967830;
						num66 = num65;
					}
					else
					{
						num65 = 1301732617;
						num66 = num65;
					}
					num = num65 ^ ((int)num2 * -400177792);
					continue;
				}
				case 14u:
					num = (int)((num2 * 796185253) ^ 0x60514A9D);
					continue;
				case 55u:
					c = _202D_200B_206D_200C_206F_206E_206B_206E_200B_200B_202C_206B_206A_202B_200F_200F_206E_206F_206B_206F_206C_206A_200F_206F_206D_202C_202E_206C_200B_200E_202D_206F_206E_200D_206E_202E_202D_202A_202C_200F_202E(theScanType, 0);
					switch (c)
					{
					case 'A':
					case 'B':
					case 'F':
					case 'G':
					case 'K':
					case 'L':
					case 'M':
					case 'O':
					case 'T':
					case 'Y':
						goto IL_0580;
					case 'E':
					case 'I':
					case 'J':
					case 'P':
					case 'Q':
					case 'R':
					case 'U':
					case 'V':
					case 'X':
						goto IL_06f9;
					case 'N':
						goto IL_0821;
					case 'H':
						goto IL_0973;
					case 'W':
						goto IL_09bd;
					case 'D':
						goto IL_0b49;
					case 'C':
					case 'S':
						goto IL_0c61;
					}
					num = 7206180;
					continue;
				case 115u:
					num = ((int)num2 * -1184760259) ^ -700412996;
					continue;
				case 63u:
				{
					int num28;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DB"))
					{
						num = 691587631;
						num28 = num;
					}
					else
					{
						num = 1083660526;
						num28 = num;
					}
					continue;
				}
				case 67u:
					num = (int)((num2 * 1327650181) ^ 0x49FFE452);
					continue;
				case 98u:
				{
					int num43;
					int num44;
					if (c == 'B')
					{
						num43 = -792550447;
						num44 = num43;
					}
					else
					{
						num43 = -2082030909;
						num44 = num43;
					}
					num = num43 ^ ((int)num2 * -822199439);
					continue;
				}
				case 87u:
					goto IL_0313;
				case 80u:
				{
					int num14;
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "CN"))
					{
						num = 1001327465;
						num14 = num;
					}
					else
					{
						num = 319057665;
						num14 = num;
					}
					continue;
				}
				case 103u:
				{
					int num4;
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "WNC"))
					{
						num = 1442822707;
						num4 = num;
					}
					else
					{
						num = 732664288;
						num4 = num;
					}
					continue;
				}
				case 107u:
					num = (int)(num2 * 986202363) ^ -1350729290;
					continue;
				case 113u:
				{
					int num59;
					int num60;
					if (c != 'A')
					{
						num59 = 1072660268;
						num60 = num59;
					}
					else
					{
						num59 = 249655081;
						num60 = num59;
					}
					num = num59 ^ ((int)num2 * -1793552925);
					continue;
				}
				case 109u:
					num = ((int)num2 * -1894744158) ^ 0x4EFCC76F;
					continue;
				case 28u:
					num = ((int)num2 * -1914080549) ^ -1752815675;
					continue;
				case 69u:
				{
					int num49;
					int num50;
					if (c != 'O')
					{
						num49 = 902525212;
						num50 = num49;
					}
					else
					{
						num49 = 1047397660;
						num50 = num49;
					}
					num = num49 ^ ((int)num2 * -792924983);
					continue;
				}
				case 32u:
				{
					int num42;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "M_RedGiant"))
					{
						num = 566506418;
						num42 = num;
					}
					else
					{
						num = 7958071;
						num42 = num;
					}
					continue;
				}
				case 64u:
					num = ((int)num2 * -518744022) ^ 0x5798D4A1;
					continue;
				case 48u:
					num9 = _200E_200B_206A_202B_206B_200F_206D_206B_206B_206A_200B_206E_206A_206F_202A_202B_200E_200E_206E_206D_206A_202B_202C_206C_206A_202B_200E_206E_206B_206B_206E_200E_202B_202A_200C_200C_200B_200E_200E_202E_202E(theScanType);
					num = (int)((num2 * 2013965246) ^ 0x3BAC234);
					continue;
				case 68u:
				{
					int num29;
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DOV"))
					{
						num = 15997487;
						num29 = num;
					}
					else
					{
						num = 691587631;
						num29 = num;
					}
					continue;
				}
				case 99u:
					switch (c)
					{
					case 'N':
						goto IL_0685;
					case 'P':
					case 'R':
						goto IL_06f9;
					case 'Q':
						goto IL_076c;
					case 'O':
						goto IL_08ed;
					case 'S':
						goto IL_0a6d;
					}
					num = ((int)num2 * -563883771) ^ -622677599;
					continue;
				case 2u:
					num = ((int)num2 * -223851546) ^ 0x268C8591;
					continue;
				case 85u:
				{
					int num62;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "M_RedSuperGiant"))
					{
						num = 2023865449;
						num62 = num;
					}
					else
					{
						num = 392050047;
						num62 = num;
					}
					continue;
				}
				case 82u:
					num = ((int)num2 * -814030446) ^ 0x2D52C2C1;
					continue;
				case 8u:
				{
					int num56;
					int num57;
					if (c != 'H')
					{
						num56 = 1555844123;
						num57 = num56;
					}
					else
					{
						num56 = 1422751190;
						num57 = num56;
					}
					num = num56 ^ (int)(num2 * 66250712);
					continue;
				}
				case 43u:
				{
					int num53;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DCV"))
					{
						num = 691587631;
						num53 = num;
					}
					else
					{
						num = 151678151;
						num53 = num;
					}
					continue;
				}
				case 93u:
					goto IL_0546;
				case 23u:
				{
					int num51;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "CJ"))
					{
						num = 319057665;
						num51 = num;
					}
					else
					{
						num = 1829610883;
						num51 = num;
					}
					continue;
				}
				case 77u:
					goto IL_0580;
				case 73u:
					num = ((int)num2 * -453499100) ^ -709882623;
					continue;
				case 90u:
					num = ((int)num2 * -1505994712) ^ -1327216679;
					continue;
				case 3u:
					num = ((int)num2 * -708009155) ^ 0x528373FA;
					continue;
				case 78u:
					num = ((int)num2 * -501743646) ^ 0x486CD001;
					continue;
				case 5u:
				{
					int num38;
					int num39;
					if (c == 'F')
					{
						num38 = 1188037926;
						num39 = num38;
					}
					else
					{
						num38 = 878399307;
						num39 = num38;
					}
					num = num38 ^ ((int)num2 * -986585747);
					continue;
				}
				case 91u:
				{
					int num35;
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "A_BlueWhiteSuperGiant"))
					{
						num = 1435696716;
						num35 = num;
					}
					else
					{
						num = 566506418;
						num35 = num;
					}
					continue;
				}
				case 100u:
					num = (int)(num2 * 647052861) ^ -221607277;
					continue;
				case 27u:
					num = (int)((num2 * 1101846549) ^ 0x1EC1DEF4);
					continue;
				case 19u:
					switch (c)
					{
					case 'B':
						break;
					case 'A':
						goto IL_0313;
					default:
						goto IL_0673;
					case 'C':
						goto IL_0e79;
					}
					goto case 63u;
				case 44u:
					goto IL_0685;
				case 89u:
				{
					int num26;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DO"))
					{
						num = 691587631;
						num26 = num;
					}
					else
					{
						num = 838448170;
						num26 = num;
					}
					continue;
				}
				case 65u:
					num = (int)((num2 * 1937133555) ^ 0x7B35FD1C);
					continue;
				case 54u:
					num = ((int)num2 * -707805437) ^ 0x5E5E500F;
					continue;
				case 96u:
					goto IL_06f9;
				case 30u:
					result = "Supermassive Black Hole";
					num = 1403428698;
					continue;
				case 74u:
				{
					int num21;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "MS"))
					{
						num = 319057665;
						num21 = num;
					}
					else
					{
						num = 1396741180;
						num21 = num;
					}
					continue;
				}
				case 25u:
				{
					int num19;
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "CHd"))
					{
						num = 1999631100;
						num19 = num;
					}
					else
					{
						num = 319057665;
						num19 = num;
					}
					continue;
				}
				case 108u:
					goto IL_076c;
				case 71u:
				{
					int num15;
					if (c == 'J')
					{
						num = 274393090;
						num15 = num;
					}
					else
					{
						num = 1980876544;
						num15 = num;
					}
					continue;
				}
				case 37u:
					goto IL_07ad;
				case 7u:
					goto IL_07d5;
				case 86u:
					num = (int)((num2 * 193952685) ^ 0x582C5A1B);
					continue;
				case 45u:
				{
					int num70;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DX"))
					{
						num = 691587631;
						num70 = num;
					}
					else
					{
						num = 1615266376;
						num70 = num;
					}
					continue;
				}
				case 16u:
					goto IL_0821;
				case 76u:
				{
					int num67;
					int num68;
					if (c == 'S')
					{
						num67 = 1170739422;
						num68 = num67;
					}
					else
					{
						num67 = 1636892082;
						num68 = num67;
					}
					num = num67 ^ (int)(num2 * 1851091287);
					continue;
				}
				case 4u:
				{
					int num63;
					int num64;
					if (num9 != 10)
					{
						num63 = -1544259010;
						num64 = num63;
					}
					else
					{
						num63 = -1725067859;
						num64 = num63;
					}
					num = num63 ^ ((int)num2 * -1684264789);
					continue;
				}
				case 41u:
					num = ((int)num2 * -1597401269) ^ 0x601EE332;
					continue;
				case 35u:
					switch (c)
					{
					case 'C':
						break;
					case 'B':
						goto IL_07ad;
					default:
						goto IL_08a1;
					case 'A':
						goto IL_0b9a;
					}
					goto case 43u;
				case 104u:
					goto IL_08b3;
				case 17u:
					num = (int)((num2 * 78312320) ^ 0x5C9FDFA5);
					continue;
				case 53u:
					goto IL_08ed;
				case 26u:
				{
					int num55;
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DAO"))
					{
						num = 2047870266;
						num55 = num;
					}
					else
					{
						num = 691587631;
						num55 = num;
					}
					continue;
				}
				case 29u:
					num = (int)(num2 * 1521433089) ^ -1734191820;
					continue;
				case 101u:
					num = (int)((num2 * 1830811976) ^ 0x51287B2D);
					continue;
				case 0u:
					num = ((int)num2 * -1390923113) ^ 0x59C92BAF;
					continue;
				case 50u:
					goto IL_0973;
				case 9u:
					goto IL_098a;
				case 34u:
					num = (int)(num2 * 1530040695) ^ -1665961469;
					continue;
				case 95u:
					goto IL_09bd;
				case 66u:
				{
					int num52;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "F_WhiteSuperGiant"))
					{
						num = 566506418;
						num52 = num;
					}
					else
					{
						num = 2089777800;
						num52 = num;
					}
					continue;
				}
				case 117u:
				{
					int num47;
					int num48;
					if (c == 'G')
					{
						num47 = 2006190046;
						num48 = num47;
					}
					else
					{
						num47 = 1517151710;
						num48 = num47;
					}
					num = num47 ^ ((int)num2 * -384220037);
					continue;
				}
				case 106u:
					result = "T Tauri";
					num = 1898288984;
					continue;
				case 105u:
					switch (num9)
					{
					case 15:
						break;
					case 14:
					case 16:
						goto IL_06f9;
					case 17:
						goto IL_07d5;
					default:
						goto IL_0a63;
					case 13:
						goto IL_0edd;
					}
					goto case 85u;
				case 56u:
					goto IL_0a6d;
				case 20u:
				{
					int num45;
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DBV"))
					{
						num = 44438717;
						num45 = num;
					}
					else
					{
						num = 691587631;
						num45 = num;
					}
					continue;
				}
				case 110u:
				{
					int num40;
					int num41;
					if (num9 != 21)
					{
						num40 = -1199992863;
						num41 = num40;
					}
					else
					{
						num40 = -1109246509;
						num41 = num40;
					}
					num = num40 ^ (int)(num2 * 2019385531);
					continue;
				}
				case 22u:
				{
					int num36;
					int num37;
					if (theScanType == null)
					{
						num36 = 538646287;
						num37 = num36;
					}
					else
					{
						num36 = 1023099829;
						num37 = num36;
					}
					num = num36 ^ (int)(num2 * 1093638491);
					continue;
				}
				case 39u:
					num = ((int)num2 * -1501844402) ^ 0x629F369B;
					continue;
				case 11u:
				{
					int num34;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "TTS"))
					{
						num = 1293566695;
						num34 = num;
					}
					else
					{
						num = 239425177;
						num34 = num;
					}
					continue;
				}
				case 62u:
					num = (int)(num2 * 52766540) ^ -603030151;
					continue;
				case 58u:
					goto IL_0b49;
				case 40u:
				{
					int num33;
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "CH"))
					{
						num = 455912418;
						num33 = num;
					}
					else
					{
						num = 319057665;
						num33 = num;
					}
					continue;
				}
				case 38u:
					goto IL_0b9a;
				case 61u:
					num = ((int)num2 * -1463314452) ^ 0x12261F69;
					continue;
				case 10u:
				{
					int num32;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "B_BlueWhiteSuperGiant"))
					{
						num = 566506418;
						num32 = num;
					}
					else
					{
						num = 1492786824;
						num32 = num;
					}
					continue;
				}
				case 75u:
					num = ((int)num2 * -1668731631) ^ 0x13457FE8;
					continue;
				case 18u:
					num = ((int)num2 * -61131790) ^ 0x6B0443F5;
					continue;
				case 21u:
				{
					int num30;
					if (c == 'N')
					{
						num = 481492860;
						num30 = num;
					}
					else
					{
						num = 23301978;
						num30 = num;
					}
					continue;
				}
				case 83u:
				{
					int num27;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DAZ"))
					{
						num = 691587631;
						num27 = num;
					}
					else
					{
						num = 1707774756;
						num27 = num;
					}
					continue;
				}
				case 52u:
					goto IL_0c61;
				case 36u:
				{
					int num24;
					int num25;
					if (c != 'H')
					{
						num24 = 1633933270;
						num25 = num24;
					}
					else
					{
						num24 = 1121244577;
						num25 = num24;
					}
					num = num24 ^ ((int)num2 * -1853028889);
					continue;
				}
				case 79u:
				{
					int num22;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "SupermassiveBlackHole"))
					{
						num = 904571537;
						num22 = num;
					}
					else
					{
						num = 492939392;
						num22 = num;
					}
					continue;
				}
				case 111u:
					num = ((int)num2 * -69250629) ^ 0x26F0022C;
					continue;
				case 70u:
					num = (int)(num2 * 1036407300) ^ -811480339;
					continue;
				case 92u:
					num = (int)((num2 * 686851701) ^ 0x156DEDFB);
					continue;
				case 31u:
					num = ((int)num2 * -525689241) ^ 0x291AD2AE;
					continue;
				case 97u:
				{
					int num20;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DAV"))
					{
						num = 691587631;
						num20 = num;
					}
					else
					{
						num = 1726835664;
						num20 = num;
					}
					continue;
				}
				case 33u:
					num = ((int)num2 * -760513018) ^ 0x5F08F403;
					continue;
				case 81u:
					num = (int)(num2 * 611014341) ^ -630892848;
					continue;
				case 42u:
				{
					int num16;
					int num17;
					if ((uint)c <= 72u)
					{
						num16 = -678966024;
						num17 = num16;
					}
					else
					{
						num16 = -289017944;
						num17 = num16;
					}
					num = num16 ^ (int)(num2 * 1211894117);
					continue;
				}
				case 12u:
					num = (int)((num2 * 932689560) ^ 0xB90F085);
					continue;
				case 6u:
					num = (int)((num2 * 65880208) ^ 0x3FF859A5);
					continue;
				case 102u:
					num = ((int)num2 * -1160876091) ^ -1306270225;
					continue;
				case 13u:
					num = ((int)num2 * -1476797072) ^ 0x1821CEB5;
					continue;
				case 47u:
					result = _206B_202E_202A_202B_206D_200B_206B_206E_202D_206B_202D_200C_202E_202C_206D_206C_206F_200F_202E_200D_206E_200F_200C_206B_206F_206A_200E_206D_200F_200F_206D_200F_202D_206B_202A_206D_206E_200C_200E_202E(_206A_206B_206D_202C_202E_206D_206A_200B_206B_202C_200E_206E_202D_206E_206E_206E_200B_202B_200C_206C_206E_202A_202A_202E_200C_206B_200B_202E_206A_200E_206B_200C_200C_200D_200B_206E_200C_202D_200D_202E(theScanType, 2), " (", _206A_202A_200F_202C_202A_200C_206D_206F_206B_206B_202A_206F_200D_202A_206A_206F_202E_202C_202A_200C_200D_206D_200E_206E_200B_206E_206E_202D_202B_202A_206A_200B_206F_200D_206D_202A_202E_200E_200D_206F_202E(theScanType, 0, 1), ")");
					num = 860796542;
					continue;
				case 60u:
					result = "Herbig Ae/Be";
					num = 131552377;
					continue;
				case 1u:
				{
					int num11;
					int num12;
					if (num9 <= 10)
					{
						num11 = 394181632;
						num12 = num11;
					}
					else
					{
						num11 = 370439910;
						num12 = num11;
					}
					num = num11 ^ ((int)num2 * -1278445210);
					continue;
				}
				case 49u:
					num = (int)((num2 * 1934227523) ^ 0x119370AA);
					continue;
				case 88u:
				{
					int num10;
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "G_WhiteSuperGiant"))
					{
						num = 402809226;
						num10 = num;
					}
					else
					{
						num = 566506418;
						num10 = num;
					}
					continue;
				}
				case 112u:
					goto IL_0e79;
				case 15u:
					num = ((int)num2 * -940388823) ^ 0x55C2A4F4;
					continue;
				case 59u:
					switch (num9)
					{
					case 1:
						break;
					case 3:
						goto IL_0546;
					case 4:
						goto IL_08b3;
					case 2:
						goto IL_098a;
					default:
						goto IL_0ecb;
					}
					goto case 55u;
				case 51u:
					goto IL_0edd;
				case 84u:
					c = _202D_200B_206D_200C_206F_206E_206B_206E_200B_200B_202C_206B_206A_202B_200F_200F_206E_206F_206B_206F_206C_206A_200F_206F_206D_202C_202E_206C_200B_200E_202D_206F_206E_200D_206E_202E_202D_202A_202C_200F_202E(theScanType, 0);
					num = ((int)num2 * -1908784570) ^ -361423954;
					continue;
				case 24u:
					num = ((int)num2 * -1965653836) ^ -919291915;
					continue;
				case 114u:
					result = "K";
					num = 1590245872;
					continue;
				case 57u:
				{
					int num6;
					int num7;
					if (c != 'X')
					{
						num6 = -271298964;
						num7 = num6;
					}
					else
					{
						num6 = -1896435725;
						num7 = num6;
					}
					num = num6 ^ ((int)num2 * -1130489741);
					continue;
				}
				case 94u:
				{
					int num3;
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DC"))
					{
						num = 691587631;
						num3 = num;
					}
					else
					{
						num = 224562856;
						num3 = num;
					}
					continue;
				}
				default:
					{
						return result;
					}
					IL_08ed:
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "WO"))
					{
						num = 1580787366;
						num5 = num;
					}
					else
					{
						num = 732664288;
						num5 = num;
					}
					continue;
					IL_07d5:
					c = _202D_200B_206D_200C_206F_206E_206B_206E_200B_200B_202C_206B_206A_202B_200F_200F_206E_206F_206B_206F_206C_206A_200F_206F_206D_202C_202E_206C_200B_200E_202D_206F_206E_200D_206E_202E_202D_202A_202C_200F_202E(theScanType, 0);
					num = 963922166;
					continue;
					IL_07ad:
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DBZ"))
					{
						num = 691587631;
						num8 = num;
					}
					else
					{
						num = 247793499;
						num8 = num;
					}
					continue;
					IL_0ecb:
					num = ((int)num2 * -125066902) ^ -613117637;
					continue;
					IL_076c:
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DQ"))
					{
						num = 691587631;
						num13 = num;
					}
					else
					{
						num = 606701671;
						num13 = num;
					}
					continue;
					IL_0c61:
					result = _202B_202B_200B_206F_206B_200C_206B_202B_206E_202E_202B_200D_206B_202C_206D_202A_202D_206C_206D_206A_206F_200C_200D_200C_206C_206C_202E_206D_200C_200F_206B_206B_206A_200E_200F_200E_200F_202B_206E_200C_202E("Carbon Star (", theScanType, ")");
					num = 131552377;
					continue;
					IL_0685:
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "WN"))
					{
						num = 517620727;
						num18 = num;
					}
					else
					{
						num = 732664288;
						num18 = num;
					}
					continue;
					IL_0b49:
					result = _202B_202B_200B_206F_206B_200C_206B_202B_206E_202E_202B_200D_206B_202C_206D_202A_202D_206C_206D_206A_206F_200C_200D_200C_206C_206C_202E_206D_200C_200F_206B_206B_206A_200E_200F_200E_200F_202B_206E_200C_202E("White Dwarf (", theScanType, ")");
					num = 1269251329;
					continue;
					IL_0313:
					if (!_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DA"))
					{
						num = 2102599320;
						num23 = num;
					}
					else
					{
						num = 691587631;
						num23 = num;
					}
					continue;
					IL_0e79:
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "WC"))
					{
						num = 732664288;
						num31 = num;
					}
					else
					{
						num = 1713796277;
						num31 = num;
					}
					continue;
					IL_09bd:
					result = _202B_202B_200B_206F_206B_200C_206B_202B_206E_202E_202B_200D_206B_202C_206D_202A_202D_206C_206D_206A_206F_200C_200D_200C_206C_206C_202E_206D_200C_200F_206B_206B_206A_200E_200F_200E_200F_202B_206E_200C_202E("Wolf-Rayet (", theScanType, ")");
					num = 2007905061;
					continue;
					IL_0973:
					result = "Black Hole";
					num = 131552377;
					continue;
					IL_0673:
					num = (int)(num2 * 506972498) ^ -1136138445;
					continue;
					IL_06f9:
					result = theScanType;
					num = 131552377;
					continue;
					IL_0edd:
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "K_OrangeGiant"))
					{
						num = 566506418;
						num46 = num;
					}
					else
					{
						num = 1466235899;
						num46 = num;
					}
					continue;
					IL_0a63:
					num = 1663718349;
					continue;
					IL_0821:
					result = "Neutron";
					num = 1250886589;
					continue;
					IL_0546:
					c = _202D_200B_206D_200C_206F_206E_206B_206E_200B_200B_202C_206B_206A_202B_200F_200F_206E_206F_206B_206F_206C_206A_200F_206F_206D_202C_202E_206C_200B_200E_202D_206F_206E_200D_206E_202E_202D_202A_202C_200F_202E(theScanType, 1);
					num = 576736741;
					continue;
					IL_098a:
					c = _202D_200B_206D_200C_206F_206E_206B_206E_200B_200B_202C_206B_206A_202B_200F_200F_206E_206F_206B_206F_206C_206A_200F_206F_206D_202C_202E_206C_200B_200E_202D_206F_206E_200D_206E_202E_202D_202A_202C_200F_202E(theScanType, 1);
					if ((uint)c <= 72u)
					{
						num = 2021923842;
						num54 = num;
					}
					else
					{
						num = 1118124824;
						num54 = num;
					}
					continue;
					IL_0580:
					result = _202B_202B_200B_206F_206B_200C_206B_202B_206E_202E_202B_200D_206B_202C_206D_202A_202D_206C_206D_206A_206F_200C_200D_200C_206C_206C_202E_206D_200C_200F_206B_206B_206A_200E_200F_200E_200F_202B_206E_200C_202E("&nbsp;", theScanType, "&nbsp;");
					num = 1031587091;
					continue;
					IL_08b3:
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "AeBe"))
					{
						num = 604370145;
						num58 = num;
					}
					else
					{
						num = 2082102113;
						num58 = num;
					}
					continue;
					IL_0b9a:
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "DAB"))
					{
						num = 691587631;
						num61 = num;
					}
					else
					{
						num = 1910665933;
						num61 = num;
					}
					continue;
					IL_08a1:
					num = (int)((num2 * 1185382642) ^ 0x262E2B35);
					continue;
					IL_0a6d:
					if (_202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(theScanType, "CS"))
					{
						num = 319057665;
						num69 = num;
					}
					else
					{
						num = 83622723;
						num69 = num;
					}
					continue;
				}
				break;
			}
		}
	}
static string _202E_206A_206C_200D_202B_202C_200B_200C_200C_206F_200F_206A_202E_206C_200D_206B_202D_202D_200F_202C_200D_202B_206F_200B_202C_206D_202A_200C_202E_200E_206E_206D_202A_206E_202E_206C_206A_200B_202E_202C_202E(string[] P_0)
	{
		return string.Concat(P_0);
	}
static string _206A_202A_200F_202C_202A_200C_206D_206F_206B_206B_202A_206F_200D_202A_206A_206F_202E_202C_202A_200C_200D_206D_200E_206E_200B_206E_206E_202D_202B_202A_206A_200B_206F_200D_206D_202A_202E_200E_200D_206F_202E(string P_0, int P_1, int P_2)
	{
		return P_0.Substring(P_1, P_2);
	}
static string _202B_202B_200B_206F_206B_200C_206B_202B_206E_202E_202B_200D_206B_202C_206D_202A_202D_206C_206D_206A_206F_200C_200D_200C_206C_206C_202E_206D_200C_200F_206B_206B_206A_200E_200F_200E_200F_202B_206E_200C_202E(string P_0, string P_1, string P_2)
	{
		return P_0 + P_1 + P_2;
	}
static int _200E_200B_206A_202B_206B_200F_206D_206B_206B_206A_200B_206E_206A_206F_202A_202B_200E_200E_206E_206D_206A_202B_202C_206C_206A_202B_200E_206E_206B_206B_206E_200E_202B_202A_200C_200C_200B_200E_200E_202E_202E(string P_0)
	{
		return P_0.Length;
	}
static bool _202E_202B_200B_200F_202C_202D_200F_206D_200C_200E_206F_200F_206B_206E_200F_200D_202C_202D_200D_206F_200B_200F_206E_206B_200C_200C_200C_202E_200D_200B_200B_206B_200D_202E_206D_206C_206A_202E_206B_202C_202E(string P_0, string P_1)
	{
		return P_0 == P_1;
	}
static string _200D_202B_206E_200B_200B_202A_202D_206A_206F_200E_202B_202B_202A_202E_206C_202B_206B_202A_202D_202D_202B_202A_200D_202D_202B_206A_202C_202B_202C_206D_202B_202B_206E_200E_202E_206A_200F_206D_206D_206A_202E(string P_0, string P_1)
	{
		return P_0 + P_1;
	}
static string _206A_206B_206D_202C_202E_206D_206A_200B_206B_202C_200E_206E_202D_206E_206E_206E_200B_202B_200C_206C_206E_202A_202A_202E_200C_206B_200B_202E_206A_200E_206B_200C_200C_200D_200B_206E_200C_202D_200D_202E(string P_0, int P_1)
	{
		return P_0.Substring(P_1);
	}
static string _206B_202E_202A_202B_206D_200B_206B_206E_202D_206B_202D_200C_202E_202C_206D_206C_206F_200F_202E_200D_206E_200F_200C_206B_206F_206A_200E_206D_200F_200F_206D_200F_202D_206B_202A_206D_206E_200C_200E_202E(string P_0, string P_1, string P_2, string P_3)
	{
		return P_0 + P_1 + P_2 + P_3;
	}
static StringBuilder _202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(StringBuilder P_0, string P_1)
	{
		return P_0.Append(P_1);
	}
static char _202D_200B_206D_200C_206F_206E_206B_206E_200B_200B_202C_206B_206A_202B_200F_200F_206E_206F_206B_206F_206C_206A_200F_206F_206D_202C_202E_206C_200B_200E_202D_206F_206E_200D_206E_202E_202D_202A_202C_200F_202E(string P_0, int P_1)
	{
		return P_0[P_1];
	}
static double _206D_206E_200E_200F_200B_206B_206F_202D_206D_200E_202D_200E_202C_202E_206B_206C_200B_206F_202A_202C_200D_202B_206C_202E_202C_200E_200E_202B_200F_202D_200E_202E_200B_202C_206A_200D_202D_206B_202D_206F_202E(double P_0, int P_1)
	{
		return Math.Round(P_0, P_1);
	}
static bool _200C_202E_200B_206F_206E_200C_200F_202A_200D_206E_206C_202A_200E_200F_200F_200F_200B_202E_200D_200D_202A_206F_206A_206F_206C_206F_200C_206C_202B_200D_200F_202B_200D_200B_206D_206E_202C_200B_202C_206C_202E(string P_0, string P_1)
	{
		return P_0 != P_1;
	}
static StringBuilder _202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(StringBuilder P_0, ref StringBuilder.AppendInterpolatedStringHandler P_1)
	{
		return P_0.Append(ref P_1);
	}
public void GenerateAboutPage()
	{
		StringBuilder stringBuilder = _202E_206D_206E_206C_206A_202C_206A_202D_202D_206E_202A_206F_200B_206D_202D_206F_200D_200B_202E_200E_206C_200B_206F_206F_200D_200C_200B_200F_200C_206B_206E_202E_206A_200D_202A_202D_202C_200E_200E_206F_202E();
		OutputSymbol current = default;
		while (true)
		{
			int num = -351322893;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -238767406)) % 9)
				{
				case 8u:
					break;
				case 4u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</td><td>");
					num = -390651931;
					continue;
				case 1u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<table>");
					num = -1802534601;
					continue;
				case 6u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><th colspan=2 class=\"body border\">Symbol Legend</th></tr>");
					num = -593787537;
					continue;
				case 7u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<span class=\"thanks\">Enjoying Evaluator? <a class=\"buttonlink\" href=\"https://ko-fi.com/explorationtools\"><b>Buy me a coffee</b></a><br /><br /></span>");
					num = -2139850426;
					continue;
				case 5u:
					AddButtonsAndOpenContentDiv(stringBuilder);
					num = ((int)num2 * -1999457234) ^ -1638441639;
					continue;
				case 2u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, theme.SymbolLookup[OutputSymbol.WorthMapping].Description);
					num = (int)((num2 * 787972509) ^ 0x36C75B59);
					continue;
				case 3u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><th class=\"body border\">Symbol</th><th class=\"body border\">Meaning</th></tr>");
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><td align=\"center\">");
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<div class='custom-checkbox'><input type=\"checkbox\" class=\"checkbox\" checked /><label><span></span></label></div>");
					num = -515812097;
					continue;
				default:
				{
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</td></tr>");
					using (List<OutputSymbol>.Enumerator enumerator = SymbolList.GetEnumerator())
					{
						while (true)
						{
							IL_0193:
							int num3;
							int num4;
							if (enumerator.MoveNext())
							{
								num3 = -998709364;
								num4 = num3;
							}
							else
							{
								num3 = -1547974312;
								num4 = num3;
							}
							while (true)
							{
								switch ((num2 = (uint)(num3 ^ -238767406)) % 7)
								{
								case 5u:
									num3 = -998709364;
									continue;
								default:
									goto end_IL_0163;
								case 4u:
									break;
								case 6u:
									_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</td></tr>");
									num3 = -2121937347;
									continue;
								case 2u:
									current = enumerator.Current;
									num3 = -421553946;
									continue;
								case 3u:
									_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><td align=\"center\">");
									num3 = -883163826;
									continue;
								case 1u:
									_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, GetSymbol(current));
									_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</td><td>");
									_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, theme.SymbolLookup.ContainsKey(current) ? theme.SymbolLookup[current].Description : "&nbsp;");
									num3 = -1107362145;
									continue;
								case 0u:
									goto end_IL_0163;
								}
								goto IL_0193;
								continue;
								end_IL_0163:
								break;
							}
							break;
						}
					}
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><td align=\"center\">");
					while (true)
					{
						int num5 = -1709602829;
						while (true)
						{
							switch ((num2 = (uint)(num5 ^ -238767406)) % 10)
							{
							case 2u:
								break;
							case 4u:
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<span class=\"h1thanks\">Need assistance?<br /></span>");
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<span class=\"thanks\">Try the <a href=\"https://discord.gg/4J55fNdU7B\" class=\"link\"><b>Observatory Discord</b></a></span><br /><br />");
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<br />");
								num5 = -1127125953;
								continue;
							case 8u:
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<b>CMDR Vithigar</b> for creating <a href=\"https://observatory.xjph.net/\" class=\"link\"><b>Observatory</b></a> and listening to my feedback and requests<br />");
								num5 = -51024817;
								continue;
							case 5u:
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</td><td>");
								num5 = -1804234162;
								continue;
							case 6u:
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "Currently selected body filter");
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</td></tr>");
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</table><br />");
								num5 = -833831878;
								continue;
							case 3u:
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "Filter bodies (mapped / default / all)");
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</td></tr>");
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><td align=\"center\">");
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, GetSymbol(OutputSymbol.FilterDefaultSelected));
								num5 = -1547474543;
								continue;
							case 0u:
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<span class=\"h1thanks\">Thanks<br /></span>");
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<span class=\"thanks\">");
								num5 = -865675476;
								continue;
							case 9u:
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, GetSymbol(OutputSymbol.FilterDefault));
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</td><td>");
								num5 = -1419297283;
								continue;
							case 1u:
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "Evaluator uses some .SVGs from Iain Collins's <a href=\"https://github.com/iaincollins/icarus\" class=\"link\"><b>Icarus Terminal</b></a><br />");
								_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</span><br /><br />");
								num5 = -1693920854;
								continue;
							default:
								CloseContentDiv(stringBuilder);
								SavePageAndMaybeUpdate("about", stringBuilder, showIfCurrent: false);
								return;
							}
							break;
						}
					}
				}
				}
				break;
			}
		}
	}
public void GenerateUnsoldDataPage(List<(string name, DateTime dt, long value)> lines)
	{
		long num = 0L;
		StringBuilder stringBuilder = _202E_206D_206E_206C_206A_202C_206A_202D_202D_206E_202A_206F_200B_206D_202D_206F_200D_200B_202E_200E_206C_200B_206F_206F_200D_200C_200B_200F_200C_206B_206E_202E_206A_200D_202A_202D_202C_200E_200E_206F_202E();
		AddButtonsAndOpenContentDiv(stringBuilder);
		_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<table>");
		StringBuilder stringBuilder2 = _202E_206D_206E_206C_206A_202C_206A_202D_202D_206E_202A_206F_200B_206D_202D_206F_200D_200B_202E_200E_206C_200B_206F_206F_200D_200C_200B_200F_200C_206B_206E_202E_206A_200D_202A_202D_202C_200E_200E_206F_202E();
		int num2 = 1;
		string value = "";
		StringBuilder stringBuilder3 = default;
		using (List<(string, DateTime, long)>.Enumerator enumerator = lines.GetEnumerator())
		{
			(string, DateTime, long) current = default;
			DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = default;
			while (true)
			{
				IL_02e9:
				int num3;
				int num4;
				if (!enumerator.MoveNext())
				{
					num3 = 1760419064;
					num4 = num3;
				}
				else
				{
					num3 = 1273253385;
					num4 = num3;
				}
				while (true)
				{
					uint num5;
					switch ((num5 = (uint)(num3 ^ 0x29336E39)) % 15)
					{
					case 0u:
						num3 = 1273253385;
						continue;
					default:
						goto end_IL_0044;
					case 3u:
						current = enumerator.Current;
						num3 = 606449900;
						continue;
					case 8u:
						stringBuilder3 = stringBuilder2;
						num3 = ((int)num5 * -1904924755) ^ 0x5A28FAD3;
						continue;
					case 11u:
					{
						StringBuilder stringBuilder4 = stringBuilder3;
						StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(29, 2, stringBuilder3);
						appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
						appendInterpolatedStringHandler.AppendFormatted(value);
						appendInterpolatedStringHandler.AppendLiteral(" right\">");
						appendInterpolatedStringHandler.AppendFormatted(current.Item3.ToString("n", nfi));
						appendInterpolatedStringHandler.AppendLiteral("</td></tr>");
						_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder4, ref appendInterpolatedStringHandler);
						num3 = 1719215830;
						continue;
					}
					case 10u:
						num2 = 1;
						num3 = ((int)num5 * -1745488484) ^ 0x213C2F9F;
						continue;
					case 2u:
						value = defaultInterpolatedStringHandler.ToStringAndClear();
						num3 = ((int)num5 * -1682816280) ^ 0x449CCEA4;
						continue;
					case 12u:
					{
						int num8;
						int num9;
						if (num2 > 2)
						{
							num8 = 156573093;
							num9 = num8;
						}
						else
						{
							num8 = 1959261015;
							num9 = num8;
						}
						num3 = num8 ^ ((int)num5 * -414008680);
						continue;
					}
					case 9u:
						defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(3, 1);
						defaultInterpolatedStringHandler.AppendLiteral("row");
						defaultInterpolatedStringHandler.AppendFormatted(num2);
						num3 = 2053663032;
						continue;
					case 4u:
					{
						StringBuilder stringBuilder5 = stringBuilder3;
						StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(22, 2, stringBuilder3);
						appendInterpolatedStringHandler.AppendLiteral("<tr><td class=\"");
						appendInterpolatedStringHandler.AppendFormatted(value);
						appendInterpolatedStringHandler.AppendLiteral("\">");
						appendInterpolatedStringHandler.AppendFormatted(current.Item2.ToString("dd-MMM-yy HH:mm"));
						appendInterpolatedStringHandler.AppendLiteral("</td>");
						_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder5, ref appendInterpolatedStringHandler);
						stringBuilder3 = stringBuilder2;
						StringBuilder stringBuilder6 = stringBuilder3;
						appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(18, 2, stringBuilder3);
						appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
						appendInterpolatedStringHandler.AppendFormatted(value);
						appendInterpolatedStringHandler.AppendLiteral("\">");
						appendInterpolatedStringHandler.AppendFormatted(current.Item1);
						appendInterpolatedStringHandler.AppendLiteral("</td>");
						_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder6, ref appendInterpolatedStringHandler);
						num3 = 424239486;
						continue;
					}
					case 5u:
					{
						int num6;
						int num7;
						if (current.Item3 <= 0)
						{
							num6 = 878934206;
							num7 = num6;
						}
						else
						{
							num6 = 2018712365;
							num7 = num6;
						}
						num3 = num6 ^ (int)(num5 * 1933519048);
						continue;
					}
					case 7u:
						num2++;
						num3 = ((int)num5 * -1464008126) ^ 0x2CC9D8DC;
						continue;
					case 6u:
						break;
					case 1u:
						stringBuilder3 = stringBuilder2;
						num3 = ((int)num5 * -1099376187) ^ -481609469;
						continue;
					case 14u:
						num += current.Item3;
						num3 = 490021423;
						continue;
					case 13u:
						goto end_IL_0044;
					}
					goto IL_02e9;
					continue;
					end_IL_0044:
					break;
				}
				break;
			}
		}
		_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><th colspan=3>&nbsp;</th></tr>");
		stringBuilder3 = stringBuilder;
		while (true)
		{
			int num10 = 992317954;
			while (true)
			{
				uint num5;
				switch ((num5 = (uint)(num10 ^ 0x29336E39)) % 8)
				{
				case 2u:
					break;
				case 7u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "</tr>");
					_202D_206A_206B_200D_206B_206E_206E_202D_202E_200F_200E_200B_200D_202C_200B_206B_200D_206C_206B_206C_202E_202A_202B_206E_202C_200F_200D_202D_206A_200D_200B_202A_206B_206B_200F_202E_200D_200F_200C_206C_202E(stringBuilder, stringBuilder2);
					CloseContentDiv(stringBuilder);
					num10 = 1872155773;
					continue;
				case 6u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><td colspan=3>&nbsp;</td></tr>");
					num10 = 676012436;
					continue;
				case 1u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<th>Value</th>");
					num10 = 882054078;
					continue;
				case 3u:
				{
					StringBuilder stringBuilder7 = stringBuilder3;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(92, 1, stringBuilder3);
					appendInterpolatedStringHandler.AppendLiteral("<tr><th class=\"left\">Total Unsold Data (max)</th><th>&nbsp;</th><th class=\"right\">");
					appendInterpolatedStringHandler.AppendFormatted(num.ToString("n", nfi));
					appendInterpolatedStringHandler.AppendLiteral("</th></tr>");
					_202B_206C_206E_200C_206C_202E_202A_206B_202B_200F_200B_200E_202A_200B_202B_206E_206F_200C_206D_202C_200E_200D_206D_202D_202E_206A_206F_200B_206C_206A_206D_202D_206E_202A_206E_200F_206B_202C_202D_200F_202E(stringBuilder7, ref appendInterpolatedStringHandler);
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><th colspan=3>&nbsp;</th></tr>");
					num10 = 1540263369;
					continue;
				}
				case 0u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><td colspan=3>&nbsp;</td></tr>");
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><td colspan=3>&nbsp;</td></tr>");
					num10 = 635470415;
					continue;
				case 5u:
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><th colspan=3>Breakdown</th>");
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<tr><th>Date</th>");
					_202E_200F_200E_200E_200D_202A_206B_202E_206F_202E_200C_206E_206A_202B_200C_206A_206A_202D_202B_202A_202E_206A_200B_206F_206B_206F_206C_202B_206B_206B_200E_200F_206A_206C_202B_202A_202D_202E_202E_200C_202E(stringBuilder, "<th>System</th>");
					num10 = 890245648;
					continue;
				default:
					SavePageAndMaybeUpdate("unsold", stringBuilder, showIfCurrent: true);
					return;
				}
				break;
			}
		}
	}
static StringBuilder _202D_206A_206B_200D_206B_206E_206E_202D_202E_200F_200E_200B_200D_202C_200B_206B_200D_206C_206B_206C_202E_202A_202B_206E_202C_200F_200D_202D_206A_200D_200B_202A_206B_206B_200F_202E_200D_200F_200C_206C_202E(StringBuilder P_0, StringBuilder P_1)
	{
		return P_0.Append(P_1);
	}
static StringBuilder _202E_206D_206E_206C_206A_202C_206A_202D_202D_206E_202A_206F_200B_206D_202D_206F_200D_200B_202E_200E_206C_200B_206F_206F_200D_200C_200B_200F_200C_206B_206E_202E_206A_200D_202A_202D_202C_200E_200E_206F_202E()
	{
		return new StringBuilder();
	}
} public class EvaluatorSettings{public bool ShowHeaders=true,ShowTotal=false;} public class Theme{public Dictionary<OutputSymbol,Symbol>SymbolLookup=new();public Theme(){foreach(var key in Enum.GetValues<OutputSymbol>())SymbolLookup[key]=new(){Description="{{desc:"+key+"}}"};}public ThemeSettings Settings=new();}public class ThemeSettings{public bool UseHighlightRow=true;}public class Symbol{public string Description="";}