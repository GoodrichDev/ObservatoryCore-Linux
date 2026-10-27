// Actual AddNotificationItem source body. Only RVA trim constant substitution and data/graphics stubs are authored.
using System;using System.Linq;using System.Collections.Generic;using System.Runtime.CompilerServices;using System.Text.Json;
public enum NotificationTransition{None,Left,Right,Fade}
public enum NotificationItemShape{Closed,Open}
public enum Alignment{Left,Center,Right}
public enum ScreenReferenceDirection{None,Left,Right,Up,Down,Center,Proportional}
public class Point{public float X{get;set;}public float Y{get;set;}public Point(float x,float y){X=x;Y=y;}}
public class Font{public float FontSize;public Font(float size){FontSize=size;}}
public class Graphics{public bool NeedsClear;public List<object> Ops=new();public void DrawLine(SolidBrush b,float x,float y,float x2,float y2,float stroke){Ops.Add(new {op="Line",key=b.Key,args=new[]{x,y,x2,y2,stroke}});}public void DrawRoundedRectangle(SolidBrush b,float x,float y,float x2,float y2,float radius,float stroke){Ops.Add(new{op="Rectangle",key=b.Key,args=new[]{x,y,x2,y2,radius,stroke}});}public void FillRoundedRectangle(SolidBrush b,float x,float y,float x2,float y2,float radius){Ops.Add(new{op="Fill",key=b.Key,args=new[]{x,y,x2,y2,radius}});}public void DrawText(Font f,float size,SolidBrush b,float x,float y,string text){Ops.Add(new{op="Text",key=b.Key,args=new[]{x,y,size},text});}public int Width=1920,Height=1080;public Point MeasureString(Font f,float size,string text){if(string.IsNullOrEmpty(text))return new Point(0,0);var lines=text.Replace("\r\n","\n").Split('\n');return new Point(lines.Max(l=>l.Length)*size*.5f,lines.Length*size);}}
public class NotificationArgs{public string Title="",Detail="",Sender="Test";public int Timeout=-1;public Guid? Guid;public double XPos=-1,YPos=-1;}
public class ScreenReferencePosition{public int X{get;set;}public int Y{get;set;}public ScreenReferenceDirection Horizontal{get;set;}public ScreenReferenceDirection Vertical{get;set;}public ScreenReferencePosition(int x,int y,ScreenReferenceDirection h,ScreenReferenceDirection v){X=x;Y=y;Horizontal=h;Vertical=v;}}
public class NotificationItemLine{public List<int> Length=new();public List<string> ColourKey=new();public bool HasColours=>ColourKey.Count>0;public string Text{get;set;}public float FontSize{get;set;}public bool IsBold{get;set;}public int OffsetY{get;set;}public Alignment Alignment{get;set;}public NotificationItemLine(string text,float fontSize,bool isBold,int offsetY,Alignment alignment){Text=text;FontSize=fontSize;IsBold=isBold;OffsetY=offsetY;Alignment=alignment;}}
public class SolidBrush{public string Key;public SolidBrush(string key){Key=key;}}public class BrushHandler{public bool HasBrush(string key){return true;}public SolidBrush Brush(string key){return new SolidBrush(key);}}
public class NotificationItem{public float PositionX=12,PositionY=34;public string BackgroundColorKey="notification_background",BorderColorKey="notification_border",TextColorKey="notification_text",TitleColorKey="notification_title";public int BorderWidth=3;public NotificationItemShape shape=NotificationItemShape.Open;public void Draw(Graphics gfx, NotificationQueue queue)
	{
		if (!queue.BrushHandler.HasBrush(BackgroundColorKey))
		{
			return;
		}
		int num7 = default;
		string text2 = default;
		int num8 = default;
		int num10 = default;
		int num13 = default;
		Point point2 = default;
		float num9 = default;
		Point point = default;
		while (true)
		{
			int num = -210638006;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -185094311)) % 12)
				{
				case 9u:
					break;
				case 3u:
				{
					int num18;
					int num19;
					if (!queue.BrushHandler.HasBrush(BorderColorKey))
					{
						num18 = -1915460789;
						num19 = num18;
					}
					else
					{
						num18 = -1525750724;
						num19 = num18;
					}
					num = num18 ^ (int)(num2 * 147057840);
					continue;
				}
				case 11u:
					gfx.DrawLine(queue.BrushHandler.Brush(BorderColorKey), PositionX, PositionY + Dimensions.Y, PositionX + Dimensions.X, PositionY + Dimensions.Y, BorderWidth);
					num = ((int)num2 * -66610241) ^ 0x680E62E6;
					continue;
				case 7u:
				{
					int num20;
					int num21;
					if (BorderWidth <= 0)
					{
						num20 = 1328040149;
						num21 = num20;
					}
					else
					{
						num20 = 1066794710;
						num21 = num20;
					}
					num = num20 ^ (int)(num2 * 1345603222);
					continue;
				}
				case 1u:
				{
					num7 = 0;
					int num15;
					if (shape != NotificationItemShape.Open)
					{
						num = -696933947;
						num15 = num;
					}
					else
					{
						num = -417725083;
						num15 = num;
					}
					continue;
				}
				case 5u:
					gfx.DrawRoundedRectangle(queue.BrushHandler.Brush(BorderColorKey), PositionX, PositionY, PositionX + Dimensions.X, PositionY + Dimensions.Y, NotificationQueue.CornerRounding, BorderWidth);
					num = (int)((num2 * 1207303526) ^ 0x77977DD9);
					continue;
				case 10u:
				{
					int num16;
					int num17;
					if (BorderWidth <= 0)
					{
						num16 = 2058605669;
						num17 = num16;
					}
					else
					{
						num16 = 1270030983;
						num17 = num16;
					}
					num = num16 ^ ((int)num2 * -1894458035);
					continue;
				}
				case 8u:
					gfx.DrawLine(queue.BrushHandler.Brush(BorderColorKey), PositionX, PositionY, PositionX + Dimensions.X, PositionY, BorderWidth);
					num = (int)(num2 * 2089938100) ^ -1546012986;
					continue;
				case 6u:
					return;
				case 0u:
					gfx.FillRoundedRectangle(queue.BrushHandler.Brush(BackgroundColorKey), PositionX, PositionY, PositionX + Dimensions.X, PositionY + Dimensions.Y, 0f);
					num = (int)((num2 * 1165039092) ^ 0x56CAB573);
					continue;
				case 4u:
					gfx.FillRoundedRectangle(queue.BrushHandler.Brush(BackgroundColorKey), PositionX, PositionY, PositionX + Dimensions.X, PositionY + Dimensions.Y, NotificationQueue.CornerRounding);
					num = -1156985942;
					continue;
				default:
				{
					using (List<NotificationItemLine>.Enumerator enumerator = Lines.GetEnumerator())
					{
						while (enumerator.MoveNext())
						{
							while (true)
							{
								NotificationItemLine current = enumerator.Current;
								int num3 = -191932182;
								while (true)
								{
									switch ((num2 = (uint)(num3 ^ -185094311)) % 11)
									{
									case 7u:
										num3 = -253709696;
										continue;
									case 3u:
										break;
									case 9u:
									{
										int num11;
										int num12;
										if (current.HasColours)
										{
											num11 = 1253568510;
											num12 = num11;
										}
										else
										{
											num11 = 1893207109;
											num12 = num11;
										}
										num3 = num11 ^ ((int)num2 * -1036004311);
										continue;
									}
									case 4u:
										num3 = (int)(num2 * 241330691) ^ -866660662;
										continue;
									case 1u:
										goto IL_0397;
									case 10u:
										text2 = _206D_206A_202E_200E_206E_200C_206C_206A_202C_206F_206E_206F_202E_206F_206E_200B_202D_200C_202D_206C_200F_200D_200D_206A_200E_202B_206D_200B_200F_200D_200B_206D_200B_206F_202B_202A_202B_202B_206D_202E_202E(current.Text, num8, current.Length[num10]);
										num13 = (int)gfx.MeasureString(current.IsBold ? queue.Bold : queue.Normal, current.FontSize, ";").X;
										point2 = gfx.MeasureString(current.IsBold ? queue.Bold : queue.Normal, current.FontSize, text2);
										num3 = -826358670;
										continue;
									case 6u:
										num9 += point2.X + (float)num13;
										num8 += current.Length[num10];
										num10++;
										num3 = ((int)num2 * -10136289) ^ 0x1759E6B2;
										continue;
									case 2u:
										gfx.DrawText(current.IsBold ? queue.Bold : queue.Normal, current.FontSize, queue.BrushHandler.Brush(current.ColourKey[num10]), PositionX + (float)num7 + (float)NotificationQueue.PaddingX + num9, PositionY + (float)NotificationQueue.PaddingY + (float)current.OffsetY, text2);
										num3 = -505199436;
										continue;
									case 5u:
										num8 = 0;
										num9 = 0f;
										num10 = 0;
										num3 = (int)(num2 * 1744990888) ^ -867869408;
										continue;
									default:
									{
										List<string> list = _202B_206C_200C_206C_202D_206C_202B_202A_206F_202D_206E_206C_200D_206A_202C_200C_200F_206D_200C_200C_200E_202E_202A_200D_200E_206D_206C_206A_202A_202D_202A_202B_206E_206D_206A_200C_200F_200E_206B_206D_202E(current.Text, "\n", StringSplitOptions.None).ToList();
										float num4 = PositionY + (float)NotificationQueue.PaddingY + (float)current.OffsetY;
										using (List<string>.Enumerator enumerator2 = list.GetEnumerator())
										{
											while (true)
											{
												IL_0698:
												int num5;
												int num6;
												if (!enumerator2.MoveNext())
												{
													num5 = -2031720110;
													num6 = num5;
												}
												else
												{
													num5 = -584177459;
													num6 = num5;
												}
												while (true)
												{
													switch ((num2 = (uint)(num5 ^ -185094311)) % 5)
													{
													case 2u:
														num5 = -584177459;
														continue;
													default:
														goto end_IL_0546;
													case 1u:
													{
														string current2 = enumerator2.Current;
														char[] array = new char[3];
														array[0]='\r';array[1]=' ';array[2]='\t';
														string text = _200E_200F_206A_206F_202E_200E_202B_202B_202B_200D_200B_200F_200B_206C_202D_202E_202E_200B_206E_202E_206E_206F_202D_200D_200E_206A_206C_200C_206F_202C_200F_200D_206D_200C_202E_202D_202B_200C_202C_202B_202E(current2, array);
														point = gfx.MeasureString(current.IsBold ? queue.Bold : queue.Normal, current.FontSize, text);
														Graphics graphics = gfx;
														Font font = (current.IsBold ? queue.Bold : queue.Normal);
														float fontSize = current.FontSize;
														SolidBrush brush = (current.IsBold ? queue.BrushHandler.Brush(TitleColorKey) : queue.BrushHandler.Brush(TextColorKey));
														float x;
														if (current.Alignment != Alignment.Left)
														{
															x = ((current.Alignment == Alignment.Right) ? (PositionX + Dimensions.X - (float)num7 - (float)NotificationQueue.PaddingX - point.X) : (PositionX + (Dimensions.X - point.X) / 2f));
														}
														else
														{
															x = PositionX + (float)num7 + (float)NotificationQueue.PaddingX;
														}
														graphics.DrawText(font, fontSize, brush, x, num4, text);
														num5 = -1548907601;
														continue;
													}
													case 4u:
														num4 += point.Y;
														num5 = (int)((num2 * 305167565) ^ 0x4618738D);
														continue;
													case 3u:
														break;
													case 0u:
														goto end_IL_0546;
													}
													goto IL_0698;
													continue;
													end_IL_0546:
													break;
												}
												break;
											}
										}
										goto end_IL_033b;
									}
									case 0u:
										goto end_IL_033b;
									}
									break;
									IL_0397:
									int num14;
									if (num10 >= current.ColourKey.Count)
									{
										num3 = -1848593338;
										num14 = num3;
									}
									else
									{
										num3 = -654947570;
										num14 = num3;
									}
								}
								continue;
								end_IL_033b:
								break;
							}
						}
					}
					gfx.NeedsClear = true;
					return;
				}
				}
				break;
			}
		}
	}
	static string _200E_200F_206A_206F_202E_200E_202B_202B_202B_200D_200B_200F_200B_206C_202D_202E_202E_200B_206E_202E_206E_206F_202D_200D_200E_206A_206C_200C_206F_202C_200F_200D_206D_200C_202E_202D_202B_200C_202C_202B_202E(string P_0, char[] P_1)
	{
		return P_0.TrimEnd(P_1);
	}
	static string[] _202B_206C_200C_206C_202D_206C_202B_202A_206F_202D_206E_206C_200D_206A_202C_200C_200F_206D_200C_200C_200E_202E_202A_200D_200E_206D_206C_206A_202A_202D_202A_202B_206E_206D_206A_200C_200F_200E_206B_206D_202E(string P_0, string P_1, StringSplitOptions P_2)
	{
		return P_0.Split(P_1, P_2);
	}
	static string _206D_206A_202E_200E_206E_200C_206C_206A_202C_206F_206E_206F_202E_206F_206E_200B_202D_200C_202D_206C_200F_200D_200D_206A_200E_202B_206D_200B_200F_200D_200B_206D_200B_206F_202B_202A_202B_202B_206D_202E_202E(string P_0, int P_1, int P_2)
	{
		return P_0.Substring(P_1, P_2);
	}public List<NotificationItemLine> Lines{get;set;}public Point Dimensions{get;set;}public long TTLms{get;set;}public ScreenReferencePosition Position{get;set;}public NotificationItem(List<NotificationItemLine> lines,Point dimensions,long ttl,ScreenReferencePosition position,NotificationTransition i,NotificationTransition o,int border,NotificationItemShape shape,string sender,BrushHandler brushes){Lines=lines;Dimensions=dimensions;TTLms=ttl;Position=position;}public void Update(int width,int height){}}
