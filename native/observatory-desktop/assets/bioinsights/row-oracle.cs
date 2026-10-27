using System;using System.Text;using System.Text.Json;using System.Collections.Generic;


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



public enum BioAnalysed
{
	No,
	Logged,
	Sampled,
	Complete,
	Incomplete
}



public enum BioSeen
{
	NotSeen,
	Seen,
	NotPresent
}



public enum VariantType
{
	Unknown,
	NotPredicted,
	SurfaceMaterial,
	ParentStar,
	Horizons
}



public enum VariantColour
{
	Unknown,
	Amethyst,
	Aquamarine,
	Blue,
	Cobalt,
	Cyan,
	Emerald,
	Gold,
	Green,
	Grey,
	Indigo,
	Lime,
	Magenta,
	Maroon,
	Mauve,
	Mulberry,
	Ocher,
	Orange,
	Peach,
	Red,
	Sage,
	Teal,
	Turquoise,
	White,
	Yellow,
	Horizons
}






public class VariantOutputLine
{
	public bool IsPersonalCodexEntry { get; set; }

	public bool IsGalacticCodexEntry { get; set; }

	public bool IsGalacticCodexEntryNeverSeen { get; set; }

	public bool IsMarked { get; set; }

	public bool IsInCannon { get; set; }

	public bool WasNotPredicted { get; set; }

	public string Genus { get; set; }

	public string Species { get; set; }

	public GenusType GenusType { get; set; }

	public SpeciesType SpeciesType { get; set; }

	public BioSeen Seen { get; set; }

	public BioAnalysed Samples { get; set; }

	public VariantType Type { get; set; }

	public List<VariantResult> PossibleVariants { get; set; }

	public int BaseValue { get; set; }

