using System;using System.Text;using System.Runtime.CompilerServices;using System.Text.Json;using System.Collections.Generic;


public enum OutputSymbol
{
	PersonalCodexEntry,
	Mark,
	PossibleCodexEntry,
	PossibleCodexEntryNeverSeen,
	HighValue,
	CurrentBody,
	Material,
	StarType,
	Horizons,
	Unknown,
	NotPredicted,
	NagToUpdate,
	Tick,
	Cross,
	Likely,
	Maybe,
	Unlikely,
	ReadAllRequired,
	BiologicalLogged,
	BiologicalSampled,
	BiologicalComplete,
	Aborted,
	ColourIsMissing,
	Canonn,
	MR,
	HMC,
	RB,
	RIB,
	IB,
	OptionOn,
	OptionOff,
	PossibleCodexEntryAlternate,
	PersonalCodexEntryAlternate,
	MarkAlternate,
	BioCount,
	BioCountMismatch,
	BioCountAutoscan,
	KoFi
}
public enum PlanetClass {Metal_rich_body,Icy_body,High_metal_content_body,Rocky_body,Rocky_ice_body}
public enum VariantStar {G}
class Scan {public string BodyName="Reference System 1";}
class RadiantSourceStar {public VariantStar VariantStar=VariantStar.G;}
class RadiantSources {public RadiantSourceStar RadiantStar=new();}
class BScanPlanet:Scan {public PlanetClass PlanetClass=PlanetClass.Rocky_body;public int MinValue=6900000,MaxValue=12940000;public RadiantSources RadiantSources=new();public bool HasMarkedSpecies;public bool HasTooFewGenusPredictions;public bool HasPersonalCodexEntry;public bool HasHighValue;public bool HasHighValueBiological;public bool PossibleNewGalacticCodexEntryNeverSeen;public bool PossibleNewGalacticCodexEntry;}
class BSystem {public Dictionary<int,BScanPlanet> Planets=new(){{1,new()}};public Dictionary<int,int> BioCount=new(){{1,4}};public ulong ID64=42;}
class ThemeSettings {public bool UseHighlightRow=true;}
class Theme {public ThemeSettings Settings=new();}
class UIHandler {
 Theme theme=new();string BaseURL="http://localhost/bioinsights";Dictionary<PlanetClass,OutputSymbol> planetSymbol=new(){{PlanetClass.Rocky_body,OutputSymbol.RB},{PlanetClass.Metal_rich_body,OutputSymbol.MR},{PlanetClass.Icy_body,OutputSymbol.IB},{PlanetClass.High_metal_content_body,OutputSymbol.HMC},{PlanetClass.Rocky_ice_body,OutputSymbol.RIB}};
 string GetSymbol(OutputSymbol symbol)=>"["+symbol+"]";string GetSymbol(OutputSymbol symbol,string content)=>GetSymbol((int)symbol,content);string GetSymbol(int symbol,string content)=>"["+(OutputSymbol)symbol+":"+content+"]";
public void GenerateHTMLBodyHeader(BSystem sys, int bodyid, bool current, StringBuilder ret)
	{
		_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, "<tr>");
		BScanPlanet bScanPlanet2 = default;
		BScanPlanet bScanPlanet = default;
		int num5 = default;
		DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = default;
		string text = default;
		string text3 = default;
		StringBuilder stringBuilder2 = default;
		string value = default;
		int num6 = default;
		while (true)
		{
			int num = 213298144;
			while (true)
			{
				uint num2;
				string text2;
				switch ((num2 = (uint)(num ^ 0x21CAFBCE)) % 47)
				{
				case 39u:
					break;
				default:
					return;
				case 13u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E("<span class=\"body\">", _200D_200F_200C_200E_200B_200F_206C_206B_206C_200B_202E_206D_206D_206F_202E_206B_206E_202D_200C_200C_200E_202D_200D_206A_200B_206F_206C_206B_202C_206D_206D_206B_200F_200F_202C_206F_206B_206A_206A_202C_202E((Scan)(object)bScanPlanet2), "</span></td>"));
					num = 1975888923;
					continue;
				case 24u:
				{
					int sym = (_206B_200F_206C_206B_206D_200D_202A_206A_200B_206D_206D_200F_200C_206E_206D_202C_206A_202E_202B_206A_200F_200E_206D_200D_202C_206B_206A_202B_202D_202C_206D_202B_200D_202D_206F_206D_200D_202E_202C_200D_202E(bScanPlanet) ? 35 : 34);
					string content;
					if (num5 != 1)
					{
						defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(5, 1);
						defaultInterpolatedStringHandler.AppendFormatted(num5);
						defaultInterpolatedStringHandler.AppendLiteral(" bios");
						content = defaultInterpolatedStringHandler.ToStringAndClear();
					}
					else
					{
						content = "1 bio";
					}
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, GetSymbol((OutputSymbol)sym, content));
					num = 1728531437;
					continue;
				}
				case 11u:
				{
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, _206D_206F_206B_202C_202A_206C_206C_200C_202E_206A_206F_206C_206D_206C_206C_200D_202E_200E_206A_200F_206A_202D_202B_206B_206F_206A_206D_200E_200D_206C_202D_202A_202C_206A_200E_206E_202C_206E_200D_202A_202E(bScanPlanet) ? (GetSymbol(OutputSymbol.Mark) ?? "") : "");
					int num22;
					if (_206B_206E_200D_202C_206B_202C_200F_200E_200E_200E_200D_200B_200F_202B_206A_206F_202B_200C_206F_202E_200F_202D_200C_206C_202E_206F_202E_200B_200C_200E_200D_206A_202B_206A_206E_200D_200D_200D_200C_200F_202E(bScanPlanet))
					{
						num = 2027918385;
						num22 = num;
					}
					else
					{
						num = 1936106282;
						num22 = num;
					}
					continue;
				}
				case 40u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(GetSymbol(OutputSymbol.PossibleCodexEntryNeverSeen), " "));
					num = 1478976150;
					continue;
				case 42u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(new string[5]
					{
						"<td colspan=3 class=\"radiant ",
						text,
						"\"><span title=\"Radiant Star\">Star: ",
						_200B_206F_206C_202D_200D_200E_200F_206F_202C_206F_206B_206B_206C_200F_200D_206B_200E_206A_200E_202D_200B_202A_206C_202C_202A_200D_202D_202C_202C_200B_200B_206A_202D_206E_202C_200B_200E_206E_202D_202B_202E(_206C_202D_202C_206D_200F_206D_206A_200F_202C_200C_202A_200D_202D_202B_200E_206B_202A_202D_206E_206A_202D_202C_206C_200C_200D_202E_202B_206D_200E_200D_202A_200C_200E_202E_200C_202B_206A_200F_206B_200F_202E(_206D_206F_202A_206B_206F_202C_206B_200F_206A_202B_200B_206C_200E_200E_200D_202D_206D_202E_206F_202E_200D_200C_200C_200D_200F_206A_206C_202D_206F_202C_200F_206D_206F_206F_200D_200E_202B_206C_200F_206C_202E(bScanPlanet))).ToString(),
						"</span></td>"
					}));
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(new string[5]
					{
						"<td colspan=2 class=\"value ",
						text,
						"\"><span title=\"Base value\">",
						text3,
						"</span></td>"
					}));
					num = 396120580;
					continue;
				case 7u:
					num = (int)((num2 * 614630953) ^ 0x7F8D471F);
					continue;
				case 12u:
					bScanPlanet = sys.Planets[bodyid];
					num = (int)((num2 * 1343812093) ^ 0x505682E5);
					continue;
				case 10u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, (_200C_206F_200B_200B_200E_206E_200C_206C_202A_200D_206A_202B_202A_202C_206D_200B_200D_202D_206C_200C_202A_200F_202B_206F_200B_200C_206C_202D_200B_202B_202B_206D_206F_200F_206C_202A_202D_202C_200F_206E_202E(bScanPlanet) | _200E_200E_200D_206B_206D_206C_206D_200E_202B_202C_202E_202A_206E_200F_200B_206F_202C_200D_202B_200E_200D_206B_202C_200D_206F_202C_206E_200D_200F_200D_202C_202C_202A_206D_202C_200D_206A_200B_202B_200F_202E(bScanPlanet)) ? (GetSymbol(OutputSymbol.HighValue) ?? "") : "");
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, _206D_206C_206B_206B_202C_206F_200D_206E_202B_202B_200D_202B_202C_206B_202D_200F_202C_202D_202B_202D_200F_200F_202E_200D_206B_200C_200C_202A_202E_200D_200D_202E_202D_206F_200B_202D_200F_206E_206C_200C_202E(bScanPlanet) ? (GetSymbol(OutputSymbol.PersonalCodexEntry) ?? "") : "");
					num = 26553750;
					continue;
				case 43u:
					bScanPlanet2 = sys.Planets[bodyid];
					num = (int)((num2 * 92051655) ^ 0x3237B050);
					continue;
				case 46u:
				{
					int num18;
					int num19;
					if (num5 == -1)
					{
						num18 = 121342066;
						num19 = num18;
					}
					else
					{
						num18 = 1226520133;
						num19 = num18;
					}
					num = num18 ^ (int)(num2 * 1346911890);
					continue;
				}
				case 25u:
				{
					int num17;
					if (!_202D_202C_200C_200D_200B_202D_202D_202C_206E_202D_200E_206B_200C_200E_202D_202A_206C_200D_206F_202B_200C_206C_206B_206D_206A_202C_200C_202A_202A_202C_206D_206B_202B_202B_206F_202E_202C_202C_206D_200D_202E(bScanPlanet))
					{
						num = 814536711;
						num17 = num;
					}
					else
					{
						num = 1867539424;
						num17 = num;
					}
					continue;
				}
				case 5u:
				{
					stringBuilder2 = ret;
					StringBuilder stringBuilder5 = stringBuilder2;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(44, 2, stringBuilder2);
					appendInterpolatedStringHandler.AppendLiteral("<td class=\"body roundleft ");
					appendInterpolatedStringHandler.AppendFormatted(text);
					appendInterpolatedStringHandler.AppendLiteral("\" id=\"");
					appendInterpolatedStringHandler.AppendFormatted(value);
					appendInterpolatedStringHandler.AppendLiteral("\" colspan=4>");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder5, ref appendInterpolatedStringHandler);
					num = 1453706679;
					continue;
				}
				case 20u:
					num = (int)(num2 * 1283848475) ^ -2113841598;
					continue;
				case 30u:
				{
					int num11;
					int num12;
					if (sys.Planets.ContainsKey(bodyid))
					{
						num11 = -89562644;
						num12 = num11;
					}
					else
					{
						num11 = -280911417;
						num12 = num11;
					}
					num = num11 ^ (int)(num2 * 1378424695);
					continue;
				}
				case 31u:
					text2 = "";
					goto IL_045b;
				case 27u:
				{
					stringBuilder2 = ret;
					StringBuilder stringBuilder4 = stringBuilder2;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(44, 2, stringBuilder2);
					appendInterpolatedStringHandler.AppendLiteral("<td class=\"body roundleft ");
					appendInterpolatedStringHandler.AppendFormatted(text);
					appendInterpolatedStringHandler.AppendLiteral("\" id=\"");
					appendInterpolatedStringHandler.AppendFormatted(value);
					appendInterpolatedStringHandler.AppendLiteral("\" colspan=4>");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder4, ref appendInterpolatedStringHandler);
					num = 858736238;
					continue;
				}
				case 1u:
					num = ((int)num2 * -2110786480) ^ 0x7C48A143;
					continue;
				case 37u:
					value = defaultInterpolatedStringHandler.ToStringAndClear();
					num = ((int)num2 * -165870469) ^ -1267393888;
					continue;
				case 29u:
					text3 = _206F_206D_202E_200F_206F_202E_202B_202D_206A_202E_200C_206D_206C_202A_202D_202A_206B_200F_206E_206D_202E_200B_202A_206C_202D_206D_206D_202E_200C_202E_200F_200D_200B_206C_206D_200D_202C_200B_206A_202C_202E(FormatMillions(_202D_202D_200E_202E_206D_200C_202C_206B_200F_200F_206C_200B_200D_206A_206F_206D_202E_202B_202C_206E_206F_200F_200D_202E_202E_206E_202E_200E_200D_206B_202B_200C_202B_200B_202A_206D_206F_206B_206A_206A_202E(bScanPlanet)), "M - ", FormatMillions(_206D_202A_206A_200E_206C_202A_206A_202D_200C_206B_202A_202C_206F_202C_200D_202B_206A_202D_206A_200C_202D_206A_206A_202E_200E_206C_202C_200B_200E_206B_200E_206F_202D_206C_202D_206A_206B_200C_202C_200F_202E(bScanPlanet)), "M");
					num = 100048627;
					continue;
				case 26u:
					num6 = sys.BioCount[bodyid];
					num = ((int)num2 * -1305616599) ^ -40083878;
					continue;
				case 9u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(GetSymbol(OutputSymbol.PossibleCodexEntry), " "));
					num = 814536711;
					continue;
				case 45u:
				{
					StringBuilder stringBuilder = stringBuilder2;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(31, 1, stringBuilder2);
					appendInterpolatedStringHandler.AppendLiteral("<td colspan=1 class=\"signals ");
					appendInterpolatedStringHandler.AppendFormatted(text);
					appendInterpolatedStringHandler.AppendLiteral("\">");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder, ref appendInterpolatedStringHandler);
					num5 = sys.BioCount[bodyid];
					num = 322921538;
					continue;
				}
				case 44u:
				{
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(_200D_200F_200C_200E_200B_200F_206C_206B_206C_200B_202E_206D_206D_206F_202E_206B_206E_202D_200C_200C_200E_202D_200D_206A_200B_206F_206C_206B_202C_206D_206D_206B_200F_200F_202C_206F_206B_206A_206A_202C_202E((Scan)(object)bScanPlanet), "</td>"));
					stringBuilder2 = ret;
					StringBuilder stringBuilder3 = stringBuilder2;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(20, 1, stringBuilder2);
					appendInterpolatedStringHandler.AppendLiteral("<td class=\"planet ");
					appendInterpolatedStringHandler.AppendFormatted(text);
					appendInterpolatedStringHandler.AppendLiteral("\">");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder3, ref appendInterpolatedStringHandler);
					num = 1135086521;
					continue;
				}
				case 35u:
				{
					int num20;
					int num21;
					if (current)
					{
						num20 = 1601572523;
						num21 = num20;
					}
					else
					{
						num20 = 404167332;
						num21 = num20;
					}
					num = num20 ^ ((int)num2 * -2098932567);
					continue;
				}
				case 34u:
				{
					string text4 = "<td colspan=5>";
					string text5;
					if (num6 != -1)
					{
						defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(10, 1);
						defaultInterpolatedStringHandler.AppendFormatted(num6);
						defaultInterpolatedStringHandler.AppendLiteral(" bios</td>");
						text5 = defaultInterpolatedStringHandler.ToStringAndClear();
					}
					else
					{
						text5 = "?";
					}
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text4, text5));
					num = 362242;
					continue;
				}
				case 41u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, "</td>");
					num = 1303399394;
					continue;
				case 32u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, "</tr>");
					num = 232948081;
					continue;
				case 15u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, "<span class=\"body\">unknown</span></td>");
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, "<td colspan=5>? signals</td>");
					num = 1161937862;
					continue;
				case 2u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, "</tr>");
					return;
				case 19u:
				{
					StringBuilder stringBuilder6 = stringBuilder2;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(44, 2, stringBuilder2);
					appendInterpolatedStringHandler.AppendLiteral("<td class=\"body roundleft ");
					appendInterpolatedStringHandler.AppendFormatted(text);
					appendInterpolatedStringHandler.AppendLiteral("\" id=\"");
					appendInterpolatedStringHandler.AppendFormatted(value);
					appendInterpolatedStringHandler.AppendLiteral("\" colspan=4>");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder6, ref appendInterpolatedStringHandler);
					int num16;
					if (current)
					{
						num = 743371554;
						num16 = num;
					}
					else
					{
						num = 880652554;
						num16 = num;
					}
					continue;
				}
				case 0u:
				{
					int num15;
					if (sys.Planets.ContainsKey(bodyid))
					{
						num = 1269306651;
						num15 = num;
					}
					else
					{
						num = 1602819494;
						num15 = num;
					}
					continue;
				}
				case 17u:
					stringBuilder2 = ret;
					num = (int)(num2 * 930648011) ^ -1309021521;
					continue;
				case 4u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, GetSymbol(planetSymbol[_202E_206F_202A_200F_200B_202C_202A_200C_200E_202A_202B_202A_206E_206D_200D_206A_200B_202B_202A_200E_206A_206F_200C_206C_206B_206F_206B_206D_206D_200B_200B_200B_206A_202A_200F_206B_206B_200C_206F_206A_202E(bScanPlanet)]));
					num = (int)(num2 * 1993386854) ^ -1584840348;
					continue;
				case 33u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, GetSymbol(OutputSymbol.CurrentBody));
					num = (int)(num2 * 1722653628) ^ -643514278;
					continue;
				case 18u:
					text3 = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(FormatMillions(_202D_202D_200E_202E_206D_200C_202C_206B_200F_200F_206C_200B_200D_206A_206F_206D_202E_202B_202C_206E_206F_200F_200D_202E_202E_206E_202E_200E_200D_206B_202B_200C_202B_200B_202A_206D_206F_206B_206A_206A_202E(bScanPlanet)), "M");
					num = 1097339377;
					continue;
				case 28u:
					defaultInterpolatedStringHandler.AppendLiteral("-");
					defaultInterpolatedStringHandler.AppendFormatted(bodyid);
					num = 2044027297;
					continue;
				case 16u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, "</td>");
					stringBuilder2 = ret;
					num = 1282533178;
					continue;
				case 21u:
					ret.TrimEnd();
					num = 1765994085;
					continue;
				case 22u:
				{
					int num13;
					int num14;
					if (planetSymbol.ContainsKey(_202E_206F_202A_200F_200B_202C_202A_200C_200E_202A_202B_202A_206E_206D_200D_206A_200B_202B_202A_200E_206A_206F_200C_206C_206B_206F_206B_206D_206D_200B_200B_200B_206A_202A_200F_206B_206B_200C_206F_206A_202E(bScanPlanet)))
					{
						num13 = 1389474589;
						num14 = num13;
					}
					else
					{
						num13 = 779934116;
						num14 = num13;
					}
					num = num13 ^ ((int)num2 * -677712734);
					continue;
				}
				case 8u:
				{
					text3 = "";
					int num9;
					int num10;
					if (_202D_202D_200E_202E_206D_200C_202C_206B_200F_200F_206C_200B_200D_206A_206F_206D_202E_202B_202C_206E_206F_200F_200D_202E_202E_206E_202E_200E_200D_206B_202B_200C_202B_200B_202A_206D_206F_206B_206A_206A_202E(bScanPlanet) > 0)
					{
						num9 = -1860777526;
						num10 = num9;
					}
					else
					{
						num9 = -491026267;
						num10 = num9;
					}
					num = num9 ^ (int)(num2 * 1152762114);
					continue;
				}
				case 6u:
					defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(1, 2);
					defaultInterpolatedStringHandler.AppendFormatted(sys.ID64);
					num = ((int)num2 * -1926405103) ^ 0x654B2404;
					continue;
				case 3u:
				{
					int num7;
					int num8;
					if (_206D_202A_206A_200E_206C_202A_206A_202D_200C_206B_202A_202C_206F_202C_200D_202B_206A_202D_206A_200C_202D_206A_206A_202E_200E_206C_202C_200B_200E_206B_200E_206F_202D_206C_202D_206A_206B_200C_202C_200F_202E(bScanPlanet) > 0)
					{
						num7 = -290769958;
						num8 = num7;
					}
					else
					{
						num7 = -1859084567;
						num8 = num7;
					}
					num = num7 ^ (int)(num2 * 563618629);
					continue;
				}
				case 38u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ret, GetSymbol(OutputSymbol.BioCountAutoscan));
					num = ((int)num2 * -1076030756) ^ 0x5AA30E1D;
					continue;
				case 14u:
					if (theme.Settings.UseHighlightRow)
					{
						text2 = "highlight";
						goto IL_045b;
					}
					num = ((int)num2 * -1761009191) ^ -238872463;
					continue;
				case 23u:
				{
					int num3;
					int num4;
					if (_202D_202D_200E_202E_206D_200C_202C_206B_200F_200F_206C_200B_200D_206A_206F_206D_202E_202B_202C_206E_206F_200F_200D_202E_202E_206E_202E_200E_200D_206B_202B_200C_202B_200B_202A_206D_206F_206B_206A_206A_202E(bScanPlanet) == _206D_202A_206A_200E_206C_202A_206A_202D_200C_206B_202A_202C_206F_202C_200D_202B_206A_202D_206A_200C_202D_206A_206A_202E_200E_206C_202C_200B_200E_206B_200E_206F_202D_206C_202D_206A_206B_200C_202C_200F_202E(bScanPlanet))
					{
						num3 = -182605069;
						num4 = num3;
					}
					else
					{
						num3 = -657238464;
						num4 = num3;
					}
					num = num3 ^ ((int)num2 * -1402355222);
					continue;
				}
				case 36u:
					return;
					IL_045b:
					text = text2;
					num = 221691879;
					continue;
				}
				break;
			}
		}
	}

