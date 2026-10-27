using System;using System.Text;using System.Linq;using System.Collections.Generic;using System.Runtime.CompilerServices;using System.Text.Json;using System.Globalization;using System.Text.RegularExpressions;
class Requirement { public List<List<RequirementValue>> Value { get; set; }=new(); }
class RequirementValue { public RequirementValueType Type {get;set;} public string? SValue{get;set;} public double? DValue{get;set;} public int? IValue{get;set;} }
class Utils{public static string[] SplitCamelCase(string source)=>Regex.Split(source,"(?<!^)(?=[A-Z])");}
public enum RequirementValueType
{
	ATMOSPHERE_DENSITY,
	ATMOSPHERE_TYPE,
	ATMOSPHERE_COMPOSITION,
	PARENT_TYPE,
	PARENT_MIN_DISTANCE_AU,
	VOLCANISM,
	MATERIAL,
	PRESSURE_MIN,
	PRESSURE_MAX,
	GRAVITY_MIN,
	GRAVITY_MAX,
	TEMPERATURE_MIN,
	TEMPERATURE_MAX,
	MASS_MIN,
	MASS_MAX,
	PLANET_TYPE,
	IS_IN_SPHERE,
	SYSTEM_HAS_BODY,
	SYSTEM_HAS_STAR,
	SMA_MAX,
	SMA_MIN,
	DISTANCE_LS_FROM_START_MIN,
	DISTANCE_LS_FROM_START_MAX,
	APPARENT_BOLOMETRIC_FLUX_MIN,
	APPARENT_BOLOMETRIC_FLUX_MAX,
	APPARENT_BOLOMETRIC_MAGNITUDE_MIN,
	APPARENT_BOLOMETRIC_MAGNITUDE_MAX,
	STAR_LUMINOSITY,
	IS_PG,
	RADIANT_SMA_MAX,
	RADIANT_SMA_MIN,
	NEAREST_STAR_SMA_MAX,
	NEAREST_STAR_SMA_MIN,
	NOT_ATMOSPHERE_TYPE,
	APPARENT_MAGNITUDE_MIN,
	APPARENT_MAGNITUDE_MAX,
	ABSOLUTE_MAGNITUDE_MIN,
	ABSOLUTE_MAGNITUDE_MAX,
	APPARENT_CALCULATED_MAGNITUDE_MIN,
	APPARENT_CALCULATED_MAGNITUDE_MAX,
	RADIANT_STAR,
	BLACKBODY_TEMP_MIN,
	BLACKBODY_TEMP_MAX,
	IN_REGION
}

public enum AtmosphereComposition
{
	Water = 1,
	Oxygen = 2,
	CarbonDioxide = 4,
	SulphurDioxide = 8,
	Ammonia = 0x10,
	Methane = 0x20,
	Nitrogen = 0x40,
	Hydrogen = 0x80,
	Helium = 0x100,
	Neon = 0x200,
	Argon = 0x400,
	Silicates = 0x800,
	Iron = 0x1000
}

public enum AtmosphereType
{
	Ammonia,
	AmmoniaOxygen,
	AmmoniaRich,
	Argon,
	ArgonRich,
	CarbonDioxide,
	CarbonDioxideRich,
	EarthLike,
	Helium,
	MetallicVapour,
	Methane,
	MethaneRich,
	Neon,
	NeonRich,
	Nitrogen,
	None,
	Oxygen,
	SilicateVapour,
	SulphurDioxide,
	Water,
	WaterRich
}

public enum VariantStar
{
	UNKNOWN,
	O,
	B,
	A,
	F,
	G,
	K,
	M,
	L,
	T,
	TTS,
	W,
	D,
	N,
	Y,
	AEBE,
	H,
	SUPER,
	C
}

public enum POISphereType
{
	Nebula,
	PlanetaryNebula,
	BrainTrees,
	LindigoticumBrainTrees,
	OstrinumBrainTrees,
	PuniceumBrainTrees,
	SinuousTubers
}

public enum PlanetClass
{
	Unknown,
	Ammonia_world,
	Earthlike_body,
	Gas_giant_with_ammonia_based_life,
	Gas_giant_with_water_based_life,
	Helium_rich_gas_giant,
	High_metal_content_body,
	Icy_body,
	Metal_rich_body,
	Rocky_body,
	Rocky_ice_body,
	Sudarsky_class_I_gas_giant,
	Sudarsky_class_II_gas_giant,
	Sudarsky_class_III_gas_giant,
	Sudarsky_class_IV_gas_giant,
	Sudarsky_class_V_gas_giant,
	Water_giant,
	Water_world,
	Helium_gas_giant,
	Water_giant_with_life
}

