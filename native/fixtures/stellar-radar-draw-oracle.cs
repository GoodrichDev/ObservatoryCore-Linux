// Independent installed StellarOverlay.Radar.Draw capture. The method and five
// wrappers below are copied verbatim; data/render stubs only record commands.
using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text.Json;
using System.Globalization;
public enum RadarPointType {None,Vehicle,Biological,Waypoint}
public enum CrosshairStyle {Gap,Plus,Cross}
public enum AntialiasMode {Aliased,PerPrimitive}
public record Font;
public record SolidBrush(string Key);
public class RenderTarget {public AntialiasMode AntialiasMode;}
public struct Point {public float X,Y;public Point(float x,float y){X=x;Y=y;}}
public class BrushHandler {public SolidBrush Brush(string key)=>new(key);}
public class RadarPosition {public double Latitude,Longitude,Heading;}
public class RadarPoint {public RadarPointType Type;public int ContextID;public float Radius;public string Label="";public bool MeasureDistance;public Point? RelativePosition;public double CurrentDistanceM;}
public class Graphics {
 public List<object> Ops=new(); public bool NeedsClear;public RenderTarget Target=new();
 public RenderTarget GetRenderTarget()=>Target;
 public Point MeasureString(Font f,string text)=>new((text??"").Length*6,12);
 public void DrawText(Font f,SolidBrush b,float x,float y,string text)=>Ops.Add(new{op="Text",key=b.Key,args=new[]{x,y},text});
 public void DrawCircle(SolidBrush b,float x,float y,float r,float width)=>Ops.Add(new{op="Circle",key=b.Key,args=new[]{x,y,r,width}});
 public void FillCircle(SolidBrush b,float x,float y,float r)=>Ops.Add(new{op="FillCircle",key=b.Key,args=new[]{x,y,r}});
 public void DrawLine(SolidBrush b,float x,float y,float x2,float y2,float w)=>Ops.Add(new{op="Line",key=b.Key,args=new[]{x,y,x2,y2,w}});
 public void FillRectangle(SolidBrush b,float x,float y,float x2,float y2)=>Ops.Add(new{op="FillRectangle",key=b.Key,args=new[]{x,y,x2,y2}});
 public void FillEllipse(SolidBrush b,float x,float y,float rx,float ry)=>Ops.Add(new{op="FillEllipse",key=b.Key,args=new[]{x,y,rx,ry}});
 public void DrawCrosshair(SolidBrush b,float x,float y,float size,float width,CrosshairStyle style)=>Ops.Add(new{op="Crosshair",key=b.Key,args=new[]{x,y,size,width},style=style.ToString()});
 public void DrawRectangleEdges(SolidBrush b,float x,float y,float x2,float y2,float w)=>Ops.Add(new{op="RectangleEdges",key=b.Key,args=new[]{x,y,x2,y2,w}});
}
public class OriginalDraw {
 public bool forceShow,Visible=true,ForceHidden,IsInitialised=true,isUpdating,forceClear;
 public double playerAltitude,csampleDistm,cminDistm=500;public int currentContextID=7,sampleCount,fontsize=12;
 public float gCX=100,gCY=100,gOuterRadius=100,gInnerRadius=76,gMiddleRadius=88,gOffset=100,iconSize=5,fontoffsety=6;
 public Font font=new(); public RadarPosition playerCurrent=new(){Latitude=12.345678,Longitude=-45.678912,Heading=321.6};public BrushHandler brushHandler=new();
 public Dictionary<int,string> Genuses=new(){{7,"Stratum"},{3,"Aleoida"},{9,"Tussock"}};
 public Dictionary<int,int> GenusRange=new(){{7,500},{3,150},{9,200}};public HashSet<int> SeenGenuses=new(){3};public List<RadarPoint> points=new();
public bool Draw(Graphics gfx, bool show)
	{
		bool result = false;
		string text5 = default;
		float num7 = default;
		int num5 = default;
		int current = default;
		string theKey = default;
		string text2 = default;
		float x = default;
		int num20 = default;
		int num23 = default;
		string theKey3 = default;
		float num22 = default;
		float num28 = default;
		Point? relativePosition = default;
		double num29 = default;
		RadarPointType type = default;
		while (true)
		{
			int num = -416782633;
			while (true)
			{
				int num17;
				int count;
				int num21;
				uint num2;
				int num64;
				switch ((num2 = (uint)(num ^ -1610879511)) % 40)
				{
				case 17u:
					break;
				case 38u:
					num17 = (Genuses.Count + 1) / 2;
					goto IL_00db;
				case 32u:
					num = ((int)num2 * -1411675356) ^ 0x4527CF9F;
					continue;
				case 8u:
					text5 = _206D_206D_202A_202E_200E_206A_200D_202C_206F_200F_200E_206C_206C_206C_200E_200F_200C_206F_206E_206F_200D_202D_200F_202E_202E_200E_202E_202B_200F_202B_200E_200D_206E_206E_202D_200D_200D_202B_206D_200C_202E(text5, " (", csampleDistm.ToString("N0"), "M)");
					num = -443296193;
					continue;
				case 23u:
				{
					int num12;
					int num13;
					if (currentContextID <= -1)
					{
						num12 = 2086309145;
						num13 = num12;
					}
					else
					{
						num12 = 1285509744;
						num13 = num12;
					}
					num = num12 ^ (int)(num2 * 2090099448);
					continue;
				}
				case 6u:
				{
					gfx.FillRectangle(brushHandler.Brush("radarbg"), gCX - gOuterRadius - 2f, gCY + gOuterRadius + (float)fontsize + (float)fontsize - 2f, gCX + gOuterRadius + 2f, num7 + 4f);
					Font font2 = this.font;
					SolidBrush brush2 = brushHandler.Brush("radarwhite");
					float x3 = gCX - gOuterRadius;
					float y2 = gCY + gOuterRadius + (float)fontsize + (float)fontsize;
					DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(17, 3);
					defaultInterpolatedStringHandler.AppendLiteral("POSITION: (");
					defaultInterpolatedStringHandler.AppendFormatted(playerCurrent.Latitude.ToString("N4"));
					defaultInterpolatedStringHandler.AppendLiteral(",");
					defaultInterpolatedStringHandler.AppendFormatted(playerCurrent.Longitude.ToString("N4"));
					defaultInterpolatedStringHandler.AppendLiteral(") @ ");
					defaultInterpolatedStringHandler.AppendFormatted(((playerCurrent.Heading + 360.0) % 360.0).ToString("N0"));
					defaultInterpolatedStringHandler.AppendLiteral("°");
					gfx.DrawText(font2, brush2, x3, y2, defaultInterpolatedStringHandler.ToStringAndClear());
					num = -668458618;
					continue;
				}
				case 13u:
				{
					int num59;
					int num60;
					if (forceShow)
					{
						num59 = -2031619912;
						num60 = num59;
					}
					else
					{
						num59 = -90944086;
						num60 = num59;
					}
					num = num59 ^ ((int)num2 * -416452996);
					continue;
				}
				case 7u:
					if (Genuses.Count > 0)
					{
						num = (int)((num2 * 1580951455) ^ 0x5E8EBE4B);
						continue;
					}
					goto IL_0dfe;
				case 1u:
				{
					int num11;
					if (ForceHidden)
					{
						num = -1156940362;
						num11 = num;
					}
					else
					{
						num = -4212551;
						num11 = num;
					}
					continue;
				}
				case 10u:
				{
					Font font3 = this.font;
					SolidBrush brush3 = brushHandler.Brush("radarwhite");
					float x4 = gCX + 2f;
					float y3 = gCY + gMiddleRadius;
					DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(2, 1);
					defaultInterpolatedStringHandler.AppendFormatted(2f);
					defaultInterpolatedStringHandler.AppendLiteral("KM");
					gfx.DrawText(font3, brush3, x4, y3, defaultInterpolatedStringHandler.ToStringAndClear());
					Font font4 = this.font;
					SolidBrush brush4 = brushHandler.Brush("radarwhite");
					float x5 = gCX + 2f;
					float y4 = gCY + gOuterRadius;
					defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(2, 1);
					defaultInterpolatedStringHandler.AppendFormatted(10f);
					defaultInterpolatedStringHandler.AppendLiteral("KM");
					gfx.DrawText(font4, brush4, x5, y4, defaultInterpolatedStringHandler.ToStringAndClear());
					num7 = gCY + gOuterRadius + (float)fontsize + (float)fontsize + (float)fontsize;
					num = -1843741441;
					continue;
				}
				case 21u:
				{
					int num62;
					if (sampleCount != 2)
					{
						num = -1066348395;
						num62 = num;
					}
					else
					{
						num = -1709036076;
						num62 = num;
					}
					continue;
				}
				case 22u:
				{
					int num56;
					int num57;
					if (!IsInitialised)
					{
						num56 = 1261172755;
						num57 = num56;
					}
					else
					{
						num56 = 504772669;
						num57 = num56;
					}
					num = num56 ^ (int)(num2 * 1972090968);
					continue;
				}
				case 2u:
					return result;
				case 14u:
					gfx.DrawText(this.font, brushHandler.Brush("radarcyan"), gCX - gOuterRadius, num7, text5);
					num = -1593404975;
					continue;
				case 18u:
				{
					int num18;
					if (csampleDistm > 0.0)
					{
						num = -144813575;
						num18 = num;
					}
					else
					{
						num = -443296193;
						num18 = num;
					}
					continue;
				}
				case 4u:
				{
					int num16;
					if (csampleDistm > 1000.0)
					{
						num = -877983238;
						num16 = num;
					}
					else
					{
						num = -152616061;
						num16 = num;
					}
					continue;
				}
				case 9u:
				{
					int num9;
					int num10;
					if (Genuses.ContainsKey(currentContextID))
					{
						num9 = 1591356168;
						num10 = num9;
					}
					else
					{
						num9 = 280873049;
						num10 = num9;
					}
					num = num9 ^ (int)(num2 * 167546632);
					continue;
				}
				case 25u:
				{
					text5 = _200B_202A_206B_206C_200C_206C_200B_206E_206E_202A_202D_202D_206D_206E_202C_200C_202C_206E_206D_200F_202B_200C_206F_200C_202E_206C_206D_202E_202D_200E_202B_200F_206A_202C_206B_202D_202D_200F_200C_206A_202E("CURRENT SAMPLE: ", Genuses[currentContextID]);
					int num65;
					if (!GenusRange.ContainsKey(currentContextID))
					{
						num = -559552955;
						num65 = num;
					}
					else
					{
						num = -954312716;
						num65 = num;
					}
					continue;
				}
				case 11u:
					text5 = _206D_206D_202A_202E_200E_206A_200D_202C_206F_200F_200E_206C_206C_206C_200E_200F_200C_206F_206E_206F_200D_202D_200F_202E_202E_200E_202E_202B_200F_202B_200E_200D_206E_206E_202D_200D_200D_202B_206D_200C_202E(text5, " (", (csampleDistm / 1000.0).ToString("N2"), "KM)");
					num = -929910079;
					continue;
				case 5u:
					text5 = _200B_202A_206B_206C_200C_206C_200B_206E_206E_202A_202D_202D_206D_206E_202C_200C_202C_206E_206D_200F_202B_200C_206F_200C_202E_206C_206D_202E_202D_200E_202B_200F_206A_202C_206B_202D_202D_200F_200C_206A_202E(text5, "  ●●○");
					num = -1066348395;
					continue;
				case 29u:
				{
					int num63;
					if (!forceClear)
					{
						num = -2068628280;
						num63 = num;
					}
					else
					{
						num = -640213150;
						num63 = num;
					}
					continue;
				}
				case 15u:
					text5 = _200B_202A_206B_206C_200C_206C_200B_206E_206E_202A_202D_202D_206D_206E_202C_200C_202C_206E_206D_200F_202B_200C_206F_200C_202E_206C_206D_202E_202D_200E_202B_200F_206A_202C_206B_202D_202D_200F_200C_206A_202E(text5, "  ●○○");
					num = -1629468571;
					continue;
				case 36u:
					gfx.NeedsClear = true;
					num = (int)(num2 * 1002508524) ^ -402065000;
					continue;
				case 33u:
					text5 = _200B_202A_206B_206C_200C_206C_200B_206E_206E_202A_202D_202D_206D_206E_202C_200C_202C_206E_206D_200F_202B_200C_206F_200C_202E_206C_206D_202E_202D_200E_202B_200F_206A_202C_206B_202D_202D_200F_200C_206A_202E(text5, "  ○○○");
					num = -1066348395;
					continue;
				case 28u:
					num = (int)((num2 * 1221968974) ^ 0x5ED7703D);
					continue;
				case 30u:
				{
					int num61;
					if (sampleCount != 1)
					{
						num = -1005005572;
						num61 = num;
					}
					else
					{
						num = -1378936146;
						num61 = num;
					}
					continue;
				}
				case 12u:
				{
					int num58;
					if (sampleCount == 0)
					{
						num = -1742189144;
						num58 = num;
					}
					else
					{
						num = -782488737;
						num58 = num;
					}
					continue;
				}
				case 16u:
					gfx.DrawCircle(brushHandler.Brush("radar"), gCX, gCY, gOuterRadius, 2f);
					gfx.DrawLine(brushHandler.Brush("radar"), gCX, gCY - gInnerRadius, gCX, gCY - gInnerRadius + (float)fontsize, 2f);
					gfx.DrawLine(brushHandler.Brush("radar"), gCX, gCY + gInnerRadius, gCX, gCY + gInnerRadius + (float)(fontsize / 2), 1f);
					gfx.DrawLine(brushHandler.Brush("radar"), gCX, gCY + gMiddleRadius, gCX, gCY + gMiddleRadius + (float)(fontsize / 2), 1f);
					num = -265787894;
					continue;
				case 37u:
				{
					string text6 = text5;
					DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(4, 1);
					defaultInterpolatedStringHandler.AppendLiteral(" [");
					defaultInterpolatedStringHandler.AppendFormatted(GenusRange[currentContextID]);
					defaultInterpolatedStringHandler.AppendLiteral("m]");
					text5 = _200B_202A_206B_206C_200C_206C_200B_206E_206E_202A_202D_202D_206D_206E_202C_200C_202C_206E_206D_200F_202B_200C_206F_200C_202E_206C_206D_202E_202D_200E_202B_200F_206A_202C_206B_202D_202D_200F_200C_206A_202E(text6, defaultInterpolatedStringHandler.ToStringAndClear());
					num = -559552955;
					continue;
				}
				case 24u:
					gfx.FillCircle(brushHandler.Brush("radarbg"), gCX, gCY, gInnerRadius);
					gfx.DrawCircle(brushHandler.Brush("radar"), gCX, gCY, gInnerRadius, 2f);
					num = -512365950;
					continue;
				case 26u:
				{
					int num54;
					int num55;
					if (isUpdating)
					{
						num54 = 1935372911;
						num55 = num54;
					}
					else
					{
						num54 = 1430027495;
						num55 = num54;
					}
					num = num54 ^ (int)(num2 * 2064376902);
					continue;
				}
				case 31u:
					return result;
				case 27u:
					gfx.DrawCircle(brushHandler.Brush("radar"), gCX, gCY, gMiddleRadius, 2f);
					num = -1220648463;
					continue;
				case 34u:
				{
					int num19;
					if (playerAltitude > 2000.0)
					{
						num = -1574794772;
						num19 = num;
					}
					else
					{
						num = -255825708;
						num19 = num;
					}
					continue;
				}
				case 19u:
					num5 = 0;
					if (Genuses.Count % 2 == 0)
					{
						num17 = Genuses.Count / 2;
						goto IL_00db;
					}
					num = (int)((num2 * 1407090184) ^ 0x33876ACF);
					continue;
				case 20u:
				{
					int num14;
					int num15;
					if (Visible)
					{
						num14 = 1151028007;
						num15 = num14;
					}
					else
					{
						num14 = 1011405823;
						num15 = num14;
					}
					num = num14 ^ ((int)num2 * -106945073);
					continue;
				}
				case 3u:
				{
					gfx.DrawLine(brushHandler.Brush("radar"), gCX, gCY + gOuterRadius, gCX, gCY + gOuterRadius + (float)(fontsize / 2), 1f);
					Font font = this.font;
					SolidBrush brush = brushHandler.Brush("radarwhite");
					float x2 = gCX + 2f;
					float y = gCY + gInnerRadius;
					DefaultInterpolatedStringHandler defaultInterpolatedStringHandler = new DefaultInterpolatedStringHandler(2, 1);
					defaultInterpolatedStringHandler.AppendFormatted(1f);
					defaultInterpolatedStringHandler.AppendLiteral("KM");
					gfx.DrawText(font, brush, x2, y, defaultInterpolatedStringHandler.ToStringAndClear());
					num = -831411077;
					continue;
				}
				case 39u:
					return result;
				case 35u:
					forceClear = false;
					num = (int)(num2 * 490132740) ^ -1709195655;
					continue;
				default:
					{
						IEnumerator<int> enumerator = (from keyValuePair in Genuses
							orderby keyValuePair.Value
							select keyValuePair.Key).GetEnumerator();
						try
						{
							while (true)
							{
								IL_0cfe:
								int num3;
								int num4;
								if (!_200C_206B_200B_200E_200B_206C_200F_200B_206E_200B_206C_200B_202D_200C_200D_200B_206A_206D_200B_206B_202B_202E_206A_200D_200B_202E_200D_202C_206E_202A_200D_206E_206F_200C_206C_206A_206C_206D_200B_200C_202E((IEnumerator)enumerator))
								{
									num3 = -579387057;
									num4 = num3;
								}
								else
								{
									num3 = -118571256;
									num4 = num3;
								}
								while (true)
								{
									string text;
									string text3;
									int num6;
									switch ((num2 = (uint)(num3 ^ -1610879511)) % 13)
									{
									case 0u:
										num3 = -118571256;
										continue;
									default:
										goto end_IL_0b97;
									case 8u:
										text = "radarwhite";
										goto IL_0c09;
									case 7u:
										num3 = (int)((num2 * 642492520) ^ 0x5C28CF9E);
										continue;
									case 5u:
										if (!SeenGenuses.Contains(current))
										{
											num3 = ((int)num2 * -827005344) ^ -488347506;
											continue;
										}
										text3 = _200B_202A_206B_206C_200C_206C_200B_206E_206E_202A_202D_202D_206D_206E_202C_200C_202C_206E_206D_200F_202B_200C_206F_200C_202E_206C_206D_202E_202D_200E_202B_200F_206A_202C_206B_202D_202D_200F_200C_206A_202E("✔ ", Genuses[current]);
										goto IL_0c87;
									case 10u:
										text3 = Genuses[current];
										goto IL_0c87;
									case 11u:
										current = enumerator.Current;
										num3 = -2029286118;
										continue;
									case 6u:
										gfx.DrawText(this.font, brushHandler.Brush(theKey), gCX - gOuterRadius, num7 + (float)((num5 / 2 + 1) * fontsize), text2);
										num3 = (int)(num2 * 590019876) ^ -644817097;
										continue;
									case 9u:
										break;
									case 1u:
										num5++;
										num3 = -2065085410;
										continue;
									case 4u:
										x = gfx.MeasureString(this.font, text2).X;
										num3 = -608628537;
										continue;
									case 2u:
										gfx.DrawText(this.font, brushHandler.Brush(theKey), gCX + gOuterRadius - x, num7 + (float)((num5 / 2 + 1) * fontsize), text2);
										num3 = (int)(num2 * 1939397087) ^ -226142756;
										continue;
									case 12u:
										if (SeenGenuses.Contains(current))
										{
											text = "radargrey";
											goto IL_0c09;
										}
										num3 = (int)((num2 * 1311461461) ^ 0x25F1FD06);
										continue;
									case 3u:
										goto end_IL_0b97;
										IL_0c09:
										theKey = text;
										if (num5 % 2 != 0)
										{
											num3 = -748918011;
											num6 = num3;
										}
										else
										{
											num3 = -262337070;
											num6 = num3;
										}
										continue;
										IL_0c87:
										text2 = text3;
										if (current == currentContextID)
										{
											text = "radarcyan";
											goto IL_0c09;
										}
										num3 = -1899202470;
										continue;
									}
									goto IL_0cfe;
									continue;
									end_IL_0b97:
									break;
								}
								break;
							}
						}
						finally
						{
							if (enumerator != null)
							{
								while (true)
								{
									IL_0dc3:
									int num8 = -2088670975;
									while (true)
									{
										switch ((num2 = (uint)(num8 ^ -1610879511)) % 3)
										{
										case 2u:
											break;
										default:
											goto end_IL_0dc8;
										case 1u:
											goto IL_0de6;
										case 0u:
											goto end_IL_0dc8;
										}
										goto IL_0dc3;
										IL_0de6:
										_202A_206C_206D_200F_206E_202D_206D_202C_200E_206C_200D_200C_202B_206F_202C_206F_206F_200B_206E_206B_206A_202B_202D_200B_202A_200F_206A_206D_206F_206E_206C_200C_206C_202A_206D_202E_200F_202E_200B_206D_202E((IDisposable)enumerator);
										num8 = ((int)num2 * -1527784358) ^ -835271361;
										continue;
										end_IL_0dc8:
										break;
									}
									break;
								}
							}
						}
						goto IL_0dfe;
					}
					IL_0dfe:
					count = points.Count;
					if (currentContextID > -1)
					{
						goto IL_0e16;
					}
					goto IL_1714;
					IL_1714:
					num20 = 0;
					num21 = -418545890;
					goto IL_0e1b;
					IL_0e1b:
					while (true)
					{
						string text4;
						string theKey2;
						switch ((num2 = (uint)(num21 ^ -1610879511)) % 44)
						{
						case 0u:
							break;
						case 2u:
							_202E_200E_206D_200C_202A_206D_202E_206E_202C_206B_200E_206E_206B_200B_206E_206E_200B_200B_206F_202B_206D_202D_202D_206D_202D_206A_206B_200F_200F_200F_206E_202B_206E_202A_206B_202C_206C_202E_206D_200E_202E(gfx.GetRenderTarget(), AntialiasMode.Aliased);
							num21 = ((int)num2 * -1206226804) ^ 0x79814CDE;
							continue;
						case 30u:
						{
							int num40;
							int num41;
							if (currentContextID == points[num23].ContextID)
							{
								num40 = -1223778926;
								num41 = num40;
							}
							else
							{
								num40 = -1233196010;
								num41 = num40;
							}
							num21 = num40 ^ (int)(num2 * 541074013);
							continue;
						}
						case 10u:
							goto IL_0f37;
						case 37u:
							gfx.DrawCrosshair(brushHandler.Brush(theKey3), gCX, gCY, iconSize, 2f, CrosshairStyle.Gap);
							num21 = -1293066406;
							continue;
						case 28u:
							num21 = ((int)num2 * -790912619) ^ 0x52444149;
							continue;
						case 7u:
							goto IL_0f9c;
						case 8u:
						{
							int num38;
							int num39;
							if (csampleDistm >= cminDistm)
							{
								num38 = -1646414600;
								num39 = num38;
							}
							else
							{
								num38 = -1872621991;
								num39 = num38;
							}
							num21 = num38 ^ (int)(num2 * 272408167);
							continue;
						}
						case 39u:
						{
							int num30;
							int num31;
							if (cminDistm > 0.0)
							{
								num30 = 1114475204;
								num31 = num30;
							}
							else
							{
								num30 = 1202519629;
								num31 = num30;
							}
							num21 = num30 ^ (int)(num2 * 2056841151);
							continue;
						}
						case 14u:
							goto IL_1020;
						case 25u:
						{
							num22 = gInnerRadius * (points[num23].Radius / 1000f / 1f);
							num28 = (gMiddleRadius - gInnerRadius) * (points[num23].Radius / 1000f / 2f);
							int num32;
							int num33;
							if (points[num23].CurrentDistanceM > 2000.0)
							{
								num32 = 344307064;
								num33 = num32;
							}
							else
							{
								num32 = 1559865111;
								num33 = num32;
							}
							num21 = num32 ^ (int)(num2 * 1361477368);
							continue;
						}
						case 26u:
							goto IL_10c8;
						case 5u:
						{
							int num44;
							int num45;
							if (csampleDistm <= 0.0)
							{
								num44 = 1010841157;
								num45 = num44;
							}
							else
							{
								num44 = 918663903;
								num45 = num44;
							}
							num21 = num44 ^ (int)(num2 * 1722767857);
							continue;
						}
						case 21u:
							num22 = num28;
							num21 = ((int)num2 * -1723166334) ^ 0x1DBB537E;
							continue;
						case 15u:
							gfx.DrawText(this.font, brushHandler.Brush("radarwhite"), gCX + relativePosition.Value.X + iconSize + 4f, gCY + relativePosition.Value.Y - fontoffsety, (points[num20].ContextID == 1) ? "SRV" : "SHIP");
							num21 = -1392893883;
							continue;
						case 29u:
							gfx.NeedsClear = true;
							num21 = (int)(num2 * 140517662) ^ -1873464897;
							continue;
						case 42u:
						{
							int num26;
							int num27;
							if (points[num23].CurrentDistanceM < 10000.0)
							{
								num26 = -687252082;
								num27 = num26;
							}
							else
							{
								num26 = -501166658;
								num27 = num26;
							}
							num21 = num26 ^ (int)(num2 * 2111310441);
							continue;
						}
						case 33u:
							goto IL_1210;
						case 13u:
							num23++;
							num21 = -1427496457;
							continue;
						case 6u:
							num22 = (num22 - num28) * (float)num29 + num28;
							num21 = ((int)num2 * -137790740) ^ 0x3C40FC94;
							continue;
						case 16u:
							goto IL_125e;
						case 22u:
							num29 = 1.0 - (points[num23].CurrentDistanceM / 1000.0 - 1.0) / 1.0;
							num21 = ((int)num2 * -1311726856) ^ 0x1E8F3863;
							continue;
						case 24u:
							theKey3 = "radargrey";
							num21 = -2053844532;
							continue;
						case 11u:
							gfx.DrawRectangleEdges(brushHandler.Brush("radarship"), gCX + relativePosition.Value.X - iconSize, gCY + relativePosition.Value.Y - iconSize, gCX + relativePosition.Value.X + iconSize, gCY + relativePosition.Value.Y + iconSize, 4f);
							num21 = -1985587611;
							continue;
						case 17u:
							switch (type)
							{
							case RadarPointType.Waypoint:
								break;
							case RadarPointType.Biological:
								goto IL_1210;
							case RadarPointType.None:
								goto IL_125e;
							default:
								goto IL_1380;
							case RadarPointType.Vehicle:
								goto IL_1482;
							}
							goto IL_0f37;
						case 20u:
						{
							int num42;
							int num43;
							if (points[num20].CurrentDistanceM > 100.0)
							{
								num42 = 626005070;
								num43 = num42;
							}
							else
							{
								num42 = 746354385;
								num43 = num42;
							}
							num21 = num42 ^ (int)(num2 * 622742699);
							continue;
						}
						case 19u:
							goto IL_13cd;
						case 31u:
							gfx.DrawText(this.font, brushHandler.Brush("radarwhite"), gCX + relativePosition.Value.X + iconSize + 2f, gCY + relativePosition.Value.Y - fontoffsety, points[num20].Label);
							num21 = -691167147;
							continue;
						case 23u:
							num21 = ((int)num2 * -1766392811) ^ 0x1232DA00;
							continue;
						case 32u:
							goto IL_1482;
						case 3u:
							_202E_200E_206D_200C_202A_206D_202E_206E_202C_206B_200E_206E_206B_200B_206E_206E_200B_200B_206F_202B_206D_202D_202D_206D_202D_206A_206B_200F_200F_200F_206E_202B_206E_202A_206B_202C_206C_202E_206D_200E_202E(gfx.GetRenderTarget(), AntialiasMode.PerPrimitive);
							num21 = (int)((num2 * 47033719) ^ 0x423185AD);
							continue;
						case 41u:
						{
							int num36;
							int num37;
							if (points[num23].Radius > 0f)
							{
								num36 = 456889136;
								num37 = num36;
							}
							else
							{
								num36 = 376907512;
								num37 = num36;
							}
							num21 = num36 ^ ((int)num2 * -836145148);
							continue;
						}
						case 1u:
						{
							relativePosition = points[num23].RelativePosition;
							int num34;
							int num35;
							if (!relativePosition.HasValue)
							{
								num34 = -1504781174;
								num35 = num34;
							}
							else
							{
								num34 = -258242707;
								num35 = num34;
							}
							num21 = num34 ^ (int)(num2 * 347512622);
							continue;
						}
						case 34u:
							relativePosition = points[num20].RelativePosition;
							type = points[num20].Type;
							num21 = -972341412;
							continue;
						case 18u:
							if (currentContextID < 0)
							{
								num21 = (int)((num2 * 336564227) ^ 0x58201DAC);
								continue;
							}
							text4 = "radar1";
							goto IL_173b;
						case 40u:
						{
							int num24;
							int num25;
							if (points[num20].ContextID != currentContextID)
							{
								num24 = -2117276374;
								num25 = num24;
							}
							else
							{
								num24 = -1310414789;
								num25 = num24;
							}
							num21 = num24 ^ (int)(num2 * 1502051806);
							continue;
						}
						case 43u:
							gfx.DrawCrosshair(brushHandler.Brush("radar2"), gCX + relativePosition.Value.X, gCY + relativePosition.Value.Y, iconSize, 2f, CrosshairStyle.Plus);
							gfx.DrawText(this.font, brushHandler.Brush("radarwhite"), gCX + relativePosition.Value.X + iconSize + 2f, gCY + relativePosition.Value.Y - fontoffsety, points[num20].Label);
							num21 = -691167147;
							continue;
						case 36u:
							num23 = 0;
							num21 = (int)((num2 * 412676271) ^ 0x4A39967B);
							continue;
						case 38u:
							goto IL_1692;
						case 9u:
							gfx.FillEllipse(brushHandler.Brush("radarbgdark"), gCX + relativePosition.Value.X, gCY + relativePosition.Value.Y, num22, num22);
							num21 = -747716164;
							continue;
						case 35u:
							goto IL_1714;
						case 27u:
							text4 = "radar0";
							goto IL_173b;
						case 4u:
							num21 = (int)((num2 * 1783828989) ^ 0x195AD0E1);
							continue;
						default:
							{
								return true;
							}
							IL_173b:
							theKey2 = text4;
							gfx.DrawCrosshair(brushHandler.Brush(theKey2), gCX + relativePosition.Value.X, gCY + relativePosition.Value.Y, iconSize, 2f, CrosshairStyle.Cross);
							num21 = -118602622;
							continue;
							IL_1380:
							num21 = ((int)num2 * -209618115) ^ -544930292;
							continue;
							IL_125e:
							num20++;
							num21 = -934418365;
							continue;
						}
						break;
						IL_1692:
						int num46;
						if (points[num23].CurrentDistanceM > 1000.0)
						{
							num21 = -18215257;
							num46 = num21;
						}
						else
						{
							num21 = -1065250964;
							num46 = num21;
						}
						continue;
						IL_1020:
						int num47;
						if (num20 < count)
						{
							num21 = -1654168905;
							num47 = num21;
						}
						else
						{
							num21 = -195371965;
							num47 = num21;
						}
						continue;
						IL_1210:
						int num48;
						if (relativePosition.HasValue)
						{
							num21 = -764942443;
							num48 = num21;
						}
						else
						{
							num21 = -691167147;
							num48 = num21;
						}
						continue;
						IL_13cd:
						int num49;
						if (points[num23].Type != RadarPointType.Biological)
						{
							num21 = -747716164;
							num49 = num21;
						}
						else
						{
							num21 = -354602917;
							num49 = num21;
						}
						continue;
						IL_10c8:
						int num50;
						if (num23 >= count)
						{
							num21 = -1823611550;
							num50 = num21;
						}
						else
						{
							num21 = -99552918;
							num50 = num21;
						}
						continue;
						IL_0f9c:
						theKey3 = "radarcyan";
						int num51;
						if (currentContextID < 0)
						{
							num21 = -2053844532;
							num51 = num21;
						}
						else
						{
							num21 = -594812176;
							num51 = num21;
						}
						continue;
						IL_1482:
						int num52;
						if (!relativePosition.HasValue)
						{
							num21 = -691167147;
							num52 = num21;
						}
						else
						{
							num21 = -2113344910;
							num52 = num21;
						}
						continue;
						IL_0f37:
						int num53;
						if (!relativePosition.HasValue)
						{
							num21 = -691167147;
							num53 = num21;
						}
						else
						{
							num21 = -563908942;
							num53 = num21;
						}
					}
					goto IL_0e16;
					IL_0e16:
					num21 = -1267957283;
					goto IL_0e1b;
					IL_00db:
					num64 = num17 + 1;
					gfx.FillRectangle(brushHandler.Brush("radarbg"), gCX - gOuterRadius - 2f, num7 + 4f, gCX + gOuterRadius + 2f, num7 + (float)(fontsize * num64) + (float)(fontsize / 2));
					num = -1010917650;
					continue;
				}
				break;
			}
		}
	}
static string _200B_202A_206B_206C_200C_206C_200B_206E_206E_202A_202D_202D_206D_206E_202C_200C_202C_206E_206D_200F_202B_200C_206F_200C_202E_206C_206D_202E_202D_200E_202B_200F_206A_202C_206B_202D_202D_200F_200C_206A_202E(string P_0, string P_1)
	{
		return P_0 + P_1;
	}
static bool _200C_206B_200B_200E_200B_206C_200F_200B_206E_200B_206C_200B_202D_200C_200D_200B_206A_206D_200B_206B_202B_202E_206A_200D_200B_202E_200D_202C_206E_202A_200D_206E_206F_200C_206C_206A_206C_206D_200B_200C_202E(IEnumerator P_0)
	{
		return P_0.MoveNext();
	}
static void _202A_206C_206D_200F_206E_202D_206D_202C_200E_206C_200D_200C_202B_206F_202C_206F_206F_200B_206E_206B_206A_202B_202D_200B_202A_200F_206A_206D_206F_206E_206C_200C_206C_202A_206D_202E_200F_202E_200B_206D_202E(IDisposable P_0)
	{
		P_0.Dispose();
	}
static void _202E_200E_206D_200C_202A_206D_202E_206E_202C_206B_200E_206E_206B_200B_206E_206E_200B_200B_206F_202B_206D_202D_202D_206D_202D_206A_206B_200F_200F_200F_206E_202B_206E_202A_206B_202C_206C_202E_206D_200E_202E(RenderTarget P_0, AntialiasMode P_1)
	{
		P_0.AntialiasMode = P_1;
	}
static string _206D_206D_202A_202E_200E_206A_200D_202C_206F_200F_200E_206C_206C_206C_200E_200F_200C_206F_206E_206F_200D_202D_200F_202E_202E_200E_202E_202B_200F_202B_200E_200D_206E_206E_202D_200D_200D_202B_206D_200C_202E(string P_0, string P_1, string P_2, string P_3)
	{
		return P_0 + P_1 + P_2 + P_3;
	}
}