	public int Distance { get; set; }
}
public class VariantResult {public VariantColour Colour{get;set;} public string Reason{get;set;}="source reason";public bool Likely{get;set;} public bool PersonalCodexEntry{get;set;} public bool CCMark{get;set;} public bool PossibleCodexEntryNeverSeen{get;set;} public bool NotPredicted{get;set;} public bool ShowInUI{get;set;} public bool Unlikely{get;set;} public bool Seen{get;set;} public bool ColourIsMissing{get;set;} public bool PossibleCodexEntry{get;set;} public bool NotPresent{get;set;} }
public enum GenusType {Bacterium}
public enum SpeciesType {Alcyoneum}
class UIHandler {
 string BaseURL="http://localhost/bioinsights";
 string GetSymbol(OutputSymbol symbol)=>"["+symbol+"]";
public void GenerateHTMLBodyDetails(VariantOutputLine v, string rowclass, StringBuilder ln)
	{
		_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, "<tr><td>&nbsp;&nbsp;</td>");
		StringBuilder stringBuilder = ln;
		int num15 = default;
		VariantResult current = default;
		while (true)
		{
			int num = 1371951585;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0xDA3DDEF)) % 35)
				{
				case 26u:
					break;
				case 20u:
				{
					stringBuilder = ln;
					StringBuilder stringBuilder5 = stringBuilder;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(80, 6, stringBuilder);
					appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
					appendInterpolatedStringHandler.AppendFormatted(rowclass);
					appendInterpolatedStringHandler.AppendLiteral(" noleftmargin\"><a class=\"species ");
					appendInterpolatedStringHandler.AppendFormatted(rowclass);
					appendInterpolatedStringHandler.AppendLiteral("\" href=\"");
					appendInterpolatedStringHandler.AppendFormatted(BaseURL);
					appendInterpolatedStringHandler.AppendLiteral("/codexcompanion/");
					appendInterpolatedStringHandler.AppendFormatted((int)v.GenusType);
					appendInterpolatedStringHandler.AppendLiteral("/");
					appendInterpolatedStringHandler.AppendFormatted((int)v.SpeciesType);
					appendInterpolatedStringHandler.AppendLiteral("\">");
					appendInterpolatedStringHandler.AppendFormatted(v.Species);
					appendInterpolatedStringHandler.AppendLiteral("</a></td>");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder5, ref appendInterpolatedStringHandler);
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(new string[5]
					{
						"<td class=\"",
						rowclass,
						"\">&nbsp;",
						(v.Seen == BioSeen.Seen) ? GetSymbol(OutputSymbol.Tick) : ((v.Seen == BioSeen.NotPresent) ? GetSymbol(OutputSymbol.Cross) : (v.IsInCannon ? GetSymbol(OutputSymbol.Canonn) : "&nbsp;")),
						"</td>"
					}));
					stringBuilder = ln;
					num = 354318437;
					continue;
				}
				case 23u:
				{
					int num24;
					int num25;
					if (v.Type == VariantType.SurfaceMaterial)
					{
						num24 = 621967049;
						num25 = num24;
					}
					else
					{
						num24 = 1377157853;
						num25 = num24;
					}
					num = num24 ^ (int)(num2 * 1294490659);
					continue;
				}
				case 28u:
				{
					int num31;
					int num32;
					if (v.Type != VariantType.NotPredicted)
					{
						num31 = 204406694;
						num32 = num31;
					}
					else
					{
						num31 = 1364753125;
						num32 = num31;
					}
					num = num31 ^ ((int)num2 * -1391419785);
					continue;
				}
				case 22u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(new string[5]
					{
						"<td class=\"",
						rowclass,
						" norightmargin\">",
						GetSymbol(OutputSymbol.Material),
						"</td>"
					}));
					num = 89385557;
					continue;
				case 4u:
				{
					int num39;
					if (v.IsGalacticCodexEntryNeverSeen)
					{
						num = 1077128014;
						num39 = num;
					}
					else
					{
						num = 123280955;
						num39 = num;
					}
					continue;
				}
				case 13u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.PossibleCodexEntry));
					num = ((int)num2 * -479384508) ^ -2055617951;
					continue;
				case 3u:
					stringBuilder = ln;
					num = 1546029359;
					continue;
				case 33u:
					num15 = 0;
					num = 1749893972;
					continue;
				case 31u:
				{
					StringBuilder stringBuilder6 = stringBuilder;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(18, 2, stringBuilder);
					appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
					appendInterpolatedStringHandler.AppendFormatted(rowclass);
					appendInterpolatedStringHandler.AppendLiteral("\">");
					appendInterpolatedStringHandler.AppendFormatted(BioAnalysedString(v.Samples));
					appendInterpolatedStringHandler.AppendLiteral("</td>");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder6, ref appendInterpolatedStringHandler);
					num = 1017687153;
					continue;
				}
				case 8u:
				{
					int num42;
					int num43;
					if (!v.IsPersonalCodexEntry)
					{
						num42 = 1119308966;
						num43 = num42;
					}
					else
					{
						num42 = 206824321;
						num43 = num42;
					}
					num = num42 ^ ((int)num2 * -1895444485);
					continue;
				}
				case 32u:
				{
					int num38;
					if (v.IsGalacticCodexEntry)
					{
						num = 893841716;
						num38 = num;
					}
					else
					{
						num = 1336757325;
						num38 = num;
					}
					continue;
				}
				case 34u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.PersonalCodexEntry));
					num = (int)((num2 * 541828257) ^ 0x71746DA8);
					continue;
				case 29u:
				{
					int num33;
					int num34;
					if (v.WasNotPredicted)
					{
						num33 = 205432050;
						num34 = num33;
					}
					else
					{
						num33 = 622330030;
						num34 = num33;
					}
					num = num33 ^ ((int)num2 * -1416331971);
					continue;
				}
				case 14u:
				{
					int num27;
					int num28;
					if (v.Type != VariantType.Horizons)
					{
						num27 = 1593732581;
						num28 = num27;
					}
					else
					{
						num27 = 928027453;
						num28 = num27;
					}
					num = num27 ^ (int)(num2 * 828285885);
					continue;
				}
				case 11u:
				{
					int num40;
					int num41;
					if (v.Type == VariantType.ParentStar)
					{
						num40 = 391616087;
						num41 = num40;
					}
					else
					{
						num40 = 2041188184;
						num41 = num40;
					}
					num = num40 ^ (int)(num2 * 1924553644);
					continue;
				}
				case 15u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(new string[5]
					{
						"<td class=\"",
						rowclass,
						" norightmargin\">",
						GetSymbol(OutputSymbol.NotPredicted),
						"</td>"
					}));
					num = 1862054311;
					continue;
				case 25u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.PossibleCodexEntryNeverSeen));
					num = ((int)num2 * -411482889) ^ -1051495142;
					continue;
				case 12u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(new string[5]
					{
						"<td class=\"",
						rowclass,
						" norightmargin\">",
						GetSymbol(OutputSymbol.Horizons),
						"</td>"
					}));
					num = 83995736;
					continue;
				case 17u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(v.Genus, "</td>"));
					num = 987978496;
					continue;
				case 16u:
				{
					stringBuilder = ln;
					StringBuilder stringBuilder9 = stringBuilder;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(23, 1, stringBuilder);
					appendInterpolatedStringHandler.AppendLiteral("<td colspan=2 class=\"");
					appendInterpolatedStringHandler.AppendFormatted(rowclass);
					appendInterpolatedStringHandler.AppendLiteral("\">");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder9, ref appendInterpolatedStringHandler);
					num = 1872178826;
					continue;
				}
				case 19u:
					num = ((int)num2 * -924690399) ^ 0x4E4F54EE;
					continue;
				case 2u:
				{
					int num37;
					if (v.IsMarked)
					{
						num = 1351386849;
						num37 = num;
					}
					else
					{
						num = 1524228308;
						num37 = num;
					}
					continue;
				}
				case 10u:
				{
					int num36;
					if (v.Seen != BioSeen.NotPresent)
					{
						num = 293051246;
						num36 = num;
					}
					else
					{
						num = 1527012824;
						num36 = num;
					}
					continue;
				}
				case 0u:
				{
					StringBuilder stringBuilder8 = stringBuilder;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(38, 1, stringBuilder);
					appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
					appendInterpolatedStringHandler.AppendFormatted(rowclass);
					appendInterpolatedStringHandler.AppendLiteral(" norightmargin\">&nbsp;</td>");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder8, ref appendInterpolatedStringHandler);
					num = 1914456596;
					continue;
				}
				case 5u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.Mark));
					num = ((int)num2 * -916595905) ^ -646217050;
					continue;
				case 7u:
				{
					StringBuilder stringBuilder7 = stringBuilder;
					StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(13, 1, stringBuilder);
					appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
					appendInterpolatedStringHandler.AppendFormatted(rowclass);
					appendInterpolatedStringHandler.AppendLiteral("\">");
					_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder7, ref appendInterpolatedStringHandler);
					num = 670673376;
					continue;
				}
				case 21u:
				{
					int num35;
					if (v.Seen == BioSeen.NotPresent)
					{
						num = 1126402405;
						num35 = num;
					}
					else
					{
						num = 861789434;
						num35 = num;
					}
					continue;
				}
				case 27u:
				{
					int num29;
					int num30;
					if (v.Seen != BioSeen.NotPresent)
					{
						num29 = -896890218;
						num30 = num29;
					}
					else
					{
						num29 = -1123492368;
						num30 = num29;
					}
					num = num29 ^ (int)(num2 * 1376889004);
					continue;
				}
				case 9u:
					num = (int)(num2 * 1410285158) ^ -563421532;
					continue;
				case 6u:
				{
					int num26;
					if (v.Seen != BioSeen.NotPresent)
					{
						num = 690700812;
						num26 = num;
					}
					else
					{
						num = 143569620;
						num26 = num;
					}
					continue;
				}
				case 1u:
					num = (int)((num2 * 685495286) ^ 0x7D7C50CE);
					continue;
				case 30u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, "NOT PREDICTED: ");
					num = 1271025855;
					continue;
				case 18u:
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(new string[5]
					{
						"<td class=\"",
						rowclass,
						" norightmargin\">",
						GetSymbol(OutputSymbol.StarType),
						"</td>"
					}));
					num = 1914456596;
					continue;
				default:
				{
					using (List<VariantResult>.Enumerator enumerator = v.PossibleVariants.GetEnumerator())
					{
						while (true)
						{
							IL_0c71:
							int num3;
							int num4;
							if (enumerator.MoveNext())
							{
								num3 = 1124797712;
								num4 = num3;
							}
							else
							{
								num3 = 643771932;
								num4 = num3;
							}
							while (true)
							{
								switch ((num2 = (uint)(num3 ^ 0xDA3DDEF)) % 32)
								{
								case 18u:
									num3 = 1124797712;
									continue;
								default:
									goto end_IL_07c9;
								case 22u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.PossibleCodexEntryNeverSeen));
									num3 = (int)((num2 * 1748198214) ^ 0x60046466);
									continue;
								case 10u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, " or ");
									num3 = 175658961;
									continue;
								case 4u:
								{
									StringBuilder stringBuilder2 = stringBuilder;
									StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(23, 3, stringBuilder);
									appendInterpolatedStringHandler.AppendLiteral("<span title=\"");
									appendInterpolatedStringHandler.AppendFormatted(_206A_202C_206C_206B_200E_206A_202D_200C_202E_206B_206A_206C_206C_202A_202E_206A_202A_200E_206E_200C_202B_200C_202A_206A_206C_202C_202D_206A_206A_200F_206B_202E_206D_200D_200C_206C_200F_202D_200E_206D_202E(current));
									appendInterpolatedStringHandler.AppendLiteral("\"");
									appendInterpolatedStringHandler.AppendFormatted(_206B_206D_206F_202C_202A_202B_206B_206F_200B_202A_200E_206B_206C_200D_202E_200B_202E_200E_206A_200B_206C_206B_206E_202A_200C_206A_206C_200B_200B_200F_202E_206D_200B_200B_202B_206B_202E_200C_206A_202C_202E(current) ? "class=\"NotPresentItalic\"" : "");
									appendInterpolatedStringHandler.AppendLiteral("\">");
									appendInterpolatedStringHandler.AppendFormatted(_206A_202E_200F_200D_200B_200F_206E_200C_202A_200F_200B_202E_206C_206F_206C_200E_200B_206C_200C_202C_206B_202E_202C_206A_206A_200C_200C_206A_206E_200F_202E_206A_202C_206C_206E_200C_202A_206F_200F_202E_202E(current));
									appendInterpolatedStringHandler.AppendLiteral("</span>");
									_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder2, ref appendInterpolatedStringHandler);
									num15++;
									num3 = 615401998;
									continue;
								}
								case 8u:
								{
									int num22;
									if (!_202C_200E_206A_200D_202B_200E_202A_200B_202E_202B_200C_206A_200C_206C_200E_206A_202D_200D_200E_202E_202A_200C_200C_202B_206A_200C_200F_202D_202B_202C_200B_206C_206B_206F_206D_200E_200C_206A_200B_206C_202E(current))
									{
										num3 = 115935775;
										num22 = num3;
									}
									else
									{
										num3 = 17531626;
										num22 = num3;
									}
									continue;
								}
								case 6u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.Mark));
									num3 = (int)((num2 * 1034530671) ^ 0xAC8EA8F);
									continue;
								case 16u:
								{
									int num19;
									if (_202B_200E_200D_200C_206C_202B_206F_202E_202B_206B_200B_200C_200B_200B_202C_206A_206A_206F_206F_206E_200E_202B_206D_206F_200D_206D_202D_202D_200D_200F_202D_200C_202E_200E_202D_200E_202D_200E_206F_202B_202E(current))
									{
										num3 = 514001289;
										num19 = num3;
									}
									else
									{
										num3 = 1496004789;
										num19 = num3;
									}
									continue;
								}
								case 3u:
								{
									int num7;
									int num8;
									if (v.Type == VariantType.SurfaceMaterial)
									{
										num7 = -273171566;
										num8 = num7;
									}
									else
									{
										num7 = -58777965;
										num8 = num7;
									}
									num3 = num7 ^ (int)(num2 * 175775069);
									continue;
								}
								case 0u:
									num3 = (int)((num2 * 99212180) ^ 0x49CAB307);
									continue;
								case 26u:
								{
									int num10;
									if (!_200E_200E_200C_206E_200C_202A_200C_200B_206A_200D_206B_202B_202C_200B_202B_202C_206D_200D_206B_202D_200F_206C_200C_200E_206B_206F_202A_202C_202E_200F_200D_206A_206E_200C_206C_202D_200E_200D_200B_202E_202E(current))
									{
										num3 = 1827190339;
										num10 = num3;
									}
									else
									{
										num3 = 640945177;
										num10 = num3;
									}
									continue;
								}
								case 24u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.PossibleCodexEntry));
									num3 = ((int)num2 * -2001765529) ^ 0x65AE4FFA;
									continue;
								case 7u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.Tick));
									num3 = (int)((num2 * 1388501037) ^ 0x48C5F7C);
									continue;
								case 15u:
								{
									int num16;
									int num17;
									if (num15 <= 0)
									{
										num16 = 357869556;
										num17 = num16;
									}
									else
									{
										num16 = 1040357631;
										num17 = num16;
									}
									num3 = num16 ^ ((int)num2 * -341727509);
									continue;
								}
								case 9u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.Cross));
									num3 = (int)((num2 * 824849279) ^ 0x7793B210);
									continue;
								case 31u:
									current = enumerator.Current;
									num3 = 916527681;
									continue;
								case 29u:
								{
									int num11;
									if (_206B_202C_206D_200D_202A_200B_206D_206C_206F_206A_200B_200B_206B_202E_202A_200C_206F_206D_202C_200D_206B_206F_200C_200B_202A_202D_200E_206E_206C_206B_206B_206D_206A_206D_202E_206E_206B_202D_202A_202E_202E(rowclass, "NotPresentItalic"))
									{
										num3 = 1791136884;
										num11 = num3;
									}
									else
									{
										num3 = 1144259923;
										num11 = num3;
									}
									continue;
								}
								case 28u:
								{
									StringBuilder stringBuilder3 = ln;
									UIHandler uIHandler = this;
									OutputSymbol sym;
									if (_202D_200D_200D_206C_200D_200C_202A_202A_202C_200C_202D_206A_206F_200F_206B_206A_202A_202B_200D_200D_202D_202B_206E_200C_206C_206E_206E_202A_202A_202E_202E_202A_206E_202C_202C_202D_200F_202C_206C_206F_202E(current))
									{
										sym = OutputSymbol.Likely;
									}
									else
									{
										sym = (_200E_200E_200F_200E_202A_200F_206D_202A_200C_200C_200F_202D_206A_202D_206B_202C_200D_206A_200C_202A_200E_206F_202C_206A_202A_206A_200C_202D_206D_202A_202D_202D_200F_202D_200C_206E_206D_202E_206C_202D_202E(current) ? OutputSymbol.Unlikely : OutputSymbol.Maybe);
									}
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(stringBuilder3, uIHandler.GetSymbol(sym));
									num3 = 1791136884;
									continue;
								}
								case 11u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, ", ");
									num3 = 175658961;
									continue;
								case 23u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.ColourIsMissing));
									num3 = ((int)num2 * -917560224) ^ -824482033;
									continue;
								case 21u:
								{
									int num20;
									int num21;
									if (v.PossibleVariants.Count == 2)
									{
										num20 = 1459877762;
										num21 = num20;
									}
									else
									{
										num20 = 1138584042;
										num21 = num20;
									}
									num3 = num20 ^ ((int)num2 * -894149706);
									continue;
								}
								case 20u:
								{
									int num18;
									if (_202B_200F_200F_202B_206D_206D_200D_202E_200F_206A_206D_206E_202B_202A_200E_206B_202B_206E_206B_200E_206B_200F_206B_200B_200E_200E_200B_206E_206C_202C_200D_200C_200F_202D_200F_206F_202D_206C_206B_202B_202E(current))
									{
										num3 = 1434270888;
										num18 = num3;
									}
									else
									{
										num3 = 1513378814;
										num18 = num3;
									}
									continue;
								}
								case 5u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.PersonalCodexEntry));
									num3 = (int)(num2 * 1081358937) ^ -591585118;
									continue;
								case 17u:
								{
									int num14;
									if (_202B_202C_206B_200F_206D_202C_202B_200F_200B_206D_202E_200E_206F_202E_200F_200D_200D_200E_202A_200F_206E_202A_200F_206E_202A_206A_200B_206E_206B_206C_202A_202B_206D_206B_202C_206C_206B_206B_202C_200E_202E(current))
									{
										num3 = 349920088;
										num14 = num3;
									}
									else
									{
										num3 = 137116589;
										num14 = num3;
									}
									continue;
								}
								case 14u:
								{
									int num12;
									int num13;
									if (!_200B_206F_206A_200C_202B_206E_206C_202A_206E_206A_206B_200C_200D_200D_200F_206B_200D_202E_200B_200E_206A_206D_206C_202C_206D_200F_202A_200D_202C_202A_206D_206C_202A_206F_200B_200C_206C_202B_202C_202C_202E(current))
									{
										num12 = -1328681670;
										num13 = num12;
									}
									else
									{
										num12 = -521053196;
										num13 = num12;
									}
									num3 = num12 ^ ((int)num2 * -45689354);
									continue;
								}
								case 2u:
								{
									int num9;
									if (!_206B_206D_206F_202C_202A_202B_206B_206F_200B_202A_200E_206B_206C_200D_202E_200B_202E_200E_206A_200B_206C_206B_206E_202A_200C_206A_206C_200B_200B_200F_202E_206D_200B_200B_202B_206B_202E_200C_206A_202C_202E(current))
									{
										num3 = 662953479;
										num9 = num3;
									}
									else
									{
										num3 = 2074275462;
										num9 = num3;
									}
									continue;
								}
								case 13u:
									num3 = (int)((num2 * 506277361) ^ 0x63E382CF);
									continue;
								case 12u:
								{
									int num6;
									if (_206D_206C_200C_206B_202E_202A_206A_206C_206E_206B_206E_200F_206B_206A_200E_200D_202D_202C_206B_202B_200B_200D_200B_206B_206A_202C_200E_206C_206C_200C_206E_200E_202D_206F_206D_206A_202B_206B_200E_206E_202E(current))
									{
										num3 = 1399840791;
										num6 = num3;
									}
									else
									{
										num3 = 131422514;
										num6 = num3;
									}
									continue;
								}
								case 27u:
									stringBuilder = ln;
									num3 = 1700293131;
									continue;
								case 30u:
								{
									int num5;
									if (!_206A_202C_202E_206A_200C_202A_206F_206B_202E_200C_202A_206E_200E_202A_206B_202A_200F_200E_206E_200D_206D_200D_206B_200E_200F_202A_200F_200E_202A_206B_206A_200E_202B_202D_206C_202E_206D_200D_200F_200E_202E(current))
									{
										num3 = 2111393307;
										num5 = num3;
									}
									else
									{
										num3 = 1953350454;
										num5 = num3;
									}
									continue;
								}
								case 25u:
									_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, GetSymbol(OutputSymbol.NotPredicted));
									num3 = ((int)num2 * -1260465070) ^ 0x7466DB99;
									continue;
								case 1u:
									break;
								case 19u:
									goto end_IL_07c9;
								}
								goto IL_0c71;
								continue;
								end_IL_07c9:
								break;
							}
							break;
						}
					}
					_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, "</td>");
					while (true)
					{
						int num23 = 586351000;
						while (true)
						{
							switch ((num2 = (uint)(num23 ^ 0xDA3DDEF)) % 4)
							{
							case 2u:
								break;
							default:
								return;
							case 3u:
							{
								stringBuilder = ln;
								StringBuilder stringBuilder4 = stringBuilder;
								StringBuilder.AppendInterpolatedStringHandler appendInterpolatedStringHandler = new StringBuilder.AppendInterpolatedStringHandler(47, 4, stringBuilder);
								appendInterpolatedStringHandler.AppendLiteral("<td class=\"");
								appendInterpolatedStringHandler.AppendFormatted(rowclass);
								appendInterpolatedStringHandler.AppendLiteral("\">");
								appendInterpolatedStringHandler.AppendFormatted((v.BaseValue == 0) ? "" : _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(FormatMillions(v.BaseValue), "M"));
								appendInterpolatedStringHandler.AppendLiteral("</td>");
								appendInterpolatedStringHandler.AppendLiteral("<td colspan=2 class=\"");
								appendInterpolatedStringHandler.AppendFormatted(rowclass);
								appendInterpolatedStringHandler.AppendLiteral("\">");
								appendInterpolatedStringHandler.AppendFormatted((v.Distance == 0) ? "" : v.Distance.ToString());
								appendInterpolatedStringHandler.AppendLiteral("m</td>");
								_200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(stringBuilder4, ref appendInterpolatedStringHandler);
								num23 = 1911597455;
								continue;
							}
							case 0u:
								_206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(ln, "</tr>");
								num23 = 564293254;
								continue;
							case 1u:
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

	public string FormatMillions(double val)
	{
		string text = _200C_200C_206C_206F_200E_202E_200C_206F_202E_206B_202D_200E_202E_200E_206C_206A_206C_206C_200F_202E_200F_206B_206D_200F_200E_200C_200F_206F_206B_200D_200F_206B_206A_202D_206D_200C_200E_200F_200F_202C_202E(val / 1000000.0, 2).ToString();
		_200B_206B_202D_200E_206E_206E_200D_200B_206C_206A_202A_202E_206B_206B_202D_200F_202D_200B_202A_200B_202E_206D_206A_206D_206E_202C_202A_200C_202E_200B_200F_206E_200D_202B_202B_200D_200F_200F_206D_200E_202E(_200B_206B_202D_200E_206E_206E_200D_200B_206C_206A_202A_202E_206B_206B_202D_200F_202D_200B_202A_200B_202E_206D_206A_206D_206E_202C_202A_200C_202E_200B_200F_206E_200D_202B_202B_200D_200F_200F_206D_200E_202E(_200B_206B_202D_200E_206E_206E_200D_200B_206C_206A_202A_202E_206B_206B_202D_200F_202D_200B_202A_200B_202E_206D_206A_206D_206E_202C_202A_200C_202E_200B_200F_206E_200D_202B_202B_200D_200F_200F_206D_200E_202E(text, '0'), '0'), '.');
		return text;
	}

	private string BioAnalysedString(BioAnalysed b)
	{
		string result = "";
		int num;
		switch (b)
		{
		default:
			num = 85688602;
			goto IL_0023;
		case BioAnalysed.Complete:
			goto IL_0054;
		case BioAnalysed.Sampled:
			goto IL_0064;
		case BioAnalysed.Incomplete:
			goto IL_0074;
		case BioAnalysed.Logged:
			goto IL_0093;
			IL_0023:
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0x60C41E79)) % 8)
				{
				case 4u:
					break;
				case 5u:
					goto IL_0054;
				case 6u:
					goto IL_0064;
				case 0u:
					goto IL_0074;
				case 3u:
					num = (int)((num2 * 627741962) ^ 0x245E6CD5);
					continue;
				case 1u:
					goto IL_0093;
				case 7u:
					num = (int)((num2 * 2115149904) ^ 0x43E98D3B);
					continue;
				default:
					return result;
				}
				break;
			}
			goto default;
			IL_0093:
			result = GetSymbol(OutputSymbol.BiologicalLogged);
			num = 1643748558;
			goto IL_0023;
			IL_0074:
			result = GetSymbol(OutputSymbol.Aborted);
			num = 1411429387;
			goto IL_0023;
			IL_0064:
			result = GetSymbol(OutputSymbol.BiologicalSampled);
			num = 1411429387;
			goto IL_0023;
			IL_0054:
			result = GetSymbol(OutputSymbol.BiologicalComplete);
			num = 1411429387;
			goto IL_0023;
		}
	}

	

	static bool _206A_202C_202E_206A_200C_202A_206F_206B_202E_200C_202A_206E_200E_202A_206B_202A_200F_200E_206E_200D_206D_200D_206B_200E_200F_202A_200F_200E_202A_206B_206A_200E_202B_202D_206C_202E_206D_200D_200F_200E_202E(VariantResult P_0)
	{
		return P_0.NotPredicted;
	}

	static bool _202D_200D_200D_206C_200D_200C_202A_202A_202C_200C_202D_206A_206F_200F_206B_206A_202A_202B_200D_200D_202D_202B_206E_200C_206C_206E_206E_202A_202A_202E_202E_202A_206E_202C_202C_202D_200F_202C_206C_206F_202E(VariantResult P_0)
	{
		return P_0.Likely;
	}

	static bool _202B_202C_206B_200F_206D_202C_202B_200F_200B_206D_202E_200E_206F_202E_200F_200D_200D_200E_202A_200F_206E_202A_200F_206E_202A_206A_200B_206E_206B_206C_202A_202B_206D_206B_202C_206C_206B_206B_202C_200E_202E(VariantResult P_0)
	{
		return P_0.ColourIsMissing;
	}

	static VariantColour _206A_202E_200F_200D_200B_200F_206E_200C_202A_200F_200B_202E_206C_206F_206C_200E_200B_206C_200C_202C_206B_202E_202C_206A_206A_200C_200C_206A_206E_200F_202E_206A_202C_206C_206E_200C_202A_206F_200F_202E_202E(VariantResult P_0)
	{
		return P_0.Colour;
	}

	static StringBuilder _206D_206F_206D_206A_200F_206C_206C_206D_206C_202C_206B_206D_206C_206D_206B_206C_200E_206D_206B_202B_202C_206B_200B_202C_206A_202A_206D_202E_202C_202D_200B_206E_202C_206C_206C_206D_206E_206C_200C_202E_202E(StringBuilder P_0, string P_1)
	{
		return P_0.Append(P_1);
	}

	static bool _206B_206D_206F_202C_202A_202B_206B_206F_200B_202A_200E_206B_206C_200D_202E_200B_202E_200E_206A_200B_206C_206B_206E_202A_200C_206A_206C_200B_200B_200F_202E_206D_200B_200B_202B_206B_202E_200C_206A_202C_202E(VariantResult P_0)
	{
		return P_0.NotPresent;
	}

	static bool _206B_202C_206D_200D_202A_200B_206D_206C_206F_206A_200B_200B_206B_202E_202A_200C_206F_206D_202C_200D_206B_206F_200C_200B_202A_202D_200E_206E_206C_206B_206B_206D_206A_206D_202E_206E_206B_202D_202A_202E_202E(string P_0, string P_1)
	{
		return P_0.Contains(P_1);
	}

	static double _200C_200C_206C_206F_200E_202E_200C_206F_202E_206B_202D_200E_202E_200E_206C_206A_206C_206C_200F_202E_200F_206B_206D_200F_200E_200C_200F_206F_206B_200D_200F_206B_206A_202D_206D_200C_200E_200F_200F_202C_202E(double P_0, int P_1)
	{
		return Math.Round(P_0, P_1);
	}

	static StringBuilder _200B_206E_206C_206F_200E_206B_202E_202C_200F_202D_206B_206F_200B_200D_206A_202D_206A_200B_206E_202D_200E_206B_202C_202A_200D_200C_206D_202D_202D_202D_206F_202A_206F_206B_206B_202A_202D_206C_200E_206F_202E(StringBuilder P_0, ref StringBuilder.AppendInterpolatedStringHandler P_1)
	{
		return P_0.Append(ref P_1);
	}

	static bool _200E_200E_200F_200E_202A_200F_206D_202A_200C_200C_200F_202D_206A_202D_206B_202C_200D_206A_200C_202A_200E_206F_202C_206A_202A_206A_200C_202D_206D_202A_202D_202D_200F_202D_200C_206E_206D_202E_206C_202D_202E(VariantResult P_0)
	{
		return P_0.Unlikely;
	}

	static bool _202B_200E_200D_200C_206C_202B_206F_202E_202B_206B_200B_200C_200B_200B_202C_206A_206A_206F_206F_206E_200E_202B_206D_206F_200D_206D_202D_202D_200D_200F_202D_200C_202E_200E_202D_200E_202D_200E_206F_202B_202E(VariantResult P_0)
	{
		return P_0.CCMark;
	}

	static string _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(string[] P_0)
	{
		return string.Concat(P_0);
	}

	static string _206A_202C_206C_206B_200E_206A_202D_200C_202E_206B_206A_206C_206C_202A_202E_206A_202A_200E_206E_200C_202B_200C_202A_206A_206C_202C_202D_206A_206A_200F_206B_202E_206D_200D_200C_206C_200F_202D_200E_206D_202E(VariantResult P_0)
	{
		return P_0.Reason;
	}

	static bool _206D_206C_200C_206B_202E_202A_206A_206C_206E_206B_206E_200F_206B_206A_200E_200D_202D_202C_206B_202B_200B_200D_200B_206B_206A_202C_200E_206C_206C_200C_206E_200E_202D_206F_206D_206A_202B_206B_200E_206E_202E(VariantResult P_0)
	{
		return P_0.PossibleCodexEntry;
	}

	static string _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(string P_0, string P_1)
	{
		return P_0 + P_1;
	}

	static bool _202C_200E_206A_200D_202B_200E_202A_200B_202E_202B_200C_206A_200C_206C_200E_206A_202D_200D_200E_202E_202A_200C_200C_202B_206A_200C_200F_202D_202B_202C_200B_206C_206B_206F_206D_200E_200C_206A_200B_206C_202E(VariantResult P_0)
	{
		return P_0.PersonalCodexEntry;
	}

	static bool _200E_200E_200C_206E_200C_202A_200C_200B_206A_200D_206B_202B_202C_200B_202B_202C_206D_200D_206B_202D_200F_206C_200C_200E_206B_206F_202A_202C_202E_200F_200D_206A_206E_200C_206C_202D_200E_200D_200B_202E_202E(VariantResult P_0)
	{
		return P_0.PossibleCodexEntryNeverSeen;
	}

	static bool _202B_200F_200F_202B_206D_206D_200D_202E_200F_206A_206D_206E_202B_202A_200E_206B_202B_206E_206B_200E_206B_200F_206B_200B_200E_200E_200B_206E_206C_202C_200D_200C_200F_202D_200F_206F_202D_206C_206B_202B_202E(VariantResult P_0)
	{
		return P_0.Seen;
	}

	static string _200B_206B_202D_200E_206E_206E_200D_200B_206C_206A_202A_202E_206B_206B_202D_200F_202D_200B_202A_200B_202E_206D_206A_206D_206E_202C_202A_200C_202E_200B_200F_206E_200D_202B_202B_200D_200F_200F_206D_200E_202E(string P_0, char P_1)
	{
		return P_0.TrimEnd(P_1);
	}

	static bool _200B_206F_206A_200C_202B_206E_206C_202A_206E_206A_206B_200C_200D_200D_200F_206B_200D_202E_200B_200E_206A_206D_206C_202C_206D_200F_202A_200D_202C_202A_206D_206C_202A_206F_200B_200C_206C_202B_202C_202C_202E(VariantResult P_0)
	{
		return P_0.ShowInUI;
	}
 static void Main(){var results=new List<object>();foreach(VariantType type in Enum.GetValues<VariantType>())foreach(BioSeen seen in Enum.GetValues<BioSeen>())foreach(BioAnalysed samples in Enum.GetValues<BioAnalysed>()){
 var v=new VariantOutputLine{Genus="Bacterium",Species="Alcyoneum",Type=type,Seen=seen,Samples=samples,BaseValue=1658500,Distance=500,IsPersonalCodexEntry=true,PossibleVariants=new(){new(){Colour=VariantColour.Emerald,ShowInUI=true,Likely=true,PersonalCodexEntry=true},new(){Colour=VariantColour.Red,ShowInUI=true,Unlikely=true,CCMark=true}}};
 var o=new UIHandler();var s=new StringBuilder();o.GenerateHTMLBodyDetails(v,"row1",s);results.Add(new{type=type.ToString(),seen=seen.ToString(),samples=samples.ToString(),html=s.ToString()});}Console.WriteLine(JsonSerializer.Serialize(results));}
}