class Formatter {
private string FirstLetterToUpper(string str) => str==null?null:(str.Length>1?char.ToUpper(str[0])+str.Substring(1):str.ToUpper());
private string RequirementAsString(Requirement r)
	{
string text2 = default;
RequirementValue current2 = default;
		string text = "";
		using (List<List<RequirementValue>>.Enumerator enumerator = _206C_206B_202A_202E_202D_200D_206C_200C_202D_206D_200D_206A_200B_206D_206D_206C_202D_202C_206B_206E_202D_206B_206C_206A_202D_206F_206A_206D_202C_202B_200C_206C_202E_206A_206D_202B_202D_206B_206F_206C_202E(r).GetEnumerator())
		{


			while (true)
			{
				List<RequirementValue> current;
				uint num2;
				if (enumerator.MoveNext())
				{
					while (true)
					{
						IL_015e:
						current = enumerator.Current;
						int num = -1647621930;
						while (true)
						{
							switch ((num2 = (uint)(num ^ -2089183422)) % 8)
							{
							case 3u:
								num = -977364040;
								continue;
							case 0u:
								break;
							case 7u:
								goto IL_007a;
							case 6u:
								goto IL_00e6;
							case 5u:
								text2 = "";
								num = -1523990061;
								continue;
							case 4u:
								goto IL_0113;
							case 2u:
								goto IL_015e;
							default:
								goto IL_0170;
							}
							break;
							IL_0113:
							text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text, _202B_206A_202E_202C_202B_200B_206B_200D_200E_202B_200E_206F_206C_200C_200B_206B_206A_206F_202C_200C_206A_202B_202E_200E_200B_202B_206A_200C_206E_200C_206B_202A_206B_200F_206A_200C_200E_200B_206A_200F_202E(text, "") ? "&nbsp;&nbsp;&#x25CF; " : "<br />&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&#x25CB; OR ");
							int num3;
							if (current.Count == 1)
							{
								num = -1023503342;
								num3 = num;
							}
							else
							{
								num = -194758948;
								num3 = num;
							}
							continue;
							IL_00e6:
							int num4;
							if (current.Count != 2)
							{
								num = -1132605105;
								num4 = num;
							}
							else
							{
								num = -1310062123;
								num4 = num;
							}
						}
						break;
					}
					text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text, FormatReq(current[0]));
					continue;
				}
				int num5 = -1342632269;
				goto IL_024d;
				IL_0248:
				num5 = -2134112076;
				goto IL_024d;
				IL_024d:
				switch ((num2 = (uint)(num5 ^ -2089183422)) % 3)
				{
				case 0u:
					break;
				default:
					goto end_IL_026b;
				case 1u:
					continue;
				case 2u:
					goto end_IL_026b;
				}
				goto IL_0248;
				IL_0170:
				using (List<RequirementValue>.Enumerator enumerator2 = current.GetEnumerator())
				{
					while (true)
					{
						IL_01a5:
						int num6;
						int num7;
						if (enumerator2.MoveNext())
						{
							num6 = -480564804;
							num7 = num6;
						}
						else
						{
							num6 = -1591512105;
							num7 = num6;
						}
						while (true)
						{
							switch ((num2 = (uint)(num6 ^ -2089183422)) % 5)
							{
							case 3u:
								num6 = -480564804;
								continue;
							default:
								goto end_IL_017f;
							case 2u:
								break;
							case 0u:
								text2 = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text2, FormatReq(current2));
								num6 = ((int)num2 * -1919238355) ^ 0x594DFC4;
								continue;
							case 4u:
								current2 = enumerator2.Current;
								text2 = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text2, _202B_206A_202E_202C_202B_200B_206B_200D_200E_202B_200E_206F_206C_200C_200B_206B_206A_206F_202C_200C_206A_202B_202E_200E_200B_202B_206A_200C_206E_200C_206B_202A_206B_200F_206A_200C_200E_200B_206A_200F_202E(text2, "") ? "" : "<br />&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;AND ");
								num6 = -284306636;
								continue;
							case 1u:
								goto end_IL_017f;
							}
							goto IL_01a5;
							continue;
							end_IL_017f:
							break;
						}
						break;
					}
				}
				text = _206F_206D_202E_200F_206F_202E_202B_202D_206A_202E_200C_206D_206C_202A_202D_202A_206B_200F_206E_206D_202E_200B_202A_206C_202D_206D_206D_202E_200C_202E_200F_200D_200B_206C_206D_200D_202C_200B_206A_202C_202E(text, "( ", text2, " )");
				goto IL_0248;
				IL_007a:
				text = _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(new string[6]
				{
					text,
					"( ",
					FormatReq(current[0]),
					" AND ",
					FormatReq(current[1]),
					" )"
				});
				continue;
				end_IL_026b:
				break;
			}
		}
		return text;
	}
