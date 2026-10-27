using System;using System.Numerics;using System.Collections.Generic;namespace EDJP.Observatory.Solver;public static class Tsp3DSolver{
public readonly struct Result
	{
		public int[] Order { get; }

		public double Length { get; }

		public double? EstimatedGapPercent { get; }

		public bool HitTimeLimit { get; }

		public Result(int[] order, double length, double? estimatedGapPercent = null, bool hitTimeLimit = false)
		{
			Order = order;
			Length = length;
			EstimatedGapPercent = estimatedGapPercent;
			HitTimeLimit = hitTimeLimit;
		}
	}
public static Result SolveExact(IReadOnlyList<Vector3> points, int start, int? end, bool freeEnd)
	{
		int count = points.Count;
		double num12 = default;
		double num3 = default;
		int num20 = default;
		int num7 = default;
		int num13 = default;
		int num9 = default;
		int num23 = default;
		int num27 = default;
		int num4 = default;
		double[,] array2 = default;
		double num17 = default;
		double num18 = default;
		int num16 = default;
		List<int> list = default;
		int num8 = default;
		int num10 = default;
		int[,] array = default;
		int num15 = default;
		int num19 = default;
		while (true)
		{
			int num = 2126763751;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0x3AD1E89C)) % 53)
				{
				case 40u:
					break;
				case 1u:
					num = ((int)num2 * -138268027) ^ -1248895545;
					continue;
				case 3u:
				{
					int num39;
					int num40;
					if (num12 < num3)
					{
						num39 = -1495400449;
						num40 = num39;
					}
					else
					{
						num39 = -391277843;
						num40 = num39;
					}
					num = num39 ^ ((int)num2 * -337371805);
					continue;
				}
				case 38u:
					num20++;
					num = 2031145960;
					continue;
				case 25u:
					num7 = num13;
					num = (int)((num2 * 1028934710) ^ 0x6281B689);
					continue;
				case 45u:
				{
					int num34;
					if (num9 < num23)
					{
						num = 1193753374;
						num34 = num;
					}
					else
					{
						num = 83956563;
						num34 = num;
					}
					continue;
				}
				case 31u:
					num = ((int)num2 * -1925850221) ^ -1124990898;
					continue;
				case 43u:
					num3 = num12;
					num27 = num4;
					num = (int)(num2 * 1095298045) ^ -1980502450;
					continue;
				case 36u:
					num27 = end.Value;
					num3 = array2[num13, num27];
					num = (int)(num2 * 1624572644) ^ -322007811;
					continue;
				case 18u:
					num17 = num18 + D(num20, num16);
					num = (int)(num2 * 2051770642) ^ -1142857961;
					continue;
				case 11u:
				{
					list.Add(num8);
					int num14;
					if (num8 != start)
					{
						num = 1609291306;
						num14 = num;
					}
					else
					{
						num = 385525946;
						num14 = num;
					}
					continue;
				}
				case 15u:
					num13 = num23 - 1;
					num = (int)((num2 * 1319561035) ^ 0x19629E05);
					continue;
				case 10u:
					num3 = 0.0;
					num = ((int)num2 * -1011973631) ^ -758430181;
					continue;
				case 4u:
					num20 = 0;
					num = ((int)num2 * -1469391902) ^ -1752465574;
					continue;
				case 34u:
				{
					int num41;
					if (num10 >= count)
					{
						num = 993804392;
						num41 = num;
					}
					else
					{
						num = 1386429417;
						num41 = num;
					}
					continue;
				}
				case 8u:
					array = new int[num23, count];
					num9 = 0;
					num = (int)(num2 * 1877570464) ^ -692753212;
					continue;
				case 26u:
					array2 = new double[num23, count];
					num = ((int)num2 * -469119468) ^ 0x5CB89F91;
					continue;
				case 21u:
					num4++;
					num = 1882675057;
					continue;
				case 27u:
				{
					int num31;
					int num32;
					if (count != 1)
					{
						num31 = -1783161863;
						num32 = num31;
					}
					else
					{
						num31 = -2138392194;
						num32 = num31;
					}
					num = num31 ^ (int)(num2 * 2089304100);
					continue;
				}
				case 47u:
				{
					num15 = num19 | (1 << num16);
					int num28;
					int num29;
					if (num17 >= array2[num15, num16])
					{
						num28 = 1842242639;
						num29 = num28;
					}
					else
					{
						num28 = 840889384;
						num29 = num28;
					}
					num = num28 ^ ((int)num2 * -757238551);
					continue;
				}
				case 33u:
					array2[1 << start, start] = 0.0;
					num = ((int)num2 * -277486521) ^ 0x5CF437C3;
					continue;
				case 0u:
					num16 = 0;
					num = (int)((num2 * 976328811) ^ 0x561B6B19);
					continue;
				case 52u:
					num23 = 1 << count;
					num = ((int)num2 * -4756044) ^ -2145629786;
					continue;
				case 46u:
					num12 = array2[num13, num4] + (freeEnd ? 0.0 : D(num4, start));
					num = 584140845;
					continue;
				case 6u:
					array2[num9, num10] = double.PositiveInfinity;
					num = 458248707;
					continue;
				case 17u:
				{
					int num6 = array[num7, num8];
					num7 ^= 1 << num8;
					num8 = num6;
					num = 999543667;
					continue;
				}
				case 13u:
					num10 = 0;
					num = 2055604045;
					continue;
				case 16u:
				{
					int num42;
					if (num16 < count)
					{
						num = 1387361397;
						num42 = num;
					}
					else
					{
						num = 1730012212;
						num42 = num;
					}
					continue;
				}
				case 19u:
				{
					int num37;
					int num38;
					if (num7 != 1 << start)
					{
						num37 = 667100692;
						num38 = num37;
					}
					else
					{
						num37 = 1928023283;
						num38 = num37;
					}
					num = num37 ^ ((int)num2 * -513359547);
					continue;
				}
				case 42u:
					list = new List<int>(count);
					num = 1669900735;
					continue;
				case 44u:
					array[num15, num16] = num20;
					num = ((int)num2 * -1511834373) ^ 0x73A40C69;
					continue;
				case 29u:
					list.Reverse();
					num = 1457817763;
					continue;
				case 39u:
					num8 = num27;
					num = ((int)num2 * -20558498) ^ -636663725;
					continue;
				case 2u:
					num = (int)(num2 * 823854251) ^ -1523552333;
					continue;
				case 50u:
					num19 = 1;
					num = (int)(num2 * 670163915) ^ -265308998;
					continue;
				case 7u:
				{
					int num36;
					if ((num19 & (1 << num16)) == 0)
					{
						num = 1400048826;
						num36 = num;
					}
					else
					{
						num = 1953765936;
						num36 = num;
					}
					continue;
				}
				case 14u:
				{
					int num35;
					if ((num19 & (1 << start)) != 0)
					{
						num = 1641729653;
						num35 = num;
					}
					else
					{
						num = 1442721561;
						num35 = num;
					}
					continue;
				}
				case 35u:
					num10++;
					num = ((int)num2 * -1861810410) ^ -553756936;
					continue;
				case 32u:
					num = ((int)num2 * -464548619) ^ -1492787366;
					continue;
				case 30u:
				{
					int num33;
					if (num20 < count)
					{
						num = 41093357;
						num33 = num;
					}
					else
					{
						num = 1442721561;
						num33 = num;
					}
					continue;
				}
				case 22u:
				{
					int num30;
					if ((num19 & (1 << num20)) != 0)
					{
						num = 509499057;
						num30 = num;
					}
					else
					{
						num = 1730012212;
						num30 = num;
					}
					continue;
				}
				case 24u:
					num16++;
					num = 683325932;
					continue;
				case 41u:
					num19++;
					num = 1612140392;
					continue;
				case 48u:
					num3 = double.PositiveInfinity;
					num27 = start;
					num = 1324486954;
					continue;
				case 28u:
					num9++;
					num = ((int)num2 * -548948421) ^ -234329958;
					continue;
				case 12u:
				{
					int num25;
					int num26;
					if (end.HasValue)
					{
						num25 = 19971616;
						num26 = num25;
					}
					else
					{
						num25 = 1192155860;
						num26 = num25;
					}
					num = num25 ^ (int)(num2 * 1999710153);
					continue;
				}
				case 49u:
				{
					int num24;
					if (num19 < num23)
					{
						num = 1129699331;
						num24 = num;
					}
					else
					{
						num = 63509642;
						num24 = num;
					}
					continue;
				}
				case 51u:
				{
					num18 = array2[num19, num20];
					int num21;
					int num22;
					if (double.IsPositiveInfinity(num18))
					{
						num21 = 2073610190;
						num22 = num21;
					}
					else
					{
						num21 = 1813846702;
						num22 = num21;
					}
					num = num21 ^ (int)(num2 * 1546955298);
					continue;
				}
				case 9u:
					array2[num15, num16] = num17;
					num = ((int)num2 * -495202125) ^ 0x1E985AD6;
					continue;
				case 5u:
					num4 = 0;
					num = (int)(num2 * 1146615419) ^ -467629821;
					continue;
				case 37u:
				{
					int num11;
					if (num4 < count)
					{
						num = 574039224;
						num11 = num;
					}
					else
					{
						num = 1042449930;
						num11 = num;
					}
					continue;
				}
				case 23u:
				{
					int num5;
					if (num4 != start)
					{
						num = 220811368;
						num5 = num;
					}
					else
					{
						num = 465281950;
						num5 = num;
					}
					continue;
				}
				default:
					return new Result(list.ToArray(), num3, 0.0);
				}
				break;
			}
		}
		double D(int a, int b)
		{
			return Vector3.Distance(points[a], points[b]);
		}
	}
}