public class NotificationQueue{public int verticalPositionPercent=97;public bool fromRight=true;public float FontHeight;public bool ReinitialiseFont;public static int NotificationSpacing=15,EdgeIndent=20;public List<ScreenReferencePosition> ReferencePosition=new();public Font Bold=new Font(14),Normal=new Font(12);public float FontSize=12;public float MinimumFontSize=>FontSize*.75f;public int MaxWidth=480,DefaultTTL=5000,BorderWidth=3;public Alignment TextAlignment=Alignment.Left;public static int PaddingX=10,PaddingY=5,CornerRounding=20,ExtraBottomPadding=5,FontVerticalSpacing=0;public NotificationItemShape DefaultShape=NotificationItemShape.Open;public BrushHandler BrushHandler=new();public List<NotificationItem> QueueItems=new();public Dictionary<Guid,NotificationItem> InstantItems=new();
public void AddNotificationItem(Graphics gfx, NotificationArgs item, ScreenReferencePosition? refpos, NotificationTransition intrans, NotificationTransition outtrans)
	{
		List<NotificationItemLine> list = new List<NotificationItemLine>();
		float num = Bold.FontSize;
		Point point2 = default;
		List<string> list2 = default;
		string text2 = default;
		string text = default;
		float num6 = default;
		float num11 = default;
		float num5 = default;
		Point point = default;
		Guid value = default;
		NotificationItem notificationItem = default;
		Guid? guid = default;
		Guid empty = default;
		int num12 = default;
		while (true)
		{
			int num2 = -5599962;
			while (true)
			{
				uint num3;
				int num14;
				int num27;
				switch ((num3 = (uint)(num2 ^ -821735261)) % 49)
				{
				case 28u:
					break;
				default:
					return;
				case 15u:
				{
					string title = item.Title;
					char[] array2 = new char[3];
					array2[0]=' ';array2[1]='\r';array2[2]='\n';
					item.Title = _200E_206A_200E_206A_202C_202D_206B_202D_200E_206B_202E_202D_202B_206D_202B_200D_206B_200B_206A_202D_202B_206C_202B_202A_202B_202D_202D_202D_206C_200F_206E_206F_202B_200E_206A_206F_200C_200C_200B_202D_202E(title, array2);
					num2 = ((int)num3 * -1127435358) ^ 0x539CAFF3;
					continue;
				}
				case 7u:
				{
					string detail = item.Detail;
					char[] array = new char[3];
					array[0]=' ';array[1]='\r';array[2]='\n';
					item.Detail = _200E_206A_200E_206A_202C_202D_206B_202D_200E_206B_202E_202D_202B_206D_202B_200D_206B_200B_206A_202D_202B_206C_202B_202A_202B_202D_202D_202D_206C_200F_206E_206F_202B_200E_206A_206F_200C_200C_200B_202D_202E(detail, array);
					point2 = gfx.MeasureString(Bold, num, item.Title);
					num2 = ((int)num3 * -838826790) ^ -140355114;
					continue;
				}
				case 46u:
				{
					int num21;
					int num22;
					if (list2.Count <= 0)
					{
						num21 = 727155898;
						num22 = num21;
					}
					else
					{
						num21 = 488297668;
						num22 = num21;
					}
					num2 = num21 ^ (int)(num3 * 2132842423);
					continue;
				}
				case 25u:
					text2 = _200B_206E_206E_206A_206A_200C_206E_206C_202C_206C_200E_202A_206E_202B_202B_206E_202B_206D_200F_202E_200F_202D_202B_202A_206F_202D_206B_206B_202E_206A_200F_200E_200F_202B_206B_206F_202D_202E_200C_202E(text2, "\r\n", text);
					num2 = -299096089;
					continue;
				case 42u:
					text = "";
					num2 = ((int)num3 * -1349588016) ^ 0xB1E952B;
					continue;
				case 19u:
				{
					int num32;
					int num33;
					if (item.XPos == -1.0)
					{
						num32 = -760085148;
						num33 = num32;
					}
					else
					{
						num32 = -1787918389;
						num33 = num32;
					}
					num2 = num32 ^ ((int)num3 * -262095758);
					continue;
				}
				case 5u:
				{
					int num24;
					if (!_206B_200E_206B_206A_206F_206D_202A_200F_200B_200E_202E_202E_206F_206B_200B_200B_206F_200B_206E_206E_206B_202D_206F_202D_206F_202B_202B_202E_200D_206E_202B_202C_202D_200F_202E_202B_202B_200C_202E_202B_202E(text2, "\r\n"))
					{
						num2 = -274496975;
						num24 = num2;
					}
					else
					{
						num2 = -1704586808;
						num24 = num2;
					}
					continue;
				}
				case 27u:
				{
					int num9;
					int num10;
					if (num6 <= (float)MaxWidth)
					{
						num9 = -1745875979;
						num10 = num9;
					}
					else
					{
						num9 = -642126538;
						num10 = num9;
					}
					num2 = num9 ^ ((int)num3 * -1418102204);
					continue;
				}
				case 13u:
					list2 = _206A_200D_202D_202C_200B_200E_202D_200E_206A_206F_200B_202E_200B_206E_202A_202B_202D_200B_206E_206C_202A_202E_206F_206D_206B_202A_200E_202B_202D_202E_206D_206B_202B_206F_206A_206A_200C_206B_202B_202E_202E(text2, ' ', StringSplitOptions.None).ToList();
					text2 = "";
					num2 = ((int)num3 * -46727064) ^ 0x647718BA;
					continue;
				case 48u:
					num11 += (float)ExtraBottomPadding;
					num2 = (int)((num3 * 1113090995) ^ 0x8C8C5FC);
					continue;
				case 17u:
				{
					int num30 = (int)((double)(gfx.Width / 100) * item.XPos);
					int num31 = (int)((double)(gfx.Height / 100) * item.YPos);
					int x = num30 - (int)((double)(num6 / 100f) * item.XPos);
					num31 -= (int)((double)(num11 / 100f) * item.YPos);
					refpos = new ScreenReferencePosition(x, num31, ScreenReferenceDirection.Right, ScreenReferenceDirection.Down);
					num2 = (int)((num3 * 1489870832) ^ 0x4C4AAE70);
					continue;
				}
				case 43u:
					num5--;
					num2 = -1704586808;
					continue;
				case 32u:
					num6 = _200C_206B_206C_200C_200B_200D_202E_206A_202C_206F_202B_200C_200B_206A_202D_202C_206E_202D_202A_200D_200D_206C_202B_202D_202E_200C_202D_206B_206B_200D_202C_206B_206E_202E_206C_202E_200B_206D_202D_206C_202E(num6, (float)(gfx.Width / 12));
					list.Add(new NotificationItemLine(text2, num5, isBold: false, (int)(point2.Y + (float)FontVerticalSpacing), TextAlignment));
					if (item.Timeout != -1)
					{
						num2 = (int)(num3 * 1094348108) ^ -145975866;
						continue;
					}
					num14 = DefaultTTL;
					goto IL_06c8;
				case 0u:
					text = _206E_206D_206C_200B_206B_200B_202C_200B_200B_206F_202B_202D_206A_200C_200C_202D_200E_200F_202E_200F_206C_200F_206F_200C_202B_200C_206B_202A_200F_202B_200D_202E_206B_200C_200C_202B_206E_206F_206D_200D_202E(" ", list2.ToArray());
					num2 = -343085131;
					continue;
				case 1u:
					num2 = (int)(num3 * 375517765) ^ -768093368;
					continue;
				case 14u:
				{
					int num17;
					int num18;
					if (item.YPos == -1.0)
					{
						num17 = 708124924;
						num18 = num17;
					}
					else
					{
						num17 = 53263;
						num18 = num17;
					}
					num2 = num17 ^ (int)(num3 * 1803784281);
					continue;
				}
				case 33u:
					num5 = Normal.FontSize;
					num6 = gfx.MeasureString(Normal, num5, text2).X;
					num2 = ((int)num3 * -668737943) ^ -1391056436;
					continue;
				case 16u:
				{
					float num19 = num6;
					_ = CornerRounding;
					num6 = num19 + (float)(0 + PaddingX * 2);
					num11 = (float)(PaddingY * 2 + FontVerticalSpacing) + point.Y + point2.Y;
					num2 = (int)((num3 * 1328452754) ^ 0x2A917FB7);
					continue;
				}
				case 34u:
					text2 = item.Detail;
					num2 = ((int)num3 * -811041600) ^ -1376821369;
					continue;
				case 31u:
				{
					int num7;
					int num8;
					if (InstantItems.ContainsKey(value))
					{
						num7 = 1789430738;
						num8 = num7;
					}
					else
					{
						num7 = 813633961;
						num8 = num7;
					}
					num2 = num7 ^ (int)(num3 * 605825266);
					continue;
				}
				case 39u:
					num2 = (int)(num3 * 1127542812) ^ -240908792;
					continue;
				case 36u:
					text = "";
					num2 = (int)((num3 * 2044280949) ^ 0x24537C27);
					continue;
				case 18u:
				{
					notificationItem.Update(gfx.Width, gfx.Height);
					int num28;
					int num29;
					if (!item.Guid.HasValue)
					{
						num28 = -25078877;
						num29 = num28;
					}
					else
					{
						num28 = -1124072697;
						num29 = num28;
					}
					num2 = num28 ^ ((int)num3 * -1853306139);
					continue;
				}
				case 29u:
					QueueItems.Add(notificationItem);
					num2 = -2054924399;
					continue;
				case 3u:
					guid = item.Guid;
					empty = Guid.Empty;
					num2 = (int)(num3 * 817176640) ^ -2076293935;
					continue;
				case 22u:
					InstantItems[value] = notificationItem;
					num2 = ((int)num3 * -1400022469) ^ -1084637047;
					continue;
				case 21u:
				{
					int num25;
					int num26;
					if (!(gfx.MeasureString(Normal, num5, text2).X <= (float)MaxWidth))
					{
						num25 = -1259450595;
						num26 = num25;
					}
					else
					{
						num25 = -819344504;
						num26 = num25;
					}
					num2 = num25 ^ (int)(num3 * 1954804725);
					continue;
				}
				case 2u:
				case 40u:
					value = item.Guid.Value;
					num2 = -1229836535;
					continue;
				case 38u:
					num2 = (int)((num3 * 244005144) ^ 0x4B842878);
					continue;
				case 8u:
					list.Add(new NotificationItemLine(item.Title, num, isBold: true, 0, TextAlignment));
					num2 = -298881709;
					continue;
				case 35u:
					text2 = _200B_206E_206E_206A_206A_200C_206E_206C_202C_206C_200E_202A_206E_202B_202B_206E_202B_206D_200F_202E_200F_202D_202B_202A_206F_202D_206B_206B_202E_206A_200F_200E_200F_202B_206B_206F_202D_202E_200C_202E(text2, _206D_206A_206F_200E_200B_202C_202A_206F_202B_206D_202D_200E_206E_202D_202C_202E_200F_200D_202E_206A_206C_200B_206C_200E_206C_202E_206F_202D_206E_200B_206C_202C_200B_200E_202D_202C_206A_206E_202A_202A_202E(text2, "") ? "" : " ", list2[0]);
					list2.RemoveAt(0);
					num2 = -1353411606;
					continue;
				case 6u:
				{
					int num23;
					if (num5 < MinimumFontSize)
					{
						num2 = -1479145815;
						num23 = num2;
					}
					else
					{
						num2 = -644271010;
						num23 = num2;
					}
					continue;
				}
				case 41u:
				{
					int num20;
					if (num >= MinimumFontSize)
					{
						num2 = -2133361022;
						num20 = num2;
					}
					else
					{
						num2 = -1571353432;
						num20 = num2;
					}
					continue;
				}
				case 23u:
				{
					int num15;
					int num16;
					if (!(point2.X <= (float)MaxWidth))
					{
						num15 = -1840537998;
						num16 = num15;
					}
					else
					{
						num15 = -1565658267;
						num16 = num15;
					}
					num2 = num15 ^ (int)(num3 * 1451516973);
					continue;
				}
				case 9u:
					return;
				case 24u:
					num14 = item.Timeout;
					goto IL_06c8;
				case 4u:
					refpos = new ScreenReferencePosition(100, 100, ScreenReferenceDirection.Right, ScreenReferenceDirection.Down);
					num2 = -324743760;
					continue;
				case 47u:
				{
					int num13;
					if (gfx.MeasureString(Normal, num5, text2).X < num6 / 2f)
					{
						num2 = -306896214;
						num13 = num2;
					}
					else
					{
						num2 = -1127000107;
						num13 = num2;
					}
					continue;
				}
				case 45u:
					notificationItem = new NotificationItem(list, new Point(num6, num11), num12, refpos, intrans, outtrans, BorderWidth, DefaultShape, item.Sender, BrushHandler);
					num2 = -1459603158;
					continue;
				case 11u:
					return;
				case 20u:
					point2 = gfx.MeasureString(Bold, num, item.Title);
					num2 = (int)(num3 * 1105064563) ^ -216928790;
					continue;
				case 37u:
					if (guid.HasValue)
					{
						if (guid.GetValueOrDefault() == empty)
						{
							num2 = -815476178;
							continue;
						}
						goto case 2u;
					}
					num2 = (int)((num3 * 805845780) ^ 0x6A5D3839);
					continue;
				case 26u:
					point = gfx.MeasureString(Normal, num5, text2);
					num6 = _200C_206B_206C_200C_200B_200D_202E_206A_202C_206F_202B_200C_200B_206A_202D_202C_206E_202D_202A_200D_200D_206C_202B_202D_202E_200C_202D_206B_206B_200D_202C_206B_206E_202E_206C_202E_200B_206D_202D_206C_202E(point.X, point2.X);
					num2 = -2087955661;
					continue;
				case 30u:
					InstantItems.Add(value, notificationItem);
					num2 = -752466382;
					continue;
				case 10u:
				{
					int num4;
					if (list2.Count > 0)
					{
						num2 = -546786250;
						num4 = num2;
					}
					else
					{
						num2 = -1704586808;
						num4 = num2;
					}
					continue;
				}
				case 12u:
					num--;
					num2 = -2061469170;
					continue;
				case 44u:
					return;
					IL_06c8:
					num12 = num14;
					if (refpos != null)
					{
						num2 = -324743760;
						num27 = num2;
					}
					else
					{
						num2 = -272334214;
						num27 = num2;
					}
					continue;
				}
				break;
			}
		}
	}
public void InitialiseFontHeight(Graphics gfx, int numqueues)
	{
		FontHeight = gfx.MeasureString(Bold, Bold.FontSize, "ABCDEFGHIJKLMNOPQRSTUVWXYZ").Y;
		float num = (float)numqueues * getNotificationHeight() + (float)(numqueues * NotificationSpacing);
		int num2 = 0;
		float num5 = default;
		ScreenReferencePosition item = default;
		while (true)
		{
			int num3 = 1831004154;
			while (true)
			{
				uint num4;
				switch ((num4 = (uint)(num3 ^ 0x6E0B4D7E)) % 16)
				{
				case 12u:
					break;
				default:
					return;
				case 15u:
					num5 = (float)gfx.Height / 100f * (float)verticalPositionPercent - num / 100f * (float)verticalPositionPercent + getNotificationHeight() + (float)BorderWidth;
					num5 += (getNotificationHeight() + (float)NotificationSpacing) * (float)num2;
					num3 = (int)((num4 * 2096185896) ^ 0x7F937E8E);
					continue;
				case 1u:
					num5 = (float)gfx.Height / 100f * (float)verticalPositionPercent + num - num / 100f * (float)verticalPositionPercent - (float)BorderWidth;
					num3 = 841805428;
					continue;
				case 3u:
					num3 = (int)((num4 * 1342658843) ^ 0x25504218);
					continue;
				case 7u:
					num2++;
					num3 = 1615889797;
					continue;
				case 6u:
					item = new ScreenReferencePosition(EdgeIndent, (int)num5, ScreenReferenceDirection.Right, ScreenReferenceDirection.Up);
					num3 = 1713119584;
					continue;
				case 5u:
				{
					ScreenReferencePosition item2 = new ScreenReferencePosition(gfx.Width - EdgeIndent, (int)num5, ScreenReferenceDirection.Left, ScreenReferenceDirection.Up);
					ReferencePosition.Add(item2);
					num3 = ((int)num4 * -1200682022) ^ 0x44394B8F;
					continue;
				}
				case 11u:
				{
					int num8;
					if (num2 >= numqueues)
					{
						num3 = 564409939;
						num8 = num3;
					}
					else
					{
						num3 = 1247798414;
						num8 = num3;
					}
					continue;
				}
				case 13u:
					ReinitialiseFont = false;
					num3 = ((int)num4 * -1816730851) ^ 0x20781AE5;
					continue;
				case 9u:
				{
					int num7;
					if (fromRight)
					{
						num3 = 4707451;
						num7 = num3;
					}
					else
					{
						num3 = 955908296;
						num7 = num3;
					}
					continue;
				}
				case 0u:
				{
					int num6;
					if (verticalPositionPercent > 50)
					{
						num3 = 942982079;
						num6 = num3;
					}
					else
					{
						num3 = 69197937;
						num6 = num3;
					}
					continue;
				}
				case 10u:
					num5 -= (getNotificationHeight() + (float)NotificationSpacing) * (float)num2;
					num3 = ((int)num4 * -494274072) ^ 0x50176007;
					continue;
				case 14u:
					ReferencePosition.Add(item);
					num3 = ((int)num4 * -730997714) ^ 0x6DBF699D;
					continue;
				case 8u:
					num3 = (int)(num4 * 1666342791) ^ -1214433649;
					continue;
				case 4u:
					num3 = (int)(num4 * 1405584443) ^ -277685783;
					continue;
				case 2u:
					return;
				}
				break;
			}
		}
	}
private float getNotificationHeight()
	{
		float num = FontHeight * 3f + (float)(FontVerticalSpacing * 2);
		_ = CornerRounding;
		return num + 0f + (float)(PaddingY * 2);
	}
	static string _200B_206E_206E_206A_206A_200C_206E_206C_202C_206C_200E_202A_206E_202B_202B_206E_202B_206D_200F_202E_200F_202D_202B_202A_206F_202D_206B_206B_202E_206A_200F_200E_200F_202B_206B_206F_202D_202E_200C_202E(string P_0, string P_1, string P_2)
	{
		return P_0 + P_1 + P_2;
	}
	static float _200C_206B_206C_200C_200B_200D_202E_206A_202C_206F_202B_200C_200B_206A_202D_202C_206E_202D_202A_200D_200D_206C_202B_202D_202E_200C_202D_206B_206B_200D_202C_206B_206E_202E_206C_202E_200B_206D_202D_206C_202E(float P_0, float P_1)
	{
		return Math.Max(P_0, P_1);
	}
	static string _200E_206A_200E_206A_202C_202D_206B_202D_200E_206B_202E_202D_202B_206D_202B_200D_206B_200B_206A_202D_202B_206C_202B_202A_202B_202D_202D_202D_206C_200F_206E_206F_202B_200E_206A_206F_200C_200C_200B_202D_202E(string P_0, char[] P_1)
	{
		return P_0.TrimEnd(P_1);
	}
	static string[] _206A_200D_202D_202C_200B_200E_202D_200E_206A_206F_200B_202E_200B_206E_202A_202B_202D_200B_206E_206C_202A_202E_206F_206D_206B_202A_200E_202B_202D_202E_206D_206B_202B_206F_206A_206A_200C_206B_202B_202E_202E(string P_0, char P_1, StringSplitOptions P_2)
	{
		return P_0.Split(P_1, P_2);
	}
	static bool _206B_200E_206B_206A_206F_206D_202A_200F_200B_200E_202E_202E_206F_206B_200B_200B_206F_200B_206E_206E_206B_202D_206F_202D_206F_202B_202B_202E_200D_206E_202B_202C_202D_200F_202E_202B_202B_200C_202E_202B_202E(string P_0, string P_1)
	{
		return P_0.Contains(P_1);
	}
	static bool _206D_206A_206F_200E_200B_202C_202A_206F_202B_206D_202D_200E_206E_202D_202C_202E_200F_200D_202E_206A_206C_200B_206C_200E_206C_202E_206F_202D_206E_200B_206C_202C_200B_200E_202D_202C_206A_206E_202A_202A_202E(string P_0, string P_1)
	{
		return P_0 == P_1;
	}
	static string _206E_206D_206C_200B_206B_200B_202C_200B_200B_206F_202B_202D_206A_200C_200C_202D_200E_200F_202E_200F_206C_200F_206F_200C_202B_200C_206B_202A_200F_202B_200D_202E_206B_200C_200C_202B_206E_206F_206D_200D_202E(string P_0, string[] P_1)
	{
		return string.Join(P_0, P_1);
	}
}
public class Program{public static void Main(){var cases=new List<object>();var draws=new List<object>();var texts=new[]{"","Short","Long words with multiple spaces and more words than fit inside a narrow notification width",new string('x',200),"First line\r\nSecond line\r\nThird", "  Trim me\r\n  "};foreach(var title in texts)foreach(var detail in texts)foreach(var maxWidth in new[]{120,240,480})foreach(var fixedPosition in new[]{false,true}){var q=new NotificationQueue{MaxWidth=maxWidth};var item=new NotificationArgs{Title=title,Detail=detail,XPos=fixedPosition?25:-1,YPos=fixedPosition?75:-1};q.AddNotificationItem(new Graphics(),item,fixedPosition?null:new ScreenReferencePosition(1900,1000,ScreenReferenceDirection.Left,ScreenReferenceDirection.Up),NotificationTransition.Left,NotificationTransition.Right);var output=q.QueueItems.SingleOrDefault()??q.InstantItems.Values.SingleOrDefault();cases.Add(new{title,detail,maxWidth,fixedPosition,output=JsonSerializer.SerializeToElement(output)});if(title=="Short"&&maxWidth==480&&!fixedPosition){foreach(var alignment in new[]{0,1,2})foreach(var shape in new[]{0,1})foreach(var border in new[]{0,3,10}){foreach(var line in output.Lines)line.Alignment=(Alignment)alignment;output.shape=(NotificationItemShape)shape;output.BorderWidth=border;var g=new Graphics();output.Draw(g,q);draws.Add(new{detail,alignment,shape,border,lines=JsonSerializer.SerializeToElement(output.Lines),dimensions=output.Dimensions,ops=g.Ops});}}}var references=new List<object>();foreach(var height in new[]{720,1080,1440})foreach(var pct in new[]{0,25,50,51,97,100})foreach(var fromRight in new[]{false,true})foreach(var count in new[]{1,3,5}){var q=new NotificationQueue{verticalPositionPercent=pct,fromRight=fromRight};q.InitialiseFontHeight(new Graphics{Height=height},count);references.Add(new{height,pct,fromRight,count,positions=q.ReferencePosition});}Console.WriteLine(JsonSerializer.Serialize(new{cases,references,draws}));}}