private string FormatReq(RequirementValue r)
	{
string text = default;
string[] array2 = default;
int num5 = default;
RequirementValueType requirementValueType = default;
POISphereType value = default;
		if (_200F_202D_202C_206F_200B_202B_202D_206C_206E_206F_202D_200D_200F_206B_206A_202D_200C_202B_200E_202B_206E_202E_200B_206F_202A_202A_206B_202E_200D_200F_200E_206C_200E_206B_202B_202D_202B_206C_206A_200C_202E(r) != null)
		{
			goto IL_000b;
		}
		string[] array = _200E_202C_202E_200D_206A_202D_202B_200E_202C_206F_206E_206E_200D_202C_200C_202D_206C_200F_200D_202A_206F_200C_206B_200C_202A_200C_202B_202E_202A_202E_202C_200E_202C_200F_202B_202C_202C_206F_202A_200E_202E("", '_', StringSplitOptions.None);
		goto IL_08e6;
		IL_08c4:
		array = _200E_206C_200E_200E_202D_200E_206A_202E_202A_202A_202E_202A_200C_200E_206E_202D_206B_206D_200F_200C_206D_206B_206C_206D_202A_202B_200F_206E_206E_206E_202E_200C_202D_200E_206E_200B_202D_202C_200C_202E_202E(_202D_206A_206E_206A_202B_202B_206B_206B_202E_200D_206B_202D_206A_206A_206D_202D_202C_206A_202B_202A_206B_202C_206D_206F_200F_200D_200B_206A_200C_200F_202B_206E_206E_200C_206F_200C_200B_202B_206E_200C_202E(_200F_202D_202C_206F_200B_202B_202D_206C_206E_206F_202D_200D_200F_206B_206A_202D_200C_202B_200E_202B_206E_202E_200B_206F_202A_202A_206B_202E_200D_200F_200E_206C_200E_206B_202B_202D_202B_206C_206A_200C_202E(r)), '_', StringSplitOptions.None);
		goto IL_08e6;
		IL_000b:
		int num = 1418285561;
		goto IL_0010;
		IL_0010:





		while (true)
		{
			uint num2;
			string text2;
			string text4;
			string text6;
			string text7;
			string text8;
			string text9;
			string text10;
			string text11;
			string text12;
			string text13;
			string text15;
			DefaultInterpolatedStringHandler defaultInterpolatedStringHandler;
			switch ((num2 = (uint)(num ^ 0x1D995AEF)) % 49)
			{
			case 45u:
				break;
			case 1u:
			{
				string text14 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(17, 1);
				defaultInterpolatedStringHandler.AppendLiteral("Temperature: >= ");
				defaultInterpolatedStringHandler.AppendFormatted(_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).Value);
				defaultInterpolatedStringHandler.AppendLiteral("K");
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text14, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 2132762306;
				continue;
			}
			case 18u:
				num = ((int)num2 * -260452559) ^ 0x19CCA7AF;
				continue;
			case 48u:
				num = ((int)num2 * -912343014) ^ -1684673658;
				continue;
			case 15u:
				array2[num5] = FirstLetterToUpper(array2[num5]);
				num = 1210548966;
				continue;
			case 20u:
				goto IL_0185;
			case 27u:
				goto IL_01bb;
			case 35u:
				num = (int)(num2 * 789524501) ^ -812950011;
				continue;
			case 43u:
				goto IL_01f7;
			case 22u:
				goto IL_0247;
			case 23u:
				num = ((int)num2 * -2070110370) ^ 0x1098A7FA;
				continue;
			case 44u:
				num = ((int)num2 * -1368073342) ^ -649325982;
				continue;
			case 21u:
				goto IL_02cc;
			case 29u:
				text = "";
				requirementValueType = _202D_202A_200B_202C_202A_200E_202B_202C_202C_200D_200D_200C_206F_200B_200F_200C_202A_202B_202D_202D_200B_206B_200D_200F_206A_206E_202E_202A_200D_202C_206D_206A_206F_206A_202C_206C_200F_206A_206E_206B_202E(r);
				switch (requirementValueType)
				{
				case RequirementValueType.TEMPERATURE_MIN:
					break;
				case RequirementValueType.ATMOSPHERE_COMPOSITION:
					goto IL_0185;
				case RequirementValueType.PARENT_TYPE:
					goto IL_01bb;
				case RequirementValueType.SYSTEM_HAS_STAR:
					goto IL_01f7;
				case RequirementValueType.PRESSURE_MAX:
					goto IL_0247;
				case RequirementValueType.GRAVITY_MAX:
					goto IL_02cc;
				default:
					goto IL_0398;
				case RequirementValueType.SYSTEM_HAS_BODY:
					goto IL_03cb;
				case RequirementValueType.PARENT_MIN_DISTANCE_AU:
					goto IL_040f;
				case RequirementValueType.MATERIAL:
					goto IL_04a7;
				case RequirementValueType.PRESSURE_MIN:
					goto IL_04da;
				case RequirementValueType.ATMOSPHERE_DENSITY:
					goto IL_0539;
				case RequirementValueType.MASS_MIN:
				case RequirementValueType.MASS_MAX:
				case RequirementValueType.SMA_MIN:
					goto IL_056c;
				case RequirementValueType.VOLCANISM:
					goto IL_068e;
				case RequirementValueType.TEMPERATURE_MAX:
					goto IL_0791;
				case RequirementValueType.ATMOSPHERE_TYPE:
					goto IL_0807;
				case RequirementValueType.DISTANCE_LS_FROM_START_MIN:
					goto IL_0865;
				case RequirementValueType.IS_IN_SPHERE:
					goto IL_092e;
				case RequirementValueType.PLANET_TYPE:
					goto IL_0948;
				case RequirementValueType.GRAVITY_MIN:
					goto IL_09eb;
				case RequirementValueType.SMA_MAX:
					goto IL_0a67;
				}
				goto case 1u;
			case 7u:
			{
				int num8;
				int num9;
				if (value != POISphereType.Nebula)
				{
					num8 = -1209047702;
					num9 = num8;
				}
				else
				{
					num8 = -1284580784;
					num9 = num8;
				}
				num = num8 ^ ((int)num2 * -614268309);
				continue;
			}
			case 28u:
				goto IL_03cb;
			case 5u:
				goto IL_040f;
			case 39u:
				num = ((int)num2 * -461068448) ^ -101561132;
				continue;
			case 2u:
				num = ((int)num2 * -217044031) ^ 0x6DDCFB2;
				continue;
			case 3u:
				num = ((int)num2 * -408824555) ^ 0x5B2CB342;
				continue;
			case 11u:
				goto IL_04a7;
			case 13u:
				num = (int)((num2 * 627529990) ^ 0x7779A40A);
				continue;
			case 0u:
				goto IL_04da;
			case 9u:
				goto IL_0539;
			case 4u:
				num = (int)(num2 * 1095761338) ^ -442319892;
				continue;
			case 41u:
				goto IL_056c;
			case 40u:
				num = ((int)num2 * -216974596) ^ -700827188;
				continue;
			case 10u:
			{
				string text5 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(11, 1);
				defaultInterpolatedStringHandler.AppendLiteral("In: Region ");
				defaultInterpolatedStringHandler.AppendFormatted(_206D_200E_200D_206A_206E_202D_206F_202A_206F_206C_202C_202E_206C_202B_202A_200C_206E_200C_202A_200F_200C_206E_206A_202B_200C_206C_202A_202D_200B_202E_206B_200F_202C_200F_206F_202E_200B_206A_206E_202B_202E(r));
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text5, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 2028116404;
				continue;
			}
			case 31u:
				num = (int)(num2 * 1056882830) ^ -513301148;
				continue;
			case 6u:
				text = _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(text, "Near: ", _206E_202A_202C_200E_202C_202D_206B_202A_206C_200F_206B_202A_206F_206B_206B_206F_200D_200F_206D_202B_206C_202A_202C_200E_202A_200B_202C_206D_206F_202E_206B_206B_200D_206F_206A_200E_202E_206D_206A_202C_202E(" ", Utils.SplitCamelCase(value.ToString())));
				num = 1620915139;
				continue;
			case 32u:
				goto IL_068e;
			case 16u:
			{
				int num6;
				int num7;
				if (value == POISphereType.PlanetaryNebula)
				{
					num6 = -462093816;
					num7 = num6;
				}
				else
				{
					num6 = -1705863521;
					num7 = num6;
				}
				num = num6 ^ ((int)num2 * -135290030);
				continue;
			}
			case 25u:
				num = (int)((num2 * 1620346745) ^ 0x740BFFCA);
				continue;
			case 26u:
			{
				string text3 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(28, 2);
				defaultInterpolatedStringHandler.AppendLiteral("Atmosphere contains: ");
				defaultInterpolatedStringHandler.AppendFormatted(((AtmosphereComposition)_206D_200E_200D_206A_206E_202D_206F_202A_206F_206C_202C_202E_206C_202B_202A_200C_206E_200C_202A_200F_200C_206E_206A_202B_200C_206C_202A_202D_200B_202E_206B_200F_202C_200F_206F_202E_200B_206A_206E_202B_202E(r).Value/*cast due to constrained. prefix*/).ToString());
				defaultInterpolatedStringHandler.AppendLiteral(" (>= ");
				defaultInterpolatedStringHandler.AppendFormatted(_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r));
				defaultInterpolatedStringHandler.AppendLiteral(")%");
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text3, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 1833637822;
				continue;
			}
			case 8u:
				num = (int)((num2 * 1609876171) ^ 0x7CAA3B7);
				continue;
			case 19u:
				goto IL_0791;
			case 36u:
				num5++;
				num = ((int)num2 * -736221960) ^ -47338860;
				continue;
			case 30u:
				goto IL_0807;
			case 46u:
			{
				int num3;
				int num4;
				if (requirementValueType != RequirementValueType.IN_REGION)
				{
					num3 = 375219675;
					num4 = num3;
				}
				else
				{
					num3 = 1856135458;
					num4 = num3;
				}
				num = num3 ^ (int)(num2 * 1850898420);
				continue;
			}
			case 47u:
				goto IL_0865;
			case 17u:
				goto IL_08c4;
			case 38u:
				text = _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(text, "Atmosphere contains: Only ", ((AtmosphereComposition)_206D_200E_200D_206A_206E_202D_206F_202A_206F_206C_202C_202E_206C_202B_202A_200C_206E_200C_202A_200F_200C_206E_206A_202B_200C_206C_202A_202D_200B_202E_206B_200F_202C_200F_206F_202E_200B_206A_206E_202B_202E(r).Value/*cast due to constrained. prefix*/).ToString());
				num = 1304564966;
				continue;
			case 33u:
				goto IL_092e;
			case 34u:
				goto IL_0948;
			case 24u:
				text = _206F_206D_202E_200F_206F_202E_202B_202D_206A_202E_200C_206D_206C_202A_202D_202A_206B_200F_206E_206D_202E_200B_202A_206C_202D_206D_206D_202E_200C_202E_200F_200D_200B_206C_206D_200D_202C_200B_206A_202C_202E(text, "In: ", _206E_202A_202C_200E_202C_202D_206B_202A_206C_200F_206B_202A_206F_206B_206B_206F_200D_200F_206D_202B_206C_202A_202C_200E_202A_200B_202C_206D_206F_202E_206B_206B_200D_206F_206A_200E_202E_206D_206A_202C_202E(" ", Utils.SplitCamelCase(value.ToString())), " area");
				num = 2028116404;
				continue;
			case 37u:
				num = (int)(num2 * 394604459) ^ -1845506608;
				continue;
			case 14u:
				goto IL_09eb;
			case 12u:
				goto IL_0a4a;
			default:
				goto IL_0a67;
				IL_03cb:
				text = _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(text, "System Contains: ", _206F_200D_200F_202C_202A_202D_206A_200C_206B_206D_206E_202C_206E_206C_202A_200F_206A_200E_206C_206E_200B_202D_206A_202B_202B_200D_200C_200F_202E_202D_200F_206F_202C_202C_200C_200E_200D_200D_200C_206B_202E(((PlanetClass)_206D_200E_200D_206A_206E_202D_206F_202A_206F_206C_202C_202E_206C_202B_202A_200C_206E_200C_202A_200F_200C_206E_206A_202B_200C_206C_202A_202D_200B_202E_206B_200F_202C_200F_206F_202E_200B_206A_206E_202B_202E(r).Value/*cast due to constrained. prefix*/).ToString(), '_', ' '));
				num = 2102804467;
				continue;
				IL_0398:
				num = ((int)num2 * -1902012242) ^ 0x2784124D;
				continue;
				IL_040f:
				text2 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(25, 1);
				defaultInterpolatedStringHandler.AppendLiteral("Distance From Star >= ");
				defaultInterpolatedStringHandler.AppendFormatted(_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).Value);
				defaultInterpolatedStringHandler.AppendLiteral(" AU");
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text2, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 2028116404;
				continue;
				IL_02cc:
				text4 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(11, 1);
				defaultInterpolatedStringHandler.AppendLiteral("Gravity < ");
				defaultInterpolatedStringHandler.AppendFormatted(_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).Value);
				defaultInterpolatedStringHandler.AppendLiteral("G");
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text4, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 810215309;
				continue;
				IL_01f7:
				text = _206F_206D_202E_200F_206F_202E_202B_202D_206A_202E_200C_206D_206C_202A_202D_202A_206B_200F_206E_206D_202E_200B_202A_206C_202D_206D_206D_202E_200C_202E_200F_200D_200B_206C_206D_200D_202C_200B_206A_202C_202E(text, "System Contains: ", _206F_200D_200F_202C_202A_202D_206A_200C_206B_206D_206E_202C_206E_206C_202A_200F_206A_200E_206C_206E_200B_202D_206A_202B_202B_200D_200C_200F_202E_202D_200F_206F_202C_202C_200C_200E_200D_200D_200C_206B_202E(((VariantStar)_206D_200E_200D_206A_206E_202D_206F_202A_206F_206C_202C_202E_206C_202B_202A_200C_206E_200C_202A_200F_200C_206E_206A_202B_200C_206C_202A_202D_200B_202E_206B_200F_202C_200F_206F_202E_200B_206A_206E_202B_202E(r).Value/*cast due to constrained. prefix*/).ToString(), '_', ' '), " Star");
				num = 877747684;
				continue;
				IL_0247:
				text6 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(15, 1);
				defaultInterpolatedStringHandler.AppendLiteral("Pressure < ");
				defaultInterpolatedStringHandler.AppendFormatted(_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).Value);
				defaultInterpolatedStringHandler.AppendLiteral(" Atm");
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text6, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 643396969;
				continue;
				IL_0a67:
				return text;
				IL_09eb:
				text7 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(12, 1);
				defaultInterpolatedStringHandler.AppendLiteral("Gravity >= ");
				defaultInterpolatedStringHandler.AppendFormatted(_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).Value);
				defaultInterpolatedStringHandler.AppendLiteral("G");
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text7, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 581570743;
				continue;
				IL_0948:
				text = _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(text, "Planet Type: ", _206F_200D_200F_202C_202A_202D_206A_200C_206B_206D_206E_202C_206E_206C_202A_200F_206A_200E_206C_206E_200B_202D_206A_202B_202B_200D_200C_200F_202E_202D_200F_206F_202C_202C_200C_200E_200D_200D_200C_206B_202E(((PlanetClass)_206D_200E_200D_206A_206E_202D_206F_202A_206F_206C_202C_202E_206C_202B_202A_200C_206E_200C_202A_200F_200C_206E_206A_202B_200C_206C_202A_202D_200B_202E_206B_200F_202C_200F_206F_202E_200B_206A_206E_202B_202E(r).Value/*cast due to constrained. prefix*/).ToString(), '_', ' '));
				num = 691490305;
				continue;
				IL_092e:
				value = (POISphereType)_206D_200E_200D_206A_206E_202D_206F_202A_206F_206C_202C_202E_206C_202B_202A_200C_206E_200C_202A_200F_200C_206E_206A_202B_200C_206C_202A_202D_200B_202E_206B_200F_202C_200F_206F_202E_200B_206A_206E_202B_202E(r).Value;
				num = 1571953749;
				continue;
				IL_0865:
				text8 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(29, 1);
				defaultInterpolatedStringHandler.AppendLiteral("Distance From Main Star: > ");
				defaultInterpolatedStringHandler.AppendFormatted(_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).Value);
				defaultInterpolatedStringHandler.AppendLiteral("LS");
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text8, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 915224474;
				continue;
				IL_0807:
				text = _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(text, "Atmosphere: ", ((AtmosphereType)_206D_200E_200D_206A_206E_202D_206F_202A_206F_206C_202C_202E_206C_202B_202A_200C_206E_200C_202A_200F_200C_206E_206A_202B_200C_206C_202A_202D_200B_202E_206B_200F_202C_200F_206F_202E_200B_206A_206E_202B_202E(r).Value/*cast due to constrained. prefix*/).ToString());
				num = 2028116404;
				continue;
				IL_0791:
				text9 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(16, 1);
				defaultInterpolatedStringHandler.AppendLiteral("Temperature: < ");
				defaultInterpolatedStringHandler.AppendFormatted(_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).Value);
				defaultInterpolatedStringHandler.AppendLiteral("K");
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text9, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 2028116404;
				continue;
				IL_068e:
				text = _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(text, "Volcanism: ", _206E_202A_202C_200E_202C_202D_206B_202A_206C_200F_206B_202A_206F_206B_206B_206F_200D_200F_206D_202B_206C_202A_202C_200E_202A_200B_202C_206D_206F_202E_206B_206B_200D_206F_206A_200E_202E_206D_206A_202C_202E(" ", array2));
				num = 2028116404;
				continue;
				IL_056c:
				text10 = text;
				text11 = _202D_202A_200B_202C_202A_200E_202B_202C_202C_200D_200D_200C_206F_200B_200F_200C_202A_202B_202D_202D_200B_206B_200D_200F_206A_206E_202E_202A_200D_202C_206D_206A_206F_206A_202C_206C_200F_206A_206E_206B_202E(r).ToString();
				text12 = ": ";
				if (_200F_202D_202C_206F_200B_202B_202D_206C_206E_206F_202D_200D_200F_206B_206A_202D_200C_202B_200E_202B_206E_202E_200B_206F_202A_202A_206B_202E_200D_200F_200E_206C_200E_206B_202B_202D_202B_206C_206A_200C_202E(r) != null)
				{
					text13 = _200F_202D_202C_206F_200B_202B_202D_206C_206E_206F_202D_200D_200F_206B_206A_202D_200C_202B_200E_202B_206E_202E_200B_206F_202A_202A_206B_202E_200D_200F_200E_206C_200E_206B_202B_202D_202B_206C_206A_200C_202E(r);
				}
				else
				{
					text13 = (_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).HasValue ? _200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).Value.ToString() : "-");
				}
				text = _206F_206D_202E_200F_206F_202E_202B_202D_206A_202E_200C_206D_206C_202A_202D_202A_206B_200F_206E_206D_202E_200B_202A_206C_202D_206D_206D_202E_200C_202E_200F_200D_200B_206C_206D_200D_202C_200B_206A_202C_202E(text10, text11, text12, text13);
				num = 2028116404;
				continue;
				IL_01bb:
				text = _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(text, "Parent: ", StarTypeFromCode(_200F_202D_202C_206F_200B_202B_202D_206C_206E_206F_202D_200D_200F_206B_206A_202D_200C_202B_200E_202B_206E_202E_200B_206F_202A_202A_206B_202E_200D_200F_200E_206C_200E_206B_202B_202D_202B_206C_206A_200C_202E(r)));
				num = 601893956;
				continue;
				IL_0539:
				text = _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(text, "Atmosphere Density: ", array2[0]);
				num = 2028116404;
				continue;
				IL_04da:
				text15 = text;
				defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(16, 1);
				defaultInterpolatedStringHandler.AppendLiteral("Pressure >= ");
				defaultInterpolatedStringHandler.AppendFormatted(_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r).Value);
				defaultInterpolatedStringHandler.AppendLiteral(" Atm");
				text = _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(text15, defaultInterpolatedStringHandler.ToStringAndClear());
				num = 2028116404;
				continue;
				IL_04a7:
				text = _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(text, "Material: ", array2[0]);
				num = 648893041;
				continue;
			}
			break;
			IL_0a4a:
			int num10;
			if (num5 >= array2.Count())
			{
				num = 2126257668;
				num10 = num;
			}
			else
			{
				num = 1248482668;
				num10 = num;
			}
			continue;
			IL_0185:
			int num11;
			if (_200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(r) != 100.0)
			{
				num = 913343962;
				num11 = num;
			}
			else
			{
				num = 49633295;
				num11 = num;
			}
		}
		goto IL_000b;
		IL_08e6:
		array2 = array;
		num5 = 0;
		num = 905805062;
		goto IL_0010;
	}