public static class Program {
 public static void Main(){
  CultureInfo.CurrentCulture=CultureInfo.GetCultureInfo("en-US");var cases=new List<object>();int id=0;
  for(int mask=0;mask<32;mask++) foreach(double altitude in new[]{0.0,2000,2001}) {
    var r=new OriginalDraw {forceShow=(mask&1)!=0,Visible=(mask&2)!=0,ForceHidden=(mask&4)!=0,IsInitialised=(mask&8)!=0,isUpdating=(mask&16)!=0,playerAltitude=altitude};
    foreach(bool show in new[]{false,true}) {var g=new Graphics();bool result=r.Draw(g,show);cases.Add(new{id=id++,kind="visibility",mask,altitude,show,result,ops=g.Ops});}
  }
  foreach(int count in new[]{0,1,2,3}) foreach(double distance in new[]{0.0,1,499,500,999,1000,1001,12345.6}) {
    var r=new OriginalDraw{sampleCount=count,csampleDistm=distance};var g=new Graphics();bool result=r.Draw(g,true);cases.Add(new{id=id++,kind="text",count,distance,result,ops=g.Ops});
  }
  foreach(int context in new[]{-1,7,9,99}) foreach(double distance in new[]{0.0,99,100,101,499,500,999,1000,1001,1999,2000,2001,10000,19999,20000}) {
    var r=new OriginalDraw{currentContextID=context,csampleDistm=distance,sampleCount=1};
    r.points=new(){new(){Type=RadarPointType.Vehicle,ContextID=1,RelativePosition=new Point(20,30),Label="ignored vehicle",CurrentDistanceM=distance},new(){Type=RadarPointType.Biological,ContextID=7,Radius=500,RelativePosition=new Point(-10,-40),Label="Stratum",CurrentDistanceM=distance},new(){Type=RadarPointType.Waypoint,ContextID=0,RelativePosition=new Point(60,-20),Label="WP1",CurrentDistanceM=distance}};
    var g=new Graphics();bool result=r.Draw(g,true);cases.Add(new{id=id++,kind="points",context,distance,result,ops=g.Ops});
  }
  Console.Write(JsonSerializer.Serialize(new{source="StellarOverlay 1.3.228.387 Radar.Draw",measure_stub="6px per UTF-16 code unit,12px height",cases}));
 }
}