public string FormatMillions(double val)
	{
		string text = _200C_200C_206C_206F_200E_202E_200C_206F_202E_206B_202D_200E_202E_200E_206C_206A_206C_206C_200F_202E_200F_206B_206D_200F_200E_200C_200F_206F_206B_200D_200F_206B_206A_202D_206D_200C_200E_200F_200F_202C_202E(val / 1000000.0, 2).ToString();
		_200B_206B_202D_200E_206E_206E_200D_200B_206C_206A_202A_202E_206B_206B_202D_200F_202D_200B_202A_200B_202E_206D_206A_206D_206E_202C_202A_200C_202E_200B_200F_206E_200D_202B_202B_200D_200F_200F_206D_200E_202E(_200B_206B_202D_200E_206E_206E_200D_200B_206C_206A_202A_202E_206B_206B_202D_200F_202D_200B_202A_200B_202E_206D_206A_206D_206E_202C_202A_200C_202E_200B_200F_206E_200D_202B_202B_200D_200F_200F_206D_200E_202E(_200B_206B_202D_200E_206E_206E_200D_200B_206C_206A_202A_202E_206B_206B_202D_200F_202D_200B_202A_200B_202E_206D_206A_206D_206E_202C_202A_200C_202E_200B_200F_206E_200D_202B_202B_200D_200F_200F_206D_200E_202E(text, '0'), '0'), '.');
		return text;
	}

	

	static bool _206D_206F_206B_202C_202A_206C_206C_200C_202E_206A_206F_206C_206D_206C_206C_200D_202E_200E_206A_200F_206A_202D_202B_206B_206F_206A_206D_200E_200D_206C_202D_202A_202C_206A_200E_206E_202C_206E_200D_202A_202E(BScanPlanet P_0)
	{
		return P_0.HasMarkedSpecies;
	}

	static string _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(string P_0, string P_1, string P_2)
	{
		return P_0 + P_1 + P_2;
	}

	static bool _206D_206C_206B_206B_202C_206F_200D_206E_202B_202B_200D_202B_202C_206B_202D_200F_202C_202D_202B_202D_200F_200F_202E_200D_206B_200C_200C_202A_202E_200D_200D_202E_202D_206F_200B_202D_200F_206E_206C_200C_202E(BScanPlanet P_0)
	{
		return P_0.HasPersonalCodexEntry;
	}

	static int _206D_202A_206A_200E_206C_202A_206A_202D_200C_206B_202A_202C_206F_202C_200D_202B_206A_202D_206A_200C_202D_206A_206A_202E_200E_206C_202C_200B_200E_206B_200E_206F_202D_206C_202D_206A_206B_200C_202C_200F_202E(BScanPlanet P_0)
	{
		return P_0.MaxValue;
	}

	static StringBuilder _206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(StringBuilder P_0, string P_1)
	{
		return P_0.Append(P_1);
	}

	static string _206F_206D_202E_200F_206F_202E_202B_202D_206A_202E_200C_206D_206C_202A_202D_202A_206B_200F_206E_206D_202E_200B_202A_206C_202D_206D_206D_202E_200C_202E_200F_200D_200B_206C_206D_200D_202C_200B_206A_202C_202E(string P_0, string P_1, string P_2, string P_3)
	{
		return P_0 + P_1 + P_2 + P_3;
	}

	static string _200D_200F_200C_200E_200B_200F_206C_206B_206C_200B_202E_206D_206D_206F_202E_206B_206E_202D_200C_200C_200E_202D_200D_206A_200B_206F_206C_206B_202C_206D_206D_206B_200F_200F_202C_206F_206B_206A_206A_202C_202E(Scan P_0)
	{
		return P_0.BodyName;
	}

	static RadiantSourceStar _206C_202D_202C_206D_200F_206D_206A_200F_202C_200C_202A_200D_202D_202B_200E_206B_202A_202D_206E_206A_202D_202C_206C_200C_200D_202E_202B_206D_200E_200D_202A_200C_200E_202E_200C_202B_206A_200F_206B_200F_202E(RadiantSources P_0)
	{
		return P_0.RadiantStar;
	}

	static PlanetClass _202E_206F_202A_200F_200B_202C_202A_200C_200E_202A_202B_202A_206E_206D_200D_206A_200B_202B_202A_200E_206A_206F_200C_206C_206B_206F_206B_206D_206D_200B_200B_200B_206A_202A_200F_206B_206B_200C_206F_206A_202E(BScanPlanet P_0)
	{
		return P_0.PlanetClass;
	}

	static string _200B_206B_202D_200E_206E_206E_200D_200B_206C_206A_202A_202E_206B_206B_202D_200F_202D_200B_202A_200B_202E_206D_206A_206D_206E_202C_202A_200C_202E_200B_200F_206E_200D_202B_202B_200D_200F_200F_206D_200E_202E(string P_0, char P_1)
	{
		return P_0.TrimEnd(P_1);
	}

	static RadiantSources _206D_206F_202A_206B_206F_202C_206B_200F_206A_202B_200B_206C_200E_200E_200D_202D_206D_202E_206F_202E_200D_200C_200C_200D_200F_206A_206C_202D_206F_202C_200F_206D_206F_206F_200D_200E_202B_206C_200F_206C_202E(BScanPlanet P_0)
	{
		return P_0.RadiantSources;
	}

	static bool _200E_200E_200D_206B_206D_206C_206D_200E_202B_202C_202E_202A_206E_200F_200B_206F_202C_200D_202B_200E_200D_206B_202C_200D_206F_202C_206E_200D_200F_200D_202C_202C_202A_206D_202C_200D_206A_200B_202B_200F_202E(BScanPlanet P_0)
	{
		return P_0.HasHighValueBiological;
	}

	static bool _200C_206F_200B_200B_200E_206E_200C_206C_202A_200D_206A_202B_202A_202C_206D_200B_200D_202D_206C_200C_202A_200F_202B_206F_200B_200C_206C_202D_200B_202B_202B_206D_206F_200F_206C_202A_202D_202C_200F_206E_202E(BScanPlanet P_0)
	{
		return P_0.HasHighValue;
	}

	static string _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(string[] P_0)
	{
		return string.Concat(P_0);
	}

	static StringBuilder _200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(StringBuilder P_0, ref StringBuilder.AppendInterpolatedStringHandler P_1)
	{
		return P_0.Append(ref P_1);
	}

	static double _200C_200C_206C_206F_200E_202E_200C_206F_202E_206B_202D_200E_202E_200E_206C_206A_206C_206C_200F_202E_200F_206B_206D_200F_200E_200C_200F_206F_206B_200D_200F_206B_206A_202D_206D_200C_200E_200F_200F_202C_202E(double P_0, int P_1)
	{
		return Math.Round(P_0, P_1);
	}

	static string _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(string P_0, string P_1)
	{
		return P_0 + P_1;
	}

	static VariantStar _200B_206F_206C_202D_200D_200E_200F_206F_202C_206F_206B_206B_206C_200F_200D_206B_200E_206A_200E_202D_200B_202A_206C_202C_202A_200D_202D_202C_202C_200B_200B_206A_202D_206E_202C_200B_200E_206E_202D_202B_202E(RadiantSourceStar P_0)
	{
		return P_0.VariantStar;
	}

	static bool _206B_200F_206C_206B_206D_200D_202A_206A_200B_206D_206D_200F_200C_206E_206D_202C_206A_202E_202B_206A_200F_200E_206D_200D_202C_206B_206A_202B_202D_202C_206D_202B_200D_202D_206F_206D_200D_202E_202C_200D_202E(BScanPlanet P_0)
	{
		return P_0.HasTooFewGenusPredictions;
	}

	static bool _202D_202C_200C_200D_200B_202D_202D_202C_206E_202D_200E_206B_200C_200E_202D_202A_206C_200D_206F_202B_200C_206C_206B_206D_206A_202C_200C_202A_202A_202C_206D_206B_202B_202B_206F_202E_202C_202C_206D_200D_202E(BScanPlanet P_0)
	{
		return P_0.PossibleNewGalacticCodexEntry;
	}

	static int _202D_202D_200E_202E_206D_200C_202C_206B_200F_200F_206C_200B_200D_206A_206F_206D_202E_202B_202C_206E_206F_200F_200D_202E_202E_206E_202E_200E_200D_206B_202B_200C_202B_200B_202A_206D_206F_206B_206A_206A_202E(BScanPlanet P_0)
	{
		return P_0.MinValue;
	}

	static bool _206B_206E_200D_202C_206B_202C_200F_200E_200E_200E_200D_200B_200F_202B_206A_206F_202B_200C_206F_202E_200F_202D_200C_206C_202E_206F_202E_200B_200C_200E_200D_206A_202B_206A_206E_200D_200D_200D_200C_200F_202E(BScanPlanet P_0)
	{
		return P_0.PossibleNewGalacticCodexEntryNeverSeen;
	}