private string StarTypeFromCode(string code)
	{
string result = default;
char c = default;
		string text = _206D_202E_206D_200E_200F_200C_202A_200C_202B_202D_206C_206B_206C_202C_206B_200C_202B_200E_202A_206C_202D_200D_206B_200F_206C_200C_202C_200D_206F_200F_202A_200D_206B_200F_200D_202B_206D_200C_202B_200F_202E(code);


		while (true)
		{
			int num = -416072566;
			while (true)
			{
				uint num2;
				int num5;
				int num16;
				switch ((num2 = (uint)(num ^ -1506491306)) % 31)
				{
				case 7u:
					break;
				case 19u:
					result = "Wolf-Rayet";
					num = -1559789912;
					continue;
				case 18u:
					c = _200C_200C_202D_206D_206D_200B_202B_206E_202B_202A_206A_206F_206D_206C_206D_206F_206D_202E_202B_202A_200B_206D_202C_200D_200C_200D_200F_200F_202D_206E_202A_202A_202C_202D_202B_202B_206F_206D_200C_202E_202E(text, 0);
					num = -494640415;
					continue;
				case 10u:
					num = (int)(num2 * 1974349291) ^ -1590945147;
					continue;
				case 21u:
					goto IL_00da;
				case 14u:
					result = "Neutron";
					num = -1496382062;
					continue;
				case 2u:
					num = ((int)num2 * -1186660571) ^ 0x4F7B40DF;
					continue;
				case 4u:
					result = "Carbon";
					num = -41099572;
					continue;
				case 6u:
				{
					int num14;
					int num15;
					if ((uint)c > 68u)
					{
						num14 = -1055314156;
						num15 = num14;
					}
					else
					{
						num14 = -810347205;
						num15 = num14;
					}
					num = num14 ^ (int)(num2 * 478112619);
					continue;
				}
				case 17u:
				{
					int num10;
					int num11;
					if (c != 'D')
					{
						num10 = 1468210291;
						num11 = num10;
					}
					else
					{
						num10 = 2082049617;
						num11 = num10;
					}
					num = num10 ^ (int)(num2 * 401128481);
					continue;
				}
				case 8u:
					num = ((int)num2 * -20953891) ^ 0x1AB2E820;
					continue;
				case 29u:
					switch (_206B_200F_200E_202A_206F_200F_200E_202D_206E_202B_202D_206B_202A_206F_200B_202D_206C_206C_206F_206A_202C_206F_206C_202A_200F_206B_200F_202D_206D_206C_206C_206A_206F_202D_206E_200C_202D_202C_202C_200F_202E(text))
					{
					case 1:
						break;
					case 4:
						goto IL_00da;
					default:
						goto IL_01b9;
					case 2:
						goto IL_027d;
					case 3:
						goto IL_0358;
					}
					goto case 18u;
				case 23u:
					num = ((int)num2 * -1550464356) ^ -533763772;
					continue;
				case 27u:
					result = "Black Hole";
					num = -833730455;
					continue;
				case 0u:
					num = ((int)num2 * -1723109425) ^ -1883476851;
					continue;
				case 12u:
					num = (int)((num2 * 1166902097) ^ 0x3F62103C);
					continue;
				case 26u:
				{
					int num17;
					if (c != 'H')
					{
						num = -815617486;
						num17 = num;
					}
					else
					{
						num = -1194051130;
						num17 = num;
					}
					continue;
				}
				case 25u:
					num = (int)(num2 * 1303289983) ^ -1366910598;
					continue;
				case 9u:
				{
					int num12;
					int num13;
					if (c == 'C')
					{
						num12 = -750436515;
						num13 = num12;
					}
					else
					{
						num12 = -2140370370;
						num13 = num12;
					}
					num = num12 ^ (int)(num2 * 91548658);
					continue;
				}
				case 16u:
					num = ((int)num2 * -1870548981) ^ 0x44834CF7;
					continue;
				case 30u:
					goto IL_027d;
				case 24u:
					num = (int)((num2 * 1981266436) ^ 0x5B91E250);
					continue;
				case 3u:
				{
					int num8;
					int num9;
					if (text == null)
					{
						num8 = -500529692;
						num9 = num8;
					}
					else
					{
						num8 = -1089211747;
						num9 = num8;
					}
					num = num8 ^ ((int)num2 * -959052544);
					continue;
				}
				case 28u:
					result = "Herbig Ae/Be";
					num = -746536645;
					continue;
				case 11u:
				{
					int num6;
					int num7;
					if (c == 'N')
					{
						num6 = -225696062;
						num7 = num6;
					}
					else
					{
						num6 = -1121054823;
						num7 = num6;
					}
					num = num6 ^ (int)(num2 * 1257481443);
					continue;
				}
				case 5u:
					result = "White Dwarf";
					num = -423354477;
					continue;
				case 13u:
					result = "T Tauri";
					num = -833730455;
					continue;
				case 1u:
					num = ((int)num2 * -1564267579) ^ 0x416C56E9;
					continue;
				case 20u:
				{
					int num3;
					int num4;
					if (c == 'W')
					{
						num3 = 12224667;
						num4 = num3;
					}
					else
					{
						num3 = 1013423773;
						num4 = num3;
					}
					num = num3 ^ (int)(num2 * 2046526980);
					continue;
				}
				case 15u:
					goto IL_0358;
				default:
					{
						return result;
					}
					IL_0358:
					if (!_202B_206A_202E_202C_202B_200B_206B_200D_200E_202B_200E_206F_206C_200C_200B_206B_206A_206F_202C_200C_206A_202B_202E_200E_200B_202B_206A_200C_206E_200C_206B_202A_206B_200F_206A_200C_200E_200B_206A_200F_202E(text, "TTS"))
					{
						num = -16007486;
						num5 = num;
					}
					else
					{
						num = -1772843738;
						num5 = num;
					}
					continue;
					IL_027d:
					result = code;
					num = -833730455;
					continue;
					IL_01b9:
					num = (int)((num2 * 1954668903) ^ 0x57C06452);
					continue;
					IL_00da:
					if (!_202B_206A_202E_202C_202B_200B_206B_200D_200E_202B_200E_206F_206C_200C_200B_206B_206A_206F_202C_200C_206A_202B_202E_200E_200B_202B_206A_200C_206E_200C_206B_202A_206B_200F_206A_200C_200E_200B_206A_200F_202E(text, "AEBE"))
					{
						num = -1382520882;
						num16 = num;
					}
					else
					{
						num = -1727199495;
						num16 = num;
					}
					continue;
				}
				break;
			}
		}
	}
