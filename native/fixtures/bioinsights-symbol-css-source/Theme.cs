using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.CompilerServices;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace EDJP.Observatory;

internal class Theme
{
	public Dictionary<OutputSymbol, ThemeSymbol> SymbolLookup = new Dictionary<OutputSymbol, ThemeSymbol>();

	public ThemeSettings Settings { get; set; }

	public Dictionary<string, ThemeColor> Colors { get; set; }

	public Dictionary<string, ThemeSymbol> Symbols { get; set; }

	public Theme()
	{
		Colors = new Dictionary<string, ThemeColor>();
		Symbols = new Dictionary<string, ThemeSymbol>();
		Settings = new ThemeSettings();
		SymbolLookup = new Dictionary<OutputSymbol, ThemeSymbol>();
	}

	[MethodImpl(MethodImplOptions.NoInlining)]
	internal string generateStyleCSS(string localpath, string template)
	{
		string text = template;
		using (Dictionary<string, ThemeColor>.KeyCollection.Enumerator enumerator = Colors.Keys.GetEnumerator())
		{
			while (true)
			{
				IL_0086:
				int num;
				int num2;
				if (!enumerator.MoveNext())
				{
					num = 1005278558;
					num2 = num;
				}
				else
				{
					num = 980591276;
					num2 = num;
				}
				while (true)
				{
					uint num3;
					switch ((num3 = (uint)(num ^ 0x58FDCC87)) % 4)
					{
					case 2u:
						num = 980591276;
						continue;
					default:
						goto end_IL_001a;
					case 3u:
					{
						string current = enumerator.Current;
						text = _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(text, _200B_206C_206B_206F_206A_202C_202A_206D_206F_206A_202D_206B_206E_206E_200C_202B_202E_202A_200E_206D_206D_202A_202B_200F_206C_200B_200E_200C_200B_200E_206D_206E_200F_202A_200F_200D_200B_206E_206C_200F_202E("$$", _206C_200F_206F_200D_200B_202A_206D_206F_206C_206E_202B_206C_206B_202C_202C_200B_206A_200B_206B_200F_200B_202A_200E_200F_202E_200E_206F_202C_202E_206B_206D_206F_206C_206E_200D_202A_202D_200F_206D_202C_202E(current), "$$"), Colors[current].ToHex());
						num = 1924572931;
						continue;
					}
					case 0u:
						break;
					case 1u:
						goto end_IL_001a;
					}
					goto IL_0086;
					continue;
					end_IL_001a:
					break;
				}
				break;
			}
		}
		int? num4 = Settings.BorderWidth;
		string text2 = default;
		string text5 = default;
		int num6 = default;
		string[] array2 = default;
		while (true)
		{
			int num5 = 814464686;
			while (true)
			{
				uint num3;
				switch ((num3 = (uint)(num5 ^ 0x58FDCC87)) % 15)
				{
				case 4u:
					break;
				case 13u:
					text2 = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text2, "/* AUTOCOMMENT ", text5, "*/\r\n");
					num5 = 1373039105;
					continue;
				case 3u:
				{
					string text3 = text;
					string text4 = "$$ROWSPACING$$";
					num4 = Settings.RowSpacing;
					num6 = num4.Value;
					text = _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(text3, text4, num6.ToString());
					num5 = 740505152;
					continue;
				}
				case 8u:
				{
					int num12;
					if (num6 >= array2.Length)
					{
						num5 = 361388573;
						num12 = num5;
					}
					else
					{
						num5 = 382878174;
						num12 = num5;
					}
					continue;
				}
				case 10u:
					text5 = array2[num6];
					num5 = 588785556;
					continue;
				case 1u:
				{
					string text6 = text;
					string text7 = "$$BORDERWIDTH$$";
					num4 = Settings.BorderWidth;
					num6 = num4.Value;
					text = _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(text6, text7, num6.ToString());
					num5 = 1724862705;
					continue;
				}
				case 6u:
				{
					int num9;
					if (!_200C_200C_202E_206C_200B_202B_202C_200C_206D_206C_206D_206B_200F_206A_206D_206A_200B_202C_206E_206E_206C_206D_202B_202A_206B_202A_206C_202C_206D_206E_206F_200F_206D_200E_200B_206D_202B_206F_202D_206D_202E(text5, "$$"))
					{
						num5 = 1482839364;
						num9 = num5;
					}
					else
					{
						num5 = 1631796560;
						num9 = num5;
					}
					continue;
				}
				case 2u:
				{
					int num10;
					int num11;
					if (num4.HasValue)
					{
						num10 = 1045474833;
						num11 = num10;
					}
					else
					{
						num10 = 1811722828;
						num11 = num10;
					}
					num5 = num10 ^ ((int)num3 * -1319713163);
					continue;
				}
				case 12u:
					text2 = _200B_206C_206B_206F_206A_202C_202A_206D_206F_206A_202D_206B_206E_206E_200C_202B_202E_202A_200E_206D_206D_202A_202B_200F_206C_200B_200E_200C_200B_200E_206D_206E_200F_202A_200F_200D_200B_206E_206C_200F_202E(text2, text5, "\r\n");
					num5 = 1373039105;
					continue;
				case 11u:
					num4 = Settings.RowSpacing;
					num5 = 695286840;
					continue;
				case 14u:
					num6++;
					num5 = 651258347;
					continue;
				case 0u:
					num5 = (int)((num3 * 392668542) ^ 0x20515A6F);
					continue;
				case 9u:
				{
					int num7;
					int num8;
					if (num4.HasValue)
					{
						num7 = -2064986924;
						num8 = num7;
					}
					else
					{
						num7 = -1553265126;
						num8 = num7;
					}
					num5 = num7 ^ ((int)num3 * -867793882);
					continue;
				}
				case 7u:
				{
					text = _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(text, "$$ROWHEIGHT$$", Settings.RowHeight.ToString());
					string[] array = _206C_200C_206D_206E_202B_200B_200D_202E_200F_206C_200B_200B_202B_206C_206E_200C_202A_202C_202E_206E_206F_206F_202E_200E_202D_200E_200D_206F_206D_200F_200C_200E_202D_200C_200C_206B_200C_200F_206F_206B_202E(text, new string[2]
					{
						"\r\n",
						"\n"
					}, StringSplitOptions.None);
					text2 = "";
					array2 = array;
					num6 = 0;
					num5 = 393190457;
					continue;
				}
				default:
					return text2;
				}
				break;
			}
		}
	}

	internal string substituteKeys(string template, Dictionary<string, string> substitutions)
	{
		string text = template;
		using (Dictionary<string, string>.KeyCollection.Enumerator enumerator = substitutions.Keys.GetEnumerator())
		{
			while (true)
			{
				IL_0054:
				int num;
				int num2;
				if (!enumerator.MoveNext())
				{
					num = 176897977;
					num2 = num;
				}
				else
				{
					num = 1823309739;
					num2 = num;
				}
				while (true)
				{
					uint num3;
					switch ((num3 = (uint)(num ^ 0x404F7748)) % 4)
					{
					case 2u:
						num = 1823309739;
						continue;
					default:
						goto end_IL_0015;
					case 3u:
					{
						string current = enumerator.Current;
						text = _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(text, current, substitutions[current]);
						num = 832782948;
						continue;
					}
					case 0u:
						break;
					case 1u:
						goto end_IL_0015;
					}
					goto IL_0054;
					continue;
					end_IL_0015:
					break;
				}
				break;
			}
		}
		return text;
	}

	[MethodImpl(MethodImplOptions.NoInlining)]
	internal string generateSymbolCSS(string localpath, string template, string defaultimagepath)
	{
		string text = template;
		string text21 = default;
		string text10 = default;
		OutputSymbol current = default;
		ThemeSymbol themeSymbol = default;
		string text11 = default;
		string text16 = default;
		string text17 = default;
		bool? flag2 = default;
		int? num14 = default;
		string text8 = default;
		string text6 = default;
		bool flag = default;
		while (true)
		{
			int num = -1124718848;
			while (true)
			{
				string? text25;
				uint num2;
				switch ((num2 = (uint)(num ^ -728312285)) % 6)
				{
				case 0u:
					break;
				case 2u:
					text21 = ".TEMPLATE_LABEL";
					text10 = ".TEMPLATE_HOVER_AFTER:hover:after";
					if (!_202E_200D_206F_206A_206B_206D_200C_206D_202A_200E_200B_202A_206E_202A_200F_206E_202E_202B_202B_200D_206C_202C_202E_206D_206B_206C_206F_202D_202B_200F_206E_200B_206E_206B_202E_202E_206C_206F_200D_202A_202E(Settings.ThemePath))
					{
						num = -1983174664;
						continue;
					}
					text25 = defaultimagepath;
					goto IL_0075;
				case 1u:
					text25 = Settings.ThemePath;
					goto IL_0075;
				case 5u:
				{
					int num57;
					int num58;
					if (SymbolLookup != null)
					{
						num57 = 1316924018;
						num58 = num57;
					}
					else
					{
						num57 = 359070888;
						num58 = num57;
					}
					num = num57 ^ ((int)num2 * -379538225);
					continue;
				}
				case 4u:
					return text;
				default:
					{
						using (Dictionary<OutputSymbol, ThemeSymbol>.KeyCollection.Enumerator enumerator = SymbolLookup.Keys.GetEnumerator())
						{
							while (true)
							{
								IL_0df0:
								int num3;
								int num4;
								if (!enumerator.MoveNext())
								{
									num3 = -1121446603;
									num4 = num3;
								}
								else
								{
									num3 = -2142160525;
									num4 = num3;
								}
								while (true)
								{
									string? text3;
									switch ((num2 = (uint)(num3 ^ -728312285)) % 94)
									{
									case 29u:
										num3 = -2142160525;
										continue;
									default:
										goto end_IL_00d3;
									case 66u:
									{
										string text23 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(2, 1);
										defaultInterpolatedStringHandler.AppendLiteral(".");
										defaultInterpolatedStringHandler.AppendFormatted(current);
										defaultInterpolatedStringHandler.AppendLiteral(" ");
										text = _200B_206C_206B_206F_206A_202C_202A_206D_206F_206A_202D_206B_206E_206E_200C_202B_202E_202A_200E_206D_206D_202A_202B_200F_206C_200B_200E_200C_200B_200E_206D_206E_200F_202A_200F_200D_200B_206E_206C_200F_202E(text23, defaultInterpolatedStringHandler.ToStringAndClear(), "{\r\n");
										num3 = -1336326293;
										continue;
									}
									case 42u:
									{
										int num41;
										if (themeSymbol.ContentType == SymbolContentType.Image)
										{
											num3 = -216177427;
											num41 = num3;
										}
										else
										{
											num3 = -1582782344;
											num41 = num3;
										}
										continue;
									}
									case 79u:
										num3 = ((int)num2 * -240910194) ^ -2078288459;
										continue;
									case 32u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "background-color: ", Colors[themeSymbol.ContentBackColorKey].ToHex(showHash: true), ";\r\n");
										num3 = -1439607580;
										continue;
									case 70u:
									{
										string text14 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(12, 1);
										defaultInterpolatedStringHandler.AppendLiteral("width: ");
										defaultInterpolatedStringHandler.AppendFormatted(Settings.RowHeight - 1);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text14, defaultInterpolatedStringHandler.ToStringAndClear());
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "display: inline-block;\r\n");
										num3 = -266674858;
										continue;
									}
									case 90u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "font-family: ", text11, ";\r\n");
										num3 = -617743100;
										continue;
									case 19u:
									{
										string text15 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(20, 1);
										defaultInterpolatedStringHandler.AppendLiteral("padding-right: ");
										defaultInterpolatedStringHandler.AppendFormatted(themeSymbol.PaddingRight);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text15, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -352608673;
										continue;
									}
									case 65u:
										num3 = (int)(num2 * 909131465) ^ -1090447790;
										continue;
									case 20u:
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "}\r\n\r\n");
										num3 = -1188273482;
										continue;
									case 85u:
									{
										int num49;
										if (themeSymbol.FontWeight == null)
										{
											num3 = -1334010108;
											num49 = num3;
										}
										else
										{
											num3 = -2132895768;
											num49 = num3;
										}
										continue;
									}
									case 60u:
										current = enumerator.Current;
										num3 = -80533542;
										continue;
									case 57u:
										text16 = Utils.MungeURL(_200C_200C_200C_206A_206E_206E_200B_202B_200F_202B_206F_206E_202D_206A_200E_200D_200B_200E_202C_202A_202B_206E_200C_202E_202D_206B_206A_202A_202A_206B_200B_200C_206F_206A_206D_206B_200F_202B_200C_200C_202E(text17, themeSymbol.Content));
										num3 = (int)((num2 * 814610456) ^ 0x177B6D20);
										continue;
									case 93u:
									{
										int num33;
										int num34;
										if (themeSymbol.ContentType != SymbolContentType.Text)
										{
											num33 = -662162084;
											num34 = num33;
										}
										else
										{
											num33 = -22755255;
											num34 = num33;
										}
										num3 = num33 ^ ((int)num2 * -1404639227);
										continue;
									}
									case 81u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "background-color: ", themeSymbol.ContentBackColor.ToHex(showHash: true), ";\r\n");
										num3 = -981939953;
										continue;
									case 50u:
									{
										int num22;
										int num23;
										if (_206A_200D_206D_200B_206E_206F_200C_206C_206C_200C_200D_206D_202D_202B_202B_202E_202B_202A_206E_206B_206A_206A_206E_206E_200B_200E_200D_200C_206A_202E_200C_200F_202A_202E_206A_206F_200C_200B_202E_206D_202E(themeSymbol.Font, '\''))
										{
											num22 = -24156410;
											num23 = num22;
										}
										else
										{
											num22 = -313499842;
											num23 = num22;
										}
										num3 = num22 ^ ((int)num2 * -480352775);
										continue;
									}
									case 55u:
										themeSymbol = SymbolLookup[current];
										flag2 = themeSymbol.SupressMouseOver;
										num3 = ((int)num2 * -1891816177) ^ 0x37E618B5;
										continue;
									case 74u:
									{
										int num13;
										if (themeSymbol.ContentType == SymbolContentType.Base64SVG)
										{
											num3 = -1298568673;
											num13 = num3;
										}
										else
										{
											num3 = -446820298;
											num13 = num3;
										}
										continue;
									}
									case 76u:
									{
										string text4 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(9, 1);
										defaultInterpolatedStringHandler.AppendLiteral(".");
										defaultInterpolatedStringHandler.AppendFormatted(current);
										defaultInterpolatedStringHandler.AppendLiteral(":before ");
										text = _200B_206C_206B_206F_206A_202C_202A_206D_206F_206A_202D_206B_206E_206E_200C_202B_202E_202A_200E_206D_206D_202A_202B_200F_206C_200B_200E_200C_200B_200E_206D_206E_200F_202A_200F_200D_200B_206E_206C_200F_202E(text4, defaultInterpolatedStringHandler.ToStringAndClear(), "{\r\n");
										num3 = -2121162362;
										continue;
									}
									case 53u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "content: \"", themeSymbol.Content, "\";\r\n");
										num3 = -594846822;
										continue;
									case 52u:
									{
										string text2 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(21, 1);
										defaultInterpolatedStringHandler.AppendLiteral("padding-bottom: ");
										defaultInterpolatedStringHandler.AppendFormatted(themeSymbol.PaddingBottom);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text2, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -835981117;
										continue;
									}
									case 92u:
										num3 = ((int)num2 * -1248742587) ^ -1413835425;
										continue;
									case 7u:
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "vertical-align: bottom;\r\n");
										num3 = -861555110;
										continue;
									case 17u:
									{
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "background-image: url(\"data:image/png;base64,", themeSymbol.Content, "\");\r\n");
										string text24 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(13, 1);
										defaultInterpolatedStringHandler.AppendLiteral("height: ");
										defaultInterpolatedStringHandler.AppendFormatted(Settings.RowHeight - 1);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text24, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -1754419133;
										continue;
									}
									case 21u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "content: url(", text16, ");\r\n");
										num3 = -763347914;
										continue;
									case 84u:
									{
										int num53;
										if (_202E_200D_206F_206A_206B_206D_200C_206D_202A_200E_200B_202A_206E_202A_200F_206E_202E_202B_202B_200D_206C_202C_202E_206D_206B_206C_206F_202D_202B_200F_206E_200B_206E_206B_202E_202E_206C_206F_200D_202A_202E(themeSymbol.Description))
										{
											num3 = -1136208726;
											num53 = num3;
										}
										else
										{
											num3 = -673606652;
											num53 = num3;
										}
										continue;
									}
									case 59u:
										num14 = themeSymbol.PaddingBottom;
										num3 = -1728008195;
										continue;
									case 38u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "content: url(", themeSymbol.Content, ");\r\n");
										num3 = -1623850477;
										continue;
									case 4u:
									{
										string text20 = text21;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(4, 1);
										defaultInterpolatedStringHandler.AppendLiteral(",\r\n.");
										defaultInterpolatedStringHandler.AppendFormatted(current);
										text21 = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text20, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -1714876222;
										continue;
									}
									case 12u:
									{
										string text19 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(19, 1);
										defaultInterpolatedStringHandler.AppendLiteral("padding-left: ");
										defaultInterpolatedStringHandler.AppendFormatted(themeSymbol.PaddingLeft);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text19, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -12188989;
										continue;
									}
									case 25u:
									{
										int num46;
										if (!_206A_200D_206D_200B_206E_206F_200C_206C_206C_200C_200D_206D_202D_202B_202B_202E_202B_202A_206E_206B_206A_206A_206E_206E_200B_200E_200D_200C_206A_202E_200C_200F_202A_202E_206A_206F_200C_200B_202E_206D_202E(themeSymbol.Font, '"'))
										{
											num3 = -1321386434;
											num46 = num3;
										}
										else
										{
											num3 = -2095005259;
											num46 = num3;
										}
										continue;
									}
									case 18u:
									{
										string text18 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(18, 1);
										defaultInterpolatedStringHandler.AppendLiteral("padding-top: ");
										defaultInterpolatedStringHandler.AppendFormatted(themeSymbol.PaddingTop);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text18, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -178162030;
										continue;
									}
									case 82u:
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "padding-right: 1px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "vertical-align: bottom;\r\n");
										num3 = -1247060480;
										continue;
									case 44u:
										text8 = Utils.MungeURL(_200C_200C_200C_206A_206E_206E_200B_202B_200F_202B_206F_206E_202D_206A_200E_200D_200B_200E_202C_202A_202B_206E_200C_202E_202D_206B_206A_202A_202A_206B_200B_200C_206F_206A_206D_206B_200F_202B_200C_200C_202E(localpath, themeSymbol.Content));
										num3 = ((int)num2 * -1116585476) ^ -831993698;
										continue;
									case 3u:
										text3 = _200B_206C_206B_206F_206A_202C_202A_206D_206F_206A_202D_206B_206E_206E_200C_202B_202E_202A_200E_206D_206D_202A_202B_200F_206C_200B_200E_200C_200B_200E_206D_206E_200F_202A_200F_200D_200B_206E_206C_200F_202E("'", themeSymbol.Font, "'");
										goto IL_092f;
									case 71u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "font-weight: ", themeSymbol.FontWeight, ";\r\n");
										num3 = -1334010108;
										continue;
									case 26u:
									{
										int num37;
										int num38;
										if (Colors.ContainsKey(themeSymbol.ContentColorKey))
										{
											num37 = -1372085369;
											num38 = num37;
										}
										else
										{
											num37 = -539400207;
											num38 = num37;
										}
										num3 = num37 ^ (int)(num2 * 904008382);
										continue;
									}
									case 31u:
									{
										int num31;
										int num32;
										if (themeSymbol.ContentColor != null)
										{
											num31 = -1994015707;
											num32 = num31;
										}
										else
										{
											num31 = -1923181423;
											num32 = num31;
										}
										num3 = num31 ^ ((int)num2 * -812797004);
										continue;
									}
									case 75u:
									{
										int num29;
										if (themeSymbol.ContentType != SymbolContentType.Base64PNG)
										{
											num3 = -551284521;
											num29 = num3;
										}
										else
										{
											num3 = -87177492;
											num29 = num3;
										}
										continue;
									}
									case 6u:
									{
										string text13 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(16, 1);
										defaultInterpolatedStringHandler.AppendLiteral("font-size: ");
										defaultInterpolatedStringHandler.AppendFormatted(themeSymbol.FontSize);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text13, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -33633860;
										continue;
									}
									case 22u:
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "}\r\n\r\n");
										num3 = -1136208726;
										continue;
									case 62u:
									{
										string text12 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(12, 1);
										defaultInterpolatedStringHandler.AppendLiteral("width: ");
										defaultInterpolatedStringHandler.AppendFormatted(themeSymbol.Width);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text12, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -587125487;
										continue;
									}
									case 27u:
									{
										int num24;
										int num25;
										if (!flag2.HasValue)
										{
											num24 = -1430678290;
											num25 = num24;
										}
										else
										{
											num24 = -1054998455;
											num25 = num24;
										}
										num3 = num24 ^ ((int)num2 * -131813911);
										continue;
									}
									case 13u:
										flag2 = themeSymbol.SupressMouseOver;
										num3 = (int)(num2 * 124887443) ^ -280143726;
										continue;
									case 64u:
										text6 = Utils.Base64Decode(themeSymbol.Content);
										num3 = (int)((num2 * 289448267) ^ 0x320306EE);
										continue;
									case 0u:
									{
										string text9 = text10;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(16, 1);
										defaultInterpolatedStringHandler.AppendLiteral(",\r\n.");
										defaultInterpolatedStringHandler.AppendFormatted(current);
										defaultInterpolatedStringHandler.AppendLiteral(":hover:after");
										text10 = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text9, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -1438417945;
										continue;
									}
									case 89u:
									{
										flag2 = themeSymbol.IsLabel;
										int num17;
										if (flag2.HasValue)
										{
											num3 = -1496886691;
											num17 = num3;
										}
										else
										{
											num3 = -1714876222;
											num17 = num3;
										}
										continue;
									}
									case 67u:
									{
										flag2 = themeSymbol.SupressMouseOver;
										int num9;
										int num10;
										if (!flag2.HasValue)
										{
											num9 = 296161320;
											num10 = num9;
										}
										else
										{
											num9 = 416915079;
											num10 = num9;
										}
										num3 = num9 ^ ((int)num2 * -222954935);
										continue;
									}
									case 36u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "vertical-align: ", themeSymbol.VerticalAlign, ";\r\n");
										num3 = -85376816;
										continue;
									case 83u:
										if (!_200D_202B_206F_202D_206C_200D_202C_206D_206A_200E_206F_200E_206D_202B_202C_206B_202B_200C_206B_200B_206F_202A_206D_202E_206C_202C_202C_200D_202A_200D_202E_202D_206B_200B_202D_200B_202A_206B_206B_206C_202E(themeSymbol.Font, '\''))
										{
											num3 = (int)((num2 * 184543151) ^ 0x1BA0FBD5);
											continue;
										}
										goto IL_0928;
									case 14u:
									{
										int num55;
										if (themeSymbol.ContentType == SymbolContentType.BuiltIn)
										{
											num3 = -677930681;
											num55 = num3;
										}
										else
										{
											num3 = -1623850477;
											num55 = num3;
										}
										continue;
									}
									case 5u:
									{
										int num54;
										if (themeSymbol.ContentBackColorKey == null)
										{
											num3 = -1439607580;
											num54 = num3;
										}
										else
										{
											num3 = -1993801978;
											num54 = num3;
										}
										continue;
									}
									case 63u:
									{
										int num51;
										int num52;
										if (!Colors.ContainsKey(themeSymbol.ContentColorKey))
										{
											num51 = 632295839;
											num52 = num51;
										}
										else
										{
											num51 = 672118031;
											num52 = num51;
										}
										num3 = num51 ^ (int)(num2 * 778132586);
										continue;
									}
									case 45u:
									{
										int num50;
										if (themeSymbol.ContentType != SymbolContentType.ThemeImage)
										{
											num3 = -518494823;
											num50 = num3;
										}
										else
										{
											num3 = -492300702;
											num50 = num3;
										}
										continue;
									}
									case 28u:
									{
										string text22 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(12, 1);
										defaultInterpolatedStringHandler.AppendLiteral("width: ");
										defaultInterpolatedStringHandler.AppendFormatted(Settings.RowHeight - 1);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text22, defaultInterpolatedStringHandler.ToStringAndClear());
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "display: inline-block;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "background-position: center;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "background-repeat: no-repeat;\r\n");
										num3 = -1080960920;
										continue;
									}
									case 41u:
									{
										int num47;
										int num48;
										if (!num14.HasValue)
										{
											num47 = 1406098811;
											num48 = num47;
										}
										else
										{
											num47 = 1991084069;
											num48 = num47;
										}
										num3 = num47 ^ ((int)num2 * -1338583368);
										continue;
									}
									case 72u:
										flag2 = themeSymbol.IsLabel;
										num3 = (int)((num2 * 192304629) ^ 0x163BA67A);
										continue;
									case 68u:
									{
										int num45;
										if (themeSymbol.ContentColorKey == null)
										{
											num3 = -1776958255;
											num45 = num3;
										}
										else
										{
											num3 = -1820975533;
											num45 = num3;
										}
										continue;
									}
									case 2u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "content: \"", themeSymbol.Description, "\";\r\n");
										num3 = -2017676291;
										continue;
									case 77u:
									{
										int num44;
										if (themeSymbol.Font != null)
										{
											num3 = -1652380527;
											num44 = num3;
										}
										else
										{
											num3 = -617743100;
											num44 = num3;
										}
										continue;
									}
									case 58u:
									{
										int num43;
										if (themeSymbol.VerticalAlign != null)
										{
											num3 = -827934035;
											num43 = num3;
										}
										else
										{
											num3 = -85376816;
											num43 = num3;
										}
										continue;
									}
									case 43u:
										break;
									case 46u:
									{
										num14 = themeSymbol.PaddingRight;
										int num42;
										if (!num14.HasValue)
										{
											num3 = -352608673;
											num42 = num3;
										}
										else
										{
											num3 = -2103496244;
											num42 = num3;
										}
										continue;
									}
									case 11u:
										num3 = (int)((num2 * 2075190262) ^ 0x3AD2963D);
										continue;
									case 91u:
										flag2 = themeSymbol.SupressMouseOver;
										flag = false;
										num3 = (int)(num2 * 181018221) ^ -115195407;
										continue;
									case 87u:
									{
										int num39;
										int num40;
										if (flag2 != flag)
										{
											num39 = -2007528314;
											num40 = num39;
										}
										else
										{
											num39 = -645789948;
											num40 = num39;
										}
										num3 = num39 ^ ((int)num2 * -714636385);
										continue;
									}
									case 47u:
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "content: \" \";\r\n");
										num3 = -1900101706;
										continue;
									case 78u:
										num3 = ((int)num2 * -1193651310) ^ 0x33505FFC;
										continue;
									case 35u:
									{
										int num35;
										int num36;
										if (num14.HasValue)
										{
											num35 = 1192284775;
											num36 = num35;
										}
										else
										{
											num35 = 408011985;
											num36 = num35;
										}
										num3 = num35 ^ (int)(num2 * 1578889792);
										continue;
									}
									case 88u:
										if (!_200D_202B_206F_202D_206C_200D_202C_206D_206A_200E_206F_200E_206D_202B_202C_206B_202B_200C_206B_200B_206F_202A_206D_202E_206C_202C_202C_200D_202A_200D_202E_202D_206B_200B_202D_200B_202A_206B_206B_206C_202E(themeSymbol.Font, '"'))
										{
											num3 = ((int)num2 * -1691634555) ^ -748910896;
											continue;
										}
										goto IL_0928;
									case 61u:
									{
										num14 = themeSymbol.FontSize;
										int num30;
										if (num14.HasValue)
										{
											num3 = -834769321;
											num30 = num3;
										}
										else
										{
											num3 = -33633860;
											num30 = num3;
										}
										continue;
									}
									case 56u:
									{
										num14 = themeSymbol.PaddingTop;
										int num28;
										if (!num14.HasValue)
										{
											num3 = -178162030;
											num28 = num3;
										}
										else
										{
											num3 = -457959641;
											num28 = num3;
										}
										continue;
									}
									case 30u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "color: ", Colors[themeSymbol.ContentColorKey].ToHex(showHash: true), ";\r\n");
										num3 = -1264675775;
										continue;
									case 34u:
									{
										int num27;
										if (themeSymbol.ContentBackColor != null)
										{
											num3 = -1022613082;
											num27 = num3;
										}
										else
										{
											num3 = -927441992;
											num27 = num3;
										}
										continue;
									}
									case 37u:
										num14 = themeSymbol.Width;
										num3 = -2093876788;
										continue;
									case 16u:
										text6 = _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(text6, "$$COLOR$$", Colors[themeSymbol.ContentColorKey].ToHex());
										num3 = -1776958255;
										continue;
									case 10u:
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "}\r\n\r\n");
										num3 = -1180946375;
										continue;
									case 51u:
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "padding-left: 1px;\r\n");
										num3 = -414432245;
										continue;
									case 15u:
										num14 = themeSymbol.PaddingLeft;
										num3 = -741808702;
										continue;
									case 73u:
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "background-position: center;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "background-repeat: no-repeat;\r\n");
										num3 = -1264446886;
										continue;
									case 1u:
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text, "content: \" \";\r\n");
										num3 = -1623850477;
										continue;
									case 23u:
									{
										int num26;
										if (themeSymbol.ContentColorKey == null)
										{
											num3 = -1264675775;
											num26 = num3;
										}
										else
										{
											num3 = -427198672;
											num26 = num3;
										}
										continue;
									}
									case 48u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "color: ", themeSymbol.ContentColor.ToHex(showHash: true), ";\r\n");
										num3 = -1264675775;
										continue;
									case 54u:
										text6 = _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(text6, "$$COLOR$$", themeSymbol.ContentColor.ToHex());
										num3 = -1676420331;
										continue;
									case 40u:
									{
										int num20;
										int num21;
										if (themeSymbol.ContentColor != null)
										{
											num20 = 927041091;
											num21 = num20;
										}
										else
										{
											num20 = 1904449430;
											num21 = num20;
										}
										num3 = num20 ^ ((int)num2 * -234980456);
										continue;
									}
									case 69u:
									{
										int num18;
										int num19;
										if (themeSymbol.ContentType == SymbolContentType.None)
										{
											num18 = 724917185;
											num19 = num18;
										}
										else
										{
											num18 = 1181981981;
											num19 = num18;
										}
										num3 = num18 ^ (int)(num2 * 888735256);
										continue;
									}
									case 9u:
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "content: url(", text8, ");\r\n");
										num3 = -1623850477;
										continue;
									case 8u:
									{
										text = _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(text, "background-image: url(\"data:image/svg+xml;base64,", Utils.Base64Encode(text6), "\");\r\n");
										string text7 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(13, 1);
										defaultInterpolatedStringHandler.AppendLiteral("height: ");
										defaultInterpolatedStringHandler.AppendFormatted(Settings.RowHeight - 1);
										defaultInterpolatedStringHandler.AppendLiteral("px;\r\n");
										text = _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(text7, defaultInterpolatedStringHandler.ToStringAndClear());
										num3 = -807126127;
										continue;
									}
									case 33u:
									{
										string text5 = text;
										DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(14, 1);
										defaultInterpolatedStringHandler.AppendLiteral(".");
										defaultInterpolatedStringHandler.AppendFormatted(current);
										defaultInterpolatedStringHandler.AppendLiteral(":hover:after ");
										text = _200B_206C_206B_206F_206A_202C_202A_206D_206F_206A_202D_206B_206E_206E_200C_202B_202E_202A_200E_206D_206D_202A_202B_200F_206C_200B_200E_200C_200B_200E_206D_206E_200F_202A_200F_200D_200B_206E_206C_200F_202E(text5, defaultInterpolatedStringHandler.ToStringAndClear(), "{\r\n");
										num3 = -166405188;
										continue;
									}
									case 24u:
									{
										int num15;
										int num16;
										if (!num14.HasValue)
										{
											num15 = 1943283353;
											num16 = num15;
										}
										else
										{
											num15 = 1597137047;
											num16 = num15;
										}
										num3 = num15 ^ (int)(num2 * 1354224035);
										continue;
									}
									case 49u:
									{
										int num11;
										int num12;
										if (flag2 == true)
										{
											num11 = -374691812;
											num12 = num11;
										}
										else
										{
											num11 = -1826091137;
											num12 = num11;
										}
										num3 = num11 ^ (int)(num2 * 2059466675);
										continue;
									}
									case 39u:
									{
										int num7;
										int num8;
										if (!Colors.ContainsKey(themeSymbol.ContentBackColorKey))
										{
											num7 = -924919781;
											num8 = num7;
										}
										else
										{
											num7 = -1612407228;
											num8 = num7;
										}
										num3 = num7 ^ (int)(num2 * 1130921555);
										continue;
									}
									case 80u:
									{
										flag = false;
										int num5;
										int num6;
										if (flag2 != flag)
										{
											num5 = 1902367751;
											num6 = num5;
										}
										else
										{
											num5 = 1738830567;
											num6 = num5;
										}
										num3 = num5 ^ ((int)num2 * -1834775364);
										continue;
									}
									case 86u:
										goto end_IL_00d3;
										IL_0928:
										text3 = themeSymbol.Font;
										goto IL_092f;
										IL_092f:
										text11 = text3;
										num3 = -231249807;
										continue;
									}
									goto IL_0df0;
									continue;
									end_IL_00d3:
									break;
								}
								break;
							}
						}
						text = _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(text, "$$TEMPLATE_LABEL$$", text21);
						while (true)
						{
							int num56 = -615223953;
							while (true)
							{
								switch ((num2 = (uint)(num56 ^ -728312285)) % 3)
								{
								case 2u:
									break;
								case 1u:
									goto IL_136e;
								default:
									return text;
								}
								break;
								IL_136e:
								text = _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(text, "$$TEMPLATE_HOVER_AFTER$$", text10);
								num56 = -1537178677;
							}
						}
					}
					IL_0075:
					text17 = text25;
					num = -1420686166;
					continue;
				}
				break;
			}
		}
	}

	public static Theme Load(string themetext)
	{
		JsonSerializerOptions jsonSerializerOptions = _200D_206C_202E_206C_202C_202D_206F_206C_206A_200C_202C_206A_206F_200C_206E_200F_206A_206A_206C_206F_200D_200D_200F_202E_206B_200C_206A_200D_206A_200D_206E_206A_202C_202D_200B_200F_200D_202A_202A_202A_202E();
		_206C_206C_202C_200E_200B_202E_206D_200D_206D_200D_200F_202E_200C_200C_200D_206C_200B_206E_200F_206F_200E_206D_206A_206F_206B_206B_206D_206E_200F_206F_200E_206B_200E_202B_202D_200F_202D_200D_206A_200D_202E(jsonSerializerOptions).Add(_206C_202B_202E_206D_206A_200F_202E_206B_206C_200E_202D_202A_206D_200F_206A_200F_202B_200D_202B_200D_206C_206C_200D_200E_206A_202D_200F_206F_200E_200C_200E_200E_202A_206A_200C_206C_202C_206C_200F_206E_202E());
		JsonSerializerOptions options = jsonSerializerOptions;
		Theme theme = JsonSerializer.Deserialize<Theme>(themetext, options);
		OutputSymbol key = default;
		string current = default;
		while (true)
		{
			int num = -2096962859;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -1849578508)) % 3)
				{
				case 2u:
					break;
				case 1u:
					goto IL_0041;
				default:
				{
					using (Dictionary<string, ThemeSymbol>.KeyCollection.Enumerator enumerator = theme.Symbols.Keys.GetEnumerator())
					{
						while (true)
						{
							IL_00a5:
							int num3;
							int num4;
							if (enumerator.MoveNext())
							{
								num3 = -1590251366;
								num4 = num3;
							}
							else
							{
								num3 = -2114254686;
								num4 = num3;
							}
							while (true)
							{
								switch ((num2 = (uint)(num3 ^ -1849578508)) % 7)
								{
								case 3u:
									num3 = -1590251366;
									continue;
								default:
									goto end_IL_0074;
								case 1u:
									break;
								case 6u:
									key = (OutputSymbol)_206D_200B_202A_206F_202A_200C_200C_202B_200E_200F_200D_206C_206B_206B_206D_202A_206F_206C_202C_202D_202A_206A_202E_202C_206D_206B_202D_200B_206C_202C_206C_200C_202B_206B_200C_202D_202B_202D_200C_200F_202E(_200D_202C_206F_206D_202C_206C_202D_200D_202C_206A_202B_206B_206A_206C_200C_206C_206E_206D_200D_206D_200D_202A_202D_200D_206C_202D_202E_200F_202E_200D_206A_206B_202E_206F_202A_202D_202D_202B_206B_200F_202E(typeof(OutputSymbol).TypeHandle), current, true);
									num3 = ((int)num2 * -1777855342) ^ 0x4ECAEF1B;
									continue;
								case 0u:
								{
									int num5;
									int num6;
									if (_206F_206C_200B_206A_200D_206F_206F_202C_202C_202B_200B_202D_202C_202B_202C_202E_202A_206F_202D_202A_202D_206B_206F_206E_200F_202C_202E_200B_206F_206E_200B_206C_206B_206F_200C_206B_206D_200F_200C_200C_202E(_200D_202C_206F_206D_202C_206C_202D_200D_202C_206A_202B_206B_206A_206C_200C_206C_206E_206D_200D_206D_200D_202A_202D_200D_206C_202D_202E_200F_202E_200D_206A_206B_202E_206F_202A_202D_202D_202B_206B_200F_202E(typeof(OutputSymbol).TypeHandle), (object)current))
									{
										num5 = 1127705057;
										num6 = num5;
									}
									else
									{
										num5 = 813177145;
										num6 = num5;
									}
									num3 = num5 ^ (int)(num2 * 1258882869);
									continue;
								}
								case 4u:
									current = enumerator.Current;
									num3 = -581986780;
									continue;
								case 5u:
									theme.SymbolLookup.Add(key, theme.Symbols[current]);
									num3 = (int)(num2 * 2144332550) ^ -1965709033;
									continue;
								case 2u:
									goto end_IL_0074;
								}
								goto IL_00a5;
								continue;
								end_IL_0074:
								break;
							}
							break;
						}
					}
					return theme;
				}
				}
				break;
				IL_0041:
				theme.SymbolLookup.Clear();
				num = ((int)num2 * -1968192289) ^ 0x5C4B174B;
			}
		}
	}

	public void OverrideColors(string overridetext)
	{
		JsonSerializerOptions jsonSerializerOptions = _200D_206C_202E_206C_202C_202D_206F_206C_206A_200C_202C_206A_206F_200C_206E_200F_206A_206A_206C_206F_200D_200D_200F_202E_206B_200C_206A_200D_206A_200D_206E_206A_202C_202D_200B_200F_200D_202A_202A_202A_202E();
		_206C_206C_202C_200E_200B_202E_206D_200D_206D_200D_200F_202E_200C_200C_200D_206C_200B_206E_200F_206F_200E_206D_206A_206F_206B_206B_206D_206E_200F_206F_200E_206B_200E_202B_202D_200F_202D_200D_206A_200D_202E(jsonSerializerOptions).Add(_206C_202B_202E_206D_206A_200F_202E_206B_206C_200E_202D_202A_206D_200F_206A_200F_202B_200D_202B_200D_206C_206C_200D_200E_206A_202D_200F_206F_200E_200C_200E_200E_202A_206A_200C_206C_202C_206C_200F_206E_202E());
		JsonSerializerOptions options = jsonSerializerOptions;
		Dictionary<string, ThemeColor> dictionary = default;
		string current = default;
		while (true)
		{
			int num = 712388352;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0x5B54A686)) % 3)
				{
				case 2u:
					break;
				case 1u:
					goto IL_0039;
				default:
				{
					using Dictionary<string, ThemeColor>.KeyCollection.Enumerator enumerator = dictionary.Keys.GetEnumerator();
					while (true)
					{
						int num3;
						int num4;
						if (enumerator.MoveNext())
						{
							num3 = 656395714;
							num4 = num3;
						}
						else
						{
							num3 = 1360273634;
							num4 = num3;
						}
						while (true)
						{
							switch ((num2 = (uint)(num3 ^ 0x5B54A686)) % 5)
							{
							case 0u:
								num3 = 656395714;
								continue;
							default:
								return;
							case 1u:
								break;
							case 2u:
								Colors[current] = dictionary[current];
								num3 = ((int)num2 * -2099312670) ^ 0x2ADC0E01;
								continue;
							case 4u:
							{
								current = enumerator.Current;
								int num5;
								if (Colors.ContainsKey(current))
								{
									num3 = 198562815;
									num5 = num3;
								}
								else
								{
									num3 = 1211829715;
									num5 = num3;
								}
								continue;
							}
							case 3u:
								return;
							}
							break;
						}
					}
				}
				}
				break;
				IL_0039:
				dictionary = JsonSerializer.Deserialize<Dictionary<string, ThemeColor>>(overridetext, options);
				num = (int)((num2 * 1315782718) ^ 0x5A99D34D);
			}
		}
	}

	static string _206C_200F_206F_200D_200B_202A_206D_206F_206C_206E_202B_206C_206B_202C_202C_200B_206A_200B_206B_200F_200B_202A_200E_200F_202E_200E_206F_202C_202E_206B_206D_206F_206C_206E_200D_202A_202D_200F_206D_202C_202E(string P_0)
	{
		return P_0.ToUpper();
	}

	static string _200B_206C_206B_206F_206A_202C_202A_206D_206F_206A_202D_206B_206E_206E_200C_202B_202E_202A_200E_206D_206D_202A_202B_200F_206C_200B_200E_200C_200B_200E_206D_206E_200F_202A_200F_200D_200B_206E_206C_200F_202E(string P_0, string P_1, string P_2)
	{
		return P_0 + P_1 + P_2;
	}

	static string _206D_202A_206F_206F_206D_200F_202C_202D_202A_200C_202B_202C_206C_202B_206E_202B_206B_202E_202C_206D_202D_202A_206C_200B_202B_200C_206D_206E_200F_206D_202A_200C_202A_206A_206C_200E_200C_206A_200B_202D_202E(string P_0, string P_1, string P_2)
	{
		return P_0.Replace(P_1, P_2);
	}

	static string[] _206C_200C_206D_206E_202B_200B_200D_202E_200F_206C_200B_200B_202B_206C_206E_200C_202A_202C_202E_206E_206F_206F_202E_200E_202D_200E_200D_206F_206D_200F_200C_200E_202D_200C_200C_206B_200C_200F_206F_206B_202E(string P_0, string[] P_1, StringSplitOptions P_2)
	{
		return P_0.Split(P_1, P_2);
	}

	static bool _200C_200C_202E_206C_200B_202B_202C_200C_206D_206C_206D_206B_200F_206A_206D_206A_200B_202C_206E_206E_206C_206D_202B_202A_206B_202A_206C_202C_206D_206E_206F_200F_206D_200E_200B_206D_202B_206F_202D_206D_202E(string P_0, string P_1)
	{
		return P_0.Contains(P_1);
	}

	static string _206A_206B_206A_202C_200C_202E_200B_202A_202C_202B_206C_200E_206A_200C_200B_200E_206B_206E_202C_202B_206D_206A_202D_202A_200C_202A_206D_202C_200B_206B_200B_200E_200E_200D_200C_200D_206B_206B_202B_206A_202E(string P_0, string P_1, string P_2, string P_3)
	{
		return P_0 + P_1 + P_2 + P_3;
	}

	static bool _202E_200D_206F_206A_206B_206D_200C_206D_202A_200E_200B_202A_206E_202A_200F_206E_202E_202B_202B_200D_206C_202C_202E_206D_206B_206C_206F_202D_202B_200F_206E_200B_206E_206B_202E_202E_206C_206F_200D_202A_202E(string P_0)
	{
		return string.IsNullOrEmpty(P_0);
	}

	static string _206D_206A_202A_200B_200C_200E_202A_200D_202B_202E_200F_200E_200D_200D_200E_206F_206C_200F_202A_200F_206E_200F_200E_206C_200E_200D_202B_200D_200F_200F_206C_200D_206F_206A_206A_206D_206B_202C_206D_200E_202E(string P_0, string P_1)
	{
		return P_0 + P_1;
	}

	static bool _206A_200D_206D_200B_206E_206F_200C_206C_206C_200C_200D_206D_202D_202B_202B_202E_202B_202A_206E_206B_206A_206A_206E_206E_200B_200E_200D_200C_206A_202E_200C_200F_202A_202E_206A_206F_200C_200B_202E_206D_202E(string P_0, char P_1)
	{
		return P_0.StartsWith(P_1);
	}

	static bool _200D_202B_206F_202D_206C_200D_202C_206D_206A_200E_206F_200E_206D_202B_202C_206B_202B_200C_206B_200B_206F_202A_206D_202E_206C_202C_202C_200D_202A_200D_202E_202D_206B_200B_202D_200B_202A_206B_206B_206C_202E(string P_0, char P_1)
	{
		return P_0.EndsWith(P_1);
	}

	static string _200C_200C_200C_206A_206E_206E_200B_202B_200F_202B_206F_206E_202D_206A_200E_200D_200B_200E_202C_202A_202B_206E_200C_202E_202D_206B_206A_202A_202A_206B_200B_200C_206F_206A_206D_206B_200F_202B_200C_200C_202E(string P_0, string P_1)
	{
		return Path.Combine(P_0, P_1);
	}

	static JsonSerializerOptions _200D_206C_202E_206C_202C_202D_206F_206C_206A_200C_202C_206A_206F_200C_206E_200F_206A_206A_206C_206F_200D_200D_200F_202E_206B_200C_206A_200D_206A_200D_206E_206A_202C_202D_200B_200F_200D_202A_202A_202A_202E()
	{
		return new JsonSerializerOptions();
	}

	static IList<JsonConverter> _206C_206C_202C_200E_200B_202E_206D_200D_206D_200D_200F_202E_200C_200C_200D_206C_200B_206E_200F_206F_200E_206D_206A_206F_206B_206B_206D_206E_200F_206F_200E_206B_200E_202B_202D_200F_202D_200D_206A_200D_202E(JsonSerializerOptions P_0)
	{
		return P_0.Converters;
	}

	static JsonStringEnumConverter _206C_202B_202E_206D_206A_200F_202E_206B_206C_200E_202D_202A_206D_200F_206A_200F_202B_200D_202B_200D_206C_206C_200D_200E_206A_202D_200F_206F_200E_200C_200E_200E_202A_206A_200C_206C_202C_206C_200F_206E_202E()
	{
		return new JsonStringEnumConverter();
	}

	static Type _200D_202C_206F_206D_202C_206C_202D_200D_202C_206A_202B_206B_206A_206C_200C_206C_206E_206D_200D_206D_200D_202A_202D_200D_206C_202D_202E_200F_202E_200D_206A_206B_202E_206F_202A_202D_202D_202B_206B_200F_202E(RuntimeTypeHandle P_0)
	{
		return Type.GetTypeFromHandle(P_0);
	}

	static bool _206F_206C_200B_206A_200D_206F_206F_202C_202C_202B_200B_202D_202C_202B_202C_202E_202A_206F_202D_202A_202D_206B_206F_206E_200F_202C_202E_200B_206F_206E_200B_206C_206B_206F_200C_206B_206D_200F_200C_200C_202E(Type P_0, object P_1)
	{
		return Enum.IsDefined(P_0, P_1);
	}

	static object _206D_200B_202A_206F_202A_200C_200C_202B_200E_200F_200D_206C_206B_206B_206D_202A_206F_206C_202C_202D_202A_206A_202E_202C_206D_206B_202D_200B_206C_202C_206C_200C_202B_206B_200C_202D_202B_202D_200C_200F_202E(Type P_0, string P_1, bool P_2)
	{
		return Enum.Parse(P_0, P_1, P_2);
	}
}