static void Main(){var results=new List<object>();foreach(bool current in new[]{false,true})foreach(int signals in new[]{0,1,4})foreach(bool flags in new[]{false,true}){var o=new UIHandler();var sys=new BSystem();sys.BioCount[1]=signals;var p=sys.Planets[1];p.HasMarkedSpecies=flags;p.HasHighValue=flags;p.HasPersonalCodexEntry=flags;p.PossibleNewGalacticCodexEntry=flags;var s=new StringBuilder();o.GenerateHTMLBodyHeader(sys,1,current,s);results.Add(new{current,signals,flags,html=s.ToString()});}Console.WriteLine(JsonSerializer.Serialize(results));}
}





public static class StringBuilderExtensions
{
	public static StringBuilder TrimEnd(this StringBuilder sb)
	{
		if (sb != null)
		{
			int num3 = default;
			while (true)
			{
				int num = 2037152053;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ 0x534E3C4B)) % 11)
					{
					case 4u:
						break;
					case 10u:
						_202D_206F_206E_200D_202C_206A_202B_206B_206D_206F_202A_200C_202E_206A_206C_202C_202D_206D_206F_206E_200C_206E_200C_206C_202B_206D_206B_206A_200E_200E_200B_206C_206C_200F_206B_202B_206E_202D_202C_200F_202E(sb, num3 + 1);
						num = ((int)num2 * -725193045) ^ 0x668211A5;
						continue;
					case 8u:
						goto IL_0064;
					case 7u:
						goto IL_0083;
					case 2u:
					{
						int num4;
						int num5;
						if (_202C_206A_206C_200B_200D_202B_206D_200B_202C_206D_206E_206A_206A_200C_202A_206F_202B_206D_200F_206B_206C_200F_202D_200E_200F_206E_200B_200F_200F_200C_202D_206E_206F_206A_202D_206D_200B_202C_200D_206C_202E(sb) != 0)
						{
							num4 = 1191568415;
							num5 = num4;
						}
						else
						{
							num4 = 2052585769;
							num5 = num4;
						}
						num = num4 ^ (int)(num2 * 342375936);
						continue;
					}
					case 1u:
						num3 = _202C_206A_206C_200B_200D_202B_206D_200B_202C_206D_206E_206A_206A_200C_202A_206F_202B_206D_200F_206B_206C_200F_202D_200E_200F_206E_200B_200F_200F_200C_202D_206E_206F_206A_202D_206D_200B_202C_200D_206C_202E(sb) - 1;
						num = 1362235978;
						continue;
					case 0u:
						num = (int)(num2 * 846788469) ^ -1608514766;
						continue;
					case 6u:
						goto end_IL_0006;
					case 3u:
						goto IL_00f7;
					case 9u:
						num3--;
						num = (int)((num2 * 193788399) ^ 0x4A4FC152);
						continue;
					default:
						return sb;
					}
					break;
					IL_00f7:
					int num6;
					if (num3 < 0)
					{
						num = 189695143;
						num6 = num;
					}
					else
					{
						num = 2097321572;
						num6 = num;
					}
					continue;
					IL_0083:
					int num7;
					if (num3 < _202C_206A_206C_200B_200D_202B_206D_200B_202C_206D_206E_206A_206A_200C_202A_206F_202B_206D_200F_206B_206C_200F_202D_200E_200F_206E_200B_200F_200F_200C_202D_206E_206F_206A_202D_206D_200B_202C_200D_206C_202E(sb) - 1)
					{
						num = 762584683;
						num7 = num;
					}
					else
					{
						num = 1455200965;
						num7 = num;
					}
					continue;
					IL_0064:
					int num8;
					if (!char.IsWhiteSpace(_200F_202E_202C_206F_202A_202A_206E_200C_200B_200F_206A_200D_200F_206A_206F_200D_202A_200D_200E_206E_206A_206C_202E_202B_206F_202A_206B_202D_202E_200C_206F_200F_206C_206B_202E_202D_202D_202C_202C_200D_202E(sb, num3)))
					{
						num = 189695143;
						num8 = num;
					}
					else
					{
						num = 1408407408;
						num8 = num;
					}
				}
				continue;
				end_IL_0006:
				break;
			}
		}
		return sb;
	}

	static int _202C_206A_206C_200B_200D_202B_206D_200B_202C_206D_206E_206A_206A_200C_202A_206F_202B_206D_200F_206B_206C_200F_202D_200E_200F_206E_200B_200F_200F_200C_202D_206E_206F_206A_202D_206D_200B_202C_200D_206C_202E(StringBuilder P_0)
	{
		return P_0.Length;
	}

	static char _200F_202E_202C_206F_202A_202A_206E_200C_200B_200F_206A_200D_200F_206A_206F_200D_202A_200D_200E_206E_206A_206C_202E_202B_206F_202A_206B_202D_202E_200C_206F_200F_206C_206B_202E_202D_202D_202C_202C_200D_202E(StringBuilder P_0, int P_1)
	{
		return P_0[P_1];
	}

	static void _202D_206F_206E_200D_202C_206A_202B_206B_206D_206F_202A_200C_202E_206A_206C_202C_202D_206D_206F_206E_200C_206E_200C_206C_202B_206D_206B_206A_200E_200E_200B_206C_206C_200F_206B_202B_206E_202D_202C_200F_202E(StringBuilder P_0, int P_1)
	{
		P_0.Length = P_1;
	}
}