static string[] _200E_206C_200E_200E_202D_200E_206A_202E_202A_202A_202E_202A_200C_200E_206E_202D_206B_206D_200F_200C_206D_206B_206C_206D_202A_202B_200F_206E_206E_206E_202E_200C_202D_200E_206E_200B_202D_202C_200C_202E_202E(string P_0, char P_1, StringSplitOptions P_2)
	{
		return P_0.Split(P_1, P_2);
	}
static string[] _200E_202C_202E_200D_206A_202D_202B_200E_202C_206F_206E_206E_200D_202C_200C_202D_206C_200F_200D_202A_206F_200C_206B_200C_202A_200C_202B_202E_202A_202E_202C_200E_202C_200F_202B_202C_202C_206F_202A_200E_202E(string P_0, char P_1, StringSplitOptions P_2)
	{
		return P_0.Split(P_1, P_2);
	}
static int? _206D_200E_200D_206A_206E_202D_206F_202A_206F_206C_202C_202E_206C_202B_202A_200C_206E_200C_202A_200F_200C_206E_206A_202B_200C_206C_202A_202D_200B_202E_206B_200F_202C_200F_206F_202E_200B_206A_206E_202B_202E(RequirementValue P_0)
	{
		return P_0.IValue;
	}
static bool _202B_206A_202E_202C_202B_200B_206B_200D_200E_202B_200E_206F_206C_200C_200B_206B_206A_206F_202C_200C_206A_202B_202E_200E_200B_202B_206A_200C_206E_200C_206B_202A_206B_200F_206A_200C_200E_200B_206A_200F_202E(string P_0, string P_1)
	{
		return P_0 == P_1;
	}
static RequirementValueType _202D_202A_200B_202C_202A_200E_202B_202C_202C_200D_200D_200C_206F_200B_200F_200C_202A_202B_202D_202D_200B_206B_200D_200F_206A_206E_202E_202A_200D_202C_206D_206A_206F_206A_202C_206C_200F_206A_206E_206B_202E(RequirementValue P_0)
	{
		return P_0.Type;
	}
static string _206E_200B_206B_202D_200D_200D_202A_200B_206D_206C_200C_206E_206F_202D_206F_202B_202B_206D_200F_206A_202C_202C_200D_206B_202B_200B_202B_200F_200C_202E_206D_206A_200D_200C_200E_202B_202A_206D_200F_200E_202E(string P_0, string P_1)
	{
		return P_0 + P_1;
	}
static string _206D_202E_206D_200E_200F_200C_202A_200C_202B_202D_206C_206B_206C_202C_206B_200C_202B_200E_202A_206C_202D_200D_206B_200F_206C_200C_202C_200D_206F_200F_202A_200D_206B_200F_200D_202B_206D_200C_202B_200F_202E(string P_0)
	{
		return P_0.ToUpper();
	}
static char _200C_200C_202D_206D_206D_200B_202B_206E_202B_202A_206A_206F_206D_206C_206D_206F_206D_202E_202B_202A_200B_206D_202C_200D_200C_200D_200F_200F_202D_206E_202A_202A_202C_202D_202B_202B_206F_206D_200C_202E_202E(string P_0, int P_1)
	{
		return P_0[P_1];
	}
static string _200F_202D_202C_206F_200B_202B_202D_206C_206E_206F_202D_200D_200F_206B_206A_202D_200C_202B_200E_202B_206E_202E_200B_206F_202A_202A_206B_202E_200D_200F_200E_206C_200E_206B_202B_202D_202B_206C_206A_200C_202E(RequirementValue P_0)
	{
		return P_0.SValue;
	}
static string _200E_202D_200E_202A_202E_206A_206E_206A_200E_200B_200C_200D_200C_206F_202C_202C_202A_202B_202E_206D_200C_200F_200F_206C_206E_200B_206F_206F_206A_200B_206F_206E_202E_202E_206C_202E_206F_200B_202B_206C_202E(string P_0, string P_1, string P_2)
	{
		return P_0 + P_1 + P_2;
	}
static string _206F_206D_202E_200F_206F_202E_202B_202D_206A_202E_200C_206D_206C_202A_202D_202A_206B_200F_206E_206D_202E_200B_202A_206C_202D_206D_206D_202E_200C_202E_200F_200D_200B_206C_206D_200D_202C_200B_206A_202C_202E(string P_0, string P_1, string P_2, string P_3)
	{
		return P_0 + P_1 + P_2 + P_3;
	}
static int _206B_200F_200E_202A_206F_200F_200E_202D_206E_202B_202D_206B_202A_206F_200B_202D_206C_206C_206F_206A_202C_206F_206C_202A_200F_206B_200F_202D_206D_206C_206C_206A_206F_202D_206E_200C_202D_202C_202C_200F_202E(string P_0)
	{
		return P_0.Length;
	}
static string _200C_200C_200E_202D_206C_200B_202D_206A_202C_202D_202E_206A_200F_200E_202B_206E_200F_206A_202A_206B_202C_206E_202B_200C_202C_206E_200F_206F_206B_202D_202A_206E_202E_200B_202B_202A_200B_202C_206A_202C_202E(string[] P_0)
	{
		return string.Concat(P_0);
	}
static List<List<RequirementValue>> _206C_206B_202A_202E_202D_200D_206C_200C_202D_206D_200D_206A_200B_206D_206D_206C_202D_202C_206B_206E_202D_206B_206C_206A_202D_206F_206A_206D_202C_202B_200C_206C_202E_206A_206D_202B_202D_206B_206F_206C_202E(Requirement P_0)
	{
		return P_0.Value;
	}
static string _202D_206A_206E_206A_202B_202B_206B_206B_202E_200D_206B_202D_206A_206A_206D_202D_202C_206A_202B_202A_206B_202C_206D_206F_200F_200D_200B_206A_200C_200F_202B_206E_206E_200C_206F_200C_200B_202B_206E_200C_202E(string P_0)
	{
		return P_0.ToLower();
	}
static double? _200D_202B_206A_206A_206F_206C_202C_206D_206B_200D_202A_202C_202C_202D_206D_206D_200E_206F_206C_200F_206F_200D_200F_200E_206D_202D_202A_206F_202E_200C_200C_200F_200D_206D_202D_206D_200D_206C_200D_200C_202E(RequirementValue P_0)
	{
		return P_0.DValue;
	}
static string _206E_202A_202C_200E_202C_202D_206B_202A_206C_200F_206B_202A_206F_206B_206B_206F_200D_200F_206D_202B_206C_202A_202C_200E_202A_200B_202C_206D_206F_202E_206B_206B_200D_206F_206A_200E_202E_206D_206A_202C_202E(string P_0, string[] P_1)
	{
		return string.Join(P_0, P_1);
	}
static string _206F_200D_200F_202C_202A_202D_206A_200C_206B_206D_206E_202C_206E_206C_202A_200F_206A_200E_206C_206E_200B_202D_206A_202B_202B_200D_200C_200F_202E_202D_200F_206F_202C_202C_200C_200E_200D_200D_200C_206B_202E(string P_0, char P_1, char P_2)
	{
		return P_0.Replace(P_1, P_2);
	}
static void Main(string[] args){CultureInfo.CurrentCulture=CultureInfo.InvariantCulture;var c=JsonDocument.Parse(System.IO.File.ReadAllText(args[0]));var output=new Dictionary<string,object>();var f=new Formatter();foreach(var genus in c.RootElement.GetProperty("Genus").EnumerateObject())foreach(var species in genus.Value.GetProperty("Species").EnumerateObject()){var requirements=new List<string>();foreach(var section in new[]{genus.Value.GetProperty("Requirements"),species.Value.GetProperty("Requirements"),species.Value.GetProperty("PostRequirements")})foreach(var r in section.EnumerateArray())requirements.Add(f.RequirementAsString(JsonSerializer.Deserialize<Requirement>(r.GetRawText())));output.Add(species.Name,requirements);}Console.WriteLine(JsonSerializer.Serialize(output));}}
