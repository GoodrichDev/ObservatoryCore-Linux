using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;

namespace EDJP.Observatory.Solver;

public sealed class Tsp3DSolver
{
public static List<object> Trace=new List<object>();
	public sealed class Options
	{
		public int NeighborListSize { get; set; } = 12;

		public TimeSpan TimeLimit { get; set; } = TimeSpan.FromSeconds(5.0);

		public int? RandomSeed { get; set; }

		public bool ParallelRestarts { get; set; } = true;

		public int ParallelWorkers { get; set; } = Wrapper13(1, Wrapper10());

		public int ExactThreshold { get; set; } = 13;

		public double? TargetGapPercent { get; set; } = 2.0;

		public int StagnationLimit { get; set; } = 200;

		public bool FreeEnd { get; set; }

		static int Wrapper10()
		{
			return Environment.ProcessorCount;
		}

		static int Wrapper13(int P_0, int P_1)
		{
			return Math.Max(P_0, P_1);
		}
	}

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

	private sealed class Engine
	{
		private sealed class KdTree
		{
			private struct Node
			{
				public int PointIdx;

				public int Left;

				public int Right;

				public int Axis;
			}

			private readonly Node[] nodes;

			private int count;

			private readonly double[] X;

			private readonly double[] Y;

			private readonly double[] Z;

			private readonly int root;

			public KdTree(double[] x, double[] y, double[] z, int n)
			{
				X = x;
				Y = y;
				Z = z;
				nodes = new Node[Wrapper16(1, n)];
				count = 0;
				int[] array = new int[n];
				for (int i = 0; i < n; i++)
				{
					array[i] = i;
				}
				root = ((n == 0) ? (-1) : Build(array, 0, n, 0));
			}

			private double Coord(int p, int axis)
			{
				if (axis != 0)
				{
					while (true)
					{
						int num = -1290861245;
						while (true)
						{
							uint num2;
							switch ((num2 = (uint)(num ^ -760594874)) % 5)
							{
							case 3u:
								break;
							case 4u:
								return Y[p];
							case 0u:
								return Z[p];
							case 1u:
							{
								int num3;
								int num4;
								if (axis == 1)
								{
									num3 = -181156604;
									num4 = num3;
								}
								else
								{
									num3 = -119089098;
									num4 = num3;
								}
								num = num3 ^ (int)(num2 * 1329999766);
								continue;
							}
							default:
								goto end_IL_0003;
							}
							break;
						}
						continue;
						end_IL_0003:
						break;
					}
				}
				return X[p];
			}

			private int Build(int[] idx, int lo, int hi, int depth)
			{
				int num3 = default;
				int num4 = default;
				int left = default;
				int axis = default;
				KdTree kdTree = default;
				while (true)
				{
					int num = 1051303228;
					while (true)
					{
						uint num2;
						switch ((num2 = (uint)(num ^ 0x40E4890B)) % 8)
						{
						case 4u:
							break;
						case 3u:
							num3 = lo + (hi - lo) / 2;
							num4 = count++;
							num = ((int)num2 * -2098240906) ^ -1745023652;
							continue;
						case 5u:
							left = Build(idx, lo, num3, depth + 1);
							num = ((int)num2 * -1876215610) ^ 0x1FDCB6E4;
							continue;
						case 0u:
							axis = depth % 3;
							Array.Sort(idx, lo, hi - lo, Comparer<int>.Create((int a, int b) => kdTree.Coord(a, axis).CompareTo(kdTree.Coord(b, axis))));
							num = 185811232;
							continue;
						case 2u:
						{
							int num5;
							int num6;
							if (lo >= hi)
							{
								num5 = 544000683;
								num6 = num5;
							}
							else
							{
								num5 = 644597597;
								num6 = num5;
							}
							num = num5 ^ ((int)num2 * -913995645);
							continue;
						}
						case 7u:
							kdTree = this;
							num = ((int)num2 * -1906767945) ^ -780430944;
							continue;
						case 6u:
							return -1;
						default:
						{
							int right = Build(idx, num3 + 1, hi, depth + 1);
							nodes[num4] = new Node
							{
								PointIdx = idx[num3],
								Left = left,
								Right = right,
								Axis = axis
							};
							return num4;
						}
						}
						break;
					}
				}
			}

			public void KNearest(int query, int k, List<int> outResult)
			{
				List<(double, int)> list = new List<(double, int)>(k + 1);
				(double, int) current = default;
				while (true)
				{
					int num = -548711869;
					while (true)
					{
						uint num2;
						switch ((num2 = (uint)(num ^ -1354569722)) % 4)
						{
						case 3u:
							break;
						case 1u:
							Search(root, query, k, list);
							list.Sort(((double d2, int id) a, (double d2, int id) b) => a.d2.CompareTo(b.d2));
							num = -203256602;
							continue;
						case 0u:
							outResult.Clear();
							num = ((int)num2 * -622734475) ^ 0x2176D500;
							continue;
						default:
						{
							using List<(double, int)>.Enumerator enumerator = list.GetEnumerator();
							while (true)
							{
								int num3;
								int num4;
								if (enumerator.MoveNext())
								{
									num3 = -440704100;
									num4 = num3;
								}
								else
								{
									num3 = -1126625658;
									num4 = num3;
								}
								while (true)
								{
									switch ((num2 = (uint)(num3 ^ -1354569722)) % 5)
									{
									case 0u:
										num3 = -440704100;
										continue;
									default:
										return;
									case 1u:
										current = enumerator.Current;
										num3 = -293982582;
										continue;
									case 4u:
										outResult.Add(current.Item2);
										num3 = ((int)num2 * -1221671568) ^ 0x3B95E94C;
										continue;
									case 3u:
										break;
									case 2u:
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

			private void Search(int node, int query, int k, List<(double d2, int id)> best)
			{
				int index = default;
				double num3 = default;
				Node node2 = default;
				int node3 = default;

				if (node < 0)
				{
					goto IL_0007;
				}
				goto IL_00ee;
				IL_0007:
				int num = 638047250;
				goto IL_000c;
				IL_000c:
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ 0x49BDDB9E)) % 13)
					{
					case 0u:
						break;
					default:
						return;
					case 8u:
						return;
					case 11u:
						index = WorstIndex(best);
						num = 373403110;
						continue;
					case 2u:
					{
						int num9;
						int num10;
						if (best.Count >= k)
						{
							num9 = -498479966;
							num10 = num9;
						}
						else
						{
							num9 = -327224164;
							num10 = num9;
						}
						num = num9 ^ (int)(num2 * 1515799473);
						continue;
					}
					case 7u:
					{
						int num11;
						int num12;
						if (best.Count >= k)
						{
							num11 = -825576737;
							num12 = num11;
						}
						else
						{
							num11 = -591212729;
							num12 = num11;
						}
						num = num11 ^ ((int)num2 * -1858699090);
						continue;
					}
					case 12u:
						best[index] = (num3, node2.PointIdx);
						num = ((int)num2 * -1890951253) ^ 0xEBD8A68;
						continue;
					case 9u:
						goto IL_00ee;
					case 5u:
					{
						int num7;
						int num8;
						if (num3 < best[index].d2)
						{
							num7 = -374044169;
							num8 = num7;
						}
						else
						{
							num7 = -1909330005;
							num8 = num7;
						}
						num = num7 ^ ((int)num2 * -1307070749);
						continue;
					}
					case 1u:
						Search(node3, query, k, best);
						num = 2021914359;
						continue;
					case 6u:
					{
						double num4 = X[node2.PointIdx] - X[query];
						double num5 = Y[node2.PointIdx] - Y[query];
						double num6 = Z[node2.PointIdx] - Z[query];
						num3 = num4 * num4 + num5 * num5 + num6 * num6;
						num = (int)((num2 * 208130829) ^ 0x6C3D34EE);
						continue;
					}
					case 10u:
						goto IL_01c5;
					case 3u:
						best.Add((num3, node2.PointIdx));
						num = ((int)num2 * -759586900) ^ -113593041;
						continue;
					case 4u:
						return;
					}
					break;
					IL_01c5:
					double num13 = Coord(query, node2.Axis) - Coord(node2.PointIdx, node2.Axis);
					int node4 = ((num13 < 0.0) ? node2.Left : node2.Right);
					node3 = ((num13 < 0.0) ? node2.Right : node2.Left);
					Search(node4, query, k, best);
					double num14 = ((best.Count < k) ? double.MaxValue : WorstDist(best));
					int num15;
					if (!(num13 * num13 < num14))
					{
						num = 2097760658;
						num15 = num;
					}
					else
					{
						num = 1787530095;
						num15 = num;
					}
				}
				goto IL_0007;
				IL_00ee:
				node2 = nodes[node];
				int num16;
				if (node2.PointIdx == query)
				{
					num = 1142649795;
					num16 = num;
				}
				else
				{
					num = 332307969;
					num16 = num;
				}
				goto IL_000c;
			}

			private static int WorstIndex(List<(double d2, int id)> l)
			{
				int num = 0;
				int num2 = 1;
				while (true)
				{
					int num3 = 1655255087;
					while (true)
					{
						uint num4;
						switch ((num4 = (uint)(num3 ^ 0x629F8719)) % 7)
						{
						case 4u:
							break;
						case 1u:
							num = num2;
							num3 = (int)(num4 * 1585173538) ^ -92139794;
							continue;
						case 3u:
						{
							int num6;
							if (l[num2].d2 <= l[num].d2)
							{
								num3 = 1756800270;
								num6 = num3;
							}
							else
							{
								num3 = 292880873;
								num6 = num3;
							}
							continue;
						}
						case 5u:
							num3 = (int)(num4 * 252483155) ^ -1563893872;
							continue;
						case 2u:
						{
							int num5;
							if (num2 < l.Count)
							{
								num3 = 399584455;
								num5 = num3;
							}
							else
							{
								num3 = 739767691;
								num5 = num3;
							}
							continue;
						}
						case 6u:
							num2++;
							num3 = 1129829650;
							continue;
						default:
							return num;
						}
						break;
					}
				}
			}

			private static double WorstDist(List<(double d2, int id)> l)
			{
				double num = double.MinValue;
				using (List<(double, int)>.Enumerator enumerator = l.GetEnumerator())
				{
					(double, int) current = default;
					while (true)
					{
						IL_0088:
						int num2;
						int num3;
						if (!enumerator.MoveNext())
						{
							num2 = -1892022730;
							num3 = num2;
						}
						else
						{
							num2 = -546454767;
							num3 = num2;
						}
						while (true)
						{
							uint num4;
							switch ((num4 = (uint)(num2 ^ -94620035)) % 6)
							{
							case 5u:
								num2 = -546454767;
								continue;
							default:
								goto end_IL_0018;
							case 4u:
								current = enumerator.Current;
								num2 = -165127415;
								continue;
							case 0u:
								num = current.Item1;
								num2 = ((int)num4 * -392467584) ^ -365980958;
								continue;
							case 2u:
							{
								int num5;
								int num6;
								if (current.Item1 <= num)
								{
									num5 = -783016146;
									num6 = num5;
								}
								else
								{
									num5 = -603871953;
									num6 = num5;
								}
								num2 = num5 ^ (int)(num4 * 1087715791);
								continue;
							}
							case 1u:
								break;
							case 3u:
								goto end_IL_0018;
							}
							goto IL_0088;
							continue;
							end_IL_0018:
							break;
						}
						break;
					}
				}
				return num;
			}

			static int Wrapper16(int P_0, int P_1)
			{
				return Math.Max(P_0, P_1);
			}
		}

		[StructLayout(LayoutKind.Auto)]
		[CompilerGenerated]
		private struct Generated___c__DisplayClass21_0
		{
			public int[] parent;

			public int[,] adj;

			public int[] degree;
		}

		[StructLayout(LayoutKind.Auto)]
		[CompilerGenerated]
		private struct Generated___c__DisplayClass22_0
		{
			public int[] tour;

			public int[] pos;

			public Engine Generated___4__this;

			public bool[] inQueue;

			public Queue<int> queue;

			public double[] distBuf;
		}

		[StructLayout(LayoutKind.Auto)]
		[CompilerGenerated]
		private struct Generated___c__DisplayClass29_0
		{
			public Engine Generated___4__this;

			public double lowerBound;
		}

		[CompilerGenerated]
		private sealed class Generated___c__DisplayClass31_0
		{
			public int seedBase;

			public (int[] tour, double len, bool timedOut)[] results;

			public Engine Generated___4__this;

			public double lowerBound;

			internal void Generated__Run_b__0(int w)
			{
				Random rng = Wrapper17(seedBase + w * 7919 + 17);
				results[w] = Generated___4__this.RunChain(rng, Generated___4__this.Opt.TimeLimit, lowerBound);
			}

			public static Random Wrapper17(int P_0)
			{
				return new Random(P_0);
			}
		}

		private readonly double[] X;

		private readonly double[] Y;

		private readonly double[] Z;

		private readonly int N;

		private readonly int RealN;

		private readonly int Start;

		private readonly int EndOrStart;

		private readonly int DummyIndex;

		private readonly bool HasDummy;

		private readonly bool FreeEnd;

		private readonly double FreeEndPenalty;

		private readonly int[][] Neighbors;

		private readonly Options Opt;

		private readonly CancellationToken Token;

		public Engine(IReadOnlyList<Vector3> points, int start, int? end, bool openPath, bool freeEnd, Options opt, CancellationToken token)
		{
			double num12 = default;
			int num6 = default;
			double num13 = default;
			double num9 = default;
			double num8 = default;
			double num14 = default;
			double num4 = default;
			double num10 = default;
			double num5 = default;
			int num3 = default;
			Vector3 vector = default;
			while (true)
			{
				int num = -843732227;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ -2113005463)) % 32)
					{
					case 8u:
						break;
					default:
						return;
					case 20u:
						Opt = opt;
						Token = token;
						RealN = points.Count;
						FreeEnd = freeEnd;
						num = ((int)num2 * -1349767602) ^ 0x666385B1;
						continue;
					case 23u:
						num12 = Wrapper1(num12, Y[num6]);
						num = ((int)num2 * -1829111896) ^ 0xCB61BE2;
						continue;
					case 29u:
					{
						DummyIndex = (HasDummy ? RealN : (-1));
						int num15;
						if (!FreeEnd)
						{
							num = -771583058;
							num15 = num;
						}
						else
						{
							num = -114714976;
							num15 = num;
						}
						continue;
					}
					case 14u:
						num13 = num9 - num8;
						num14 = num4 - num12;
						num = ((int)num2 * -1945127213) ^ 0x716E60C7;
						continue;
					case 18u:
						num12 = double.MaxValue;
						num10 = double.MaxValue;
						num = (int)(num2 * 929310864) ^ -1635886397;
						continue;
					case 6u:
						Z = new double[N];
						num = ((int)num2 * -784607403) ^ 0x796C61A9;
						continue;
					case 21u:
						num8 = Wrapper1(num8, X[num6]);
						num9 = Wrapper18(num9, X[num6]);
						num = -1490802754;
						continue;
					case 4u:
					{
						double num16 = num5 - num10;
						FreeEndPenalty = 2.0 * (double)RealN * Wrapper3(num13 * num13 + num14 * num14 + num16 * num16) + 1.0;
						num = ((int)num2 * -1879391527) ^ -1026547382;
						continue;
					}
					case 9u:
						num8 = double.MaxValue;
						num = (int)(num2 * 601479200) ^ -2056510277;
						continue;
					case 0u:
						HasDummy = openPath | freeEnd;
						N = (HasDummy ? (RealN + 1) : RealN);
						num = -1929961647;
						continue;
					case 10u:
						num9 = double.MinValue;
						num = (int)((num2 * 1825476278) ^ 0x1F44237A);
						continue;
					case 1u:
					{
						int num11;
						if (num3 < RealN)
						{
							num = -8999009;
							num11 = num;
						}
						else
						{
							num = -16333790;
							num11 = num;
						}
						continue;
					}
					case 7u:
						Neighbors = BuildNeighborLists();
						num = -398759158;
						continue;
					case 30u:
						num3 = 0;
						num = (int)((num2 * 1108523873) ^ 0x463DBA36);
						continue;
					case 19u:
						num4 = Wrapper18(num4, Y[num6]);
						num = (int)((num2 * 2024876946) ^ 0x29C34560);
						continue;
					case 22u:
						vector = points[num3];
						num = -940393493;
						continue;
					case 31u:
						num10 = Wrapper1(num10, Z[num6]);
						num = (int)((num2 * 1410086572) ^ 0x56B2B111);
						continue;
					case 11u:
						Start = start;
						num = (int)(num2 * 1365661894) ^ -819889071;
						continue;
					case 2u:
						X[num3] = vector.X;
						num = ((int)num2 * -713002692) ^ 0x41EEB308;
						continue;
					case 16u:
						num3++;
						num = (int)(num2 * 81662101) ^ -1580797736;
						continue;
					case 5u:
					{
						int num7;
						if (num6 >= RealN)
						{
							num = -1070229113;
							num7 = num;
						}
						else
						{
							num = -179635620;
							num7 = num;
						}
						continue;
					}
					case 13u:
						num6 = 0;
						num = ((int)num2 * -1005938682) ^ 0xCCCF39C;
						continue;
					case 12u:
						num5 = Wrapper18(num5, Z[num6]);
						num6++;
						num = ((int)num2 * -781728734) ^ -153841420;
						continue;
					case 27u:
						num = ((int)num2 * -1601441460) ^ 0x4F4E50A8;
						continue;
					case 26u:
						EndOrStart = end ?? start;
						num = ((int)num2 * -277758893) ^ -2059181126;
						continue;
					case 15u:
						num4 = double.MinValue;
						num5 = double.MinValue;
						num = ((int)num2 * -2008635339) ^ -1155324417;
						continue;
					case 25u:
						Y[num3] = vector.Y;
						num = ((int)num2 * -486693738) ^ 0x42018F53;
						continue;
					case 24u:
						X = new double[N];
						num = ((int)num2 * -2112921290) ^ -198798392;
						continue;
					case 28u:
						Z[num3] = vector.Z;
						num = ((int)num2 * -1990425568) ^ -1311563783;
						continue;
					case 17u:
						Y = new double[N];
						num = (int)(num2 * 1451295093) ^ -1228814838;
						continue;
					case 3u:
						return;
					}
					break;
				}
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		private double Dist(int i, int j)
		{
			if (HasDummy)
			{
				int num7 = default;
				while (true)
				{
					int num = 879042002;
					while (true)
					{
						uint num2;
						switch ((num2 = (uint)(num ^ 0x4F04D219)) % 13)
						{
						case 6u:
							break;
						case 5u:
						{
							int num8;
							int num9;
							if (num7 != Start)
							{
								num8 = 423187228;
								num9 = num8;
							}
							else
							{
								num8 = 353506415;
								num9 = num8;
							}
							num = num8 ^ (int)(num2 * 976661241);
							continue;
						}
						case 3u:
							return 1000000000000000.0;
						case 10u:
						{
							int num5;
							int num6;
							if (i != DummyIndex)
							{
								num5 = -751107575;
								num6 = num5;
							}
							else
							{
								num5 = -41575651;
								num6 = num5;
							}
							num = num5 ^ ((int)num2 * -711799291);
							continue;
						}
						case 2u:
							goto IL_00bc;
						case 8u:
							return 0.0;
						case 1u:
						{
							int num10;
							int num11;
							if (num7 != EndOrStart)
							{
								num10 = -770316052;
								num11 = num10;
							}
							else
							{
								num10 = -343388382;
								num11 = num10;
							}
							num = num10 ^ (int)(num2 * 1324296102);
							continue;
						}
						case 11u:
							goto IL_0112;
						case 0u:
							goto IL_0125;
						case 7u:
							return FreeEndPenalty;
						case 9u:
						{
							int num3;
							int num4;
							if (j != DummyIndex)
							{
								num3 = 1211796682;
								num4 = num3;
							}
							else
							{
								num3 = 1660389444;
								num4 = num3;
							}
							num = num3 ^ (int)(num2 * 807013154);
							continue;
						}
						case 12u:
							return 0.0;
						default:
							goto end_IL_000b;
						}
						break;
						IL_0125:
						int num12 = i;
						goto IL_0129;
						IL_0112:
						if (i != DummyIndex)
						{
							num = 784557701;
							continue;
						}
						num12 = j;
						goto IL_0129;
						IL_0129:
						num7 = num12;
						int num13;
						if (FreeEnd)
						{
							num = 2009720912;
							num13 = num;
						}
						else
						{
							num = 2045674303;
							num13 = num;
						}
						continue;
						IL_00bc:
						int num14;
						if (num7 == Start)
						{
							num = 1495121046;
							num14 = num;
						}
						else
						{
							num = 1309096551;
							num14 = num;
						}
					}
					continue;
					end_IL_000b:
					break;
				}
			}
			double num15 = X[i] - X[j];
			double num16 = Y[i] - Y[j];
			double num17 = Z[i] - Z[j];
			return Wrapper3(num15 * num15 + num16 * num16 + num17 * num17);
		}

		private double ComputeLowerBound()
		{
			bool[] array = new bool[N];
			double[] array2 = new double[N];
			int num9 = default;
			double num13 = default;
			int num6 = default;
			int num4 = default;
			double num3 = default;
			int num16 = default;
			double num8 = default;
			int num7 = default;
			while (true)
			{
				int num = 1224980753;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ 0x17CB07CE)) % 28)
					{
					case 10u:
						break;
					case 7u:
						num9 = 0;
						num = (int)((num2 * 187700519) ^ 0x1CD89A6F);
						continue;
					case 23u:
					{
						num13 = Dist(num6, num4);
						int num21;
						int num22;
						if (num13 < array2[num4])
						{
							num21 = 1054704163;
							num22 = num21;
						}
						else
						{
							num21 = 862223306;
							num22 = num21;
						}
						num = num21 ^ ((int)num2 * -1482472688);
						continue;
					}
					case 5u:
						array2[num4] = num13;
						num = ((int)num2 * -1405256679) ^ -122421009;
						continue;
					case 13u:
						array2[0] = 0.0;
						num = (int)(num2 * 1727852235) ^ -1067406573;
						continue;
					case 11u:
					{
						int num18;
						int num19;
						if (num6 != -1)
						{
							num18 = 2104485065;
							num19 = num18;
						}
						else
						{
							num18 = 1356304087;
							num19 = num18;
						}
						num = num18 ^ ((int)num2 * -932705497);
						continue;
					}
					case 18u:
						num3 = 0.0;
						num = (int)((num2 * 501449907) ^ 0x2B63F954);
						continue;
					case 12u:
						num16 = 0;
						num = ((int)num2 * -583554407) ^ -1983976181;
						continue;
					case 3u:
						num8 = double.PositiveInfinity;
						num = ((int)num2 * -718290724) ^ -1660787079;
						continue;
					case 8u:
						num4++;
						num = 2088525123;
						continue;
					case 1u:
					{
						int num12;
						if (num4 >= N)
						{
							num = 1917243421;
							num12 = num;
						}
						else
						{
							num = 1090147683;
							num12 = num;
						}
						continue;
					}
					case 25u:
						num7++;
						num = 1767664657;
						continue;
					case 22u:
						num6 = num7;
						num = ((int)num2 * -942278688) ^ 0xF86C27B;
						continue;
					case 2u:
						num6 = -1;
						num = 922190621;
						continue;
					case 20u:
						num8 = array2[num7];
						num = (int)(num2 * 1692038654) ^ -497188848;
						continue;
					case 19u:
						num16++;
						num = (int)(num2 * 1307295265) ^ -165376461;
						continue;
					case 26u:
					{
						int num20;
						if (num16 >= N)
						{
							num = 2127417698;
							num20 = num;
						}
						else
						{
							num = 612685252;
							num20 = num;
						}
						continue;
					}
					case 9u:
						num = (int)((num2 * 1171817820) ^ 0x453FF34D);
						continue;
					case 14u:
					{
						int num17;
						if (array[num7])
						{
							num = 612551995;
							num17 = num;
						}
						else
						{
							num = 1305066454;
							num17 = num;
						}
						continue;
					}
					case 15u:
					{
						int num15;
						if (num7 < N)
						{
							num = 53888472;
							num15 = num;
						}
						else
						{
							num = 897363981;
							num15 = num;
						}
						continue;
					}
					case 24u:
					{
						int num14;
						if (num9 >= N)
						{
							num = 1048662819;
							num14 = num;
						}
						else
						{
							num = 305633746;
							num14 = num;
						}
						continue;
					}
					case 27u:
						num7 = 0;
						num = ((int)num2 * -774545893) ^ -540342338;
						continue;
					case 16u:
					{
						int num10;
						int num11;
						if (array2[num7] >= num8)
						{
							num10 = -2018651725;
							num11 = num10;
						}
						else
						{
							num10 = -671742894;
							num11 = num10;
						}
						num = num10 ^ ((int)num2 * -1218733989);
						continue;
					}
					case 17u:
						num = (int)((num2 * 1611501096) ^ 0x5D861368);
						continue;
					case 0u:
						array2[num9] = double.PositiveInfinity;
						num9++;
						num = 1540736150;
						continue;
					case 6u:
						array[num6] = true;
						num3 += num8;
						num4 = 0;
						num = ((int)num2 * -425041979) ^ -1786961991;
						continue;
					case 21u:
					{
						int num5;
						if (array[num4])
						{
							num = 489710394;
							num5 = num;
						}
						else
						{
							num = 9774705;
							num5 = num;
						}
						continue;
					}
					default:
						return num3;
					}
					break;
				}
			}
		}

		private int[][] BuildNeighborLists()
		{
			KdTree kdTree = new KdTree(X, Y, Z, RealN);
			int[][] array = default;
			int num4 = default;
			int num3 = default;
			List<int> list = default;
			while (true)
			{
				int num = 218901300;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ 0x621F7A69)) % 16)
					{
					case 15u:
						break;
					case 13u:
						array = new int[N][];
						num = (int)((num2 * 48890863) ^ 0x57ADBDBB);
						continue;
					case 4u:
						num4 = 0;
						num = (int)((num2 * 275158646) ^ 0x6D297692);
						continue;
					case 3u:
						num = (int)((num2 * 979499038) ^ 0x67D40E35);
						continue;
					case 8u:
						num = ((int)num2 * -721119081) ^ 0x38AB3C41;
						continue;
					case 11u:
						array[DummyIndex] = new int[1] { Start };
						num = (int)((num2 * 76738857) ^ 0x75C53FAD);
						continue;
					case 2u:
					{
						int num7;
						int num8;
						if (!FreeEnd)
						{
							num7 = -1131661725;
							num8 = num7;
						}
						else
						{
							num7 = -221860252;
							num8 = num7;
						}
						num = num7 ^ ((int)num2 * -1997524629);
						continue;
					}
					case 6u:
					{
						int num6;
						if (num4 < RealN)
						{
							num = 838513811;
							num6 = num;
						}
						else
						{
							num = 1558293819;
							num6 = num;
						}
						continue;
					}
					case 9u:
						AppendIfMissing(ref array[EndOrStart], DummyIndex);
						num = ((int)num2 * -1897449401) ^ 0x796D7C26;
						continue;
					case 12u:
					{
						int num5;
						if (!HasDummy)
						{
							num = 374145161;
							num5 = num;
						}
						else
						{
							num = 1162763452;
							num5 = num;
						}
						continue;
					}
					case 1u:
						num3 = Wrapper15(1, Wrapper12(Opt.NeighborListSize, RealN - 1));
						num = ((int)num2 * -1643923831) ^ 0x589B702E;
						continue;
					case 5u:
						array[DummyIndex] = new int[2] { Start, EndOrStart };
						AppendIfMissing(ref array[Start], DummyIndex);
						num = ((int)num2 * -1170740428) ^ -1651530636;
						continue;
					case 7u:
						AppendIfMissing(ref array[Start], DummyIndex);
						num = ((int)num2 * -1118007479) ^ 0x6DD97AAE;
						continue;
					case 10u:
						kdTree.KNearest(num4, num3, list);
						array[num4] = list.ToArray();
						num4++;
						num = 754150703;
						continue;
					case 14u:
						list = new List<int>(num3);
						num = ((int)num2 * -1095692834) ^ 0x57595D09;
						continue;
					default:
						return array;
					}
					break;
				}
			}
		}

		private static void AppendIfMissing(ref int[] arr, int val)
		{
			int[] array = default;

			if (Array.IndexOf(arr, val) >= 0)
			{
				goto IL_000b;
			}
			goto IL_0049;
			IL_000b:
			int num = -836350115;
			goto IL_0010;
			IL_0010:
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -1862801656)) % 6)
				{
				case 3u:
					break;
				default:
					return;
				case 1u:
					return;
				case 2u:
					goto IL_0049;
				case 5u:
					array[^1] = val;
					arr = array;
					num = ((int)num2 * -906313342) ^ 0x5DA490A8;
					continue;
				case 4u:
					Wrapper0((Array)arr, (Array)array, arr.Length);
					num = ((int)num2 * -1737447500) ^ -336507711;
					continue;
				case 0u:
					return;
				}
				break;
			}
			goto IL_000b;
			IL_0049:
			array = new int[arr.Length + 1];
			num = -1588974458;
			goto IL_0010;
		}

		private void BatchDistancesFrom(int c1, int[] ids, double[] outDist)
		{
			double num8 = default;
			int num15 = default;
			double num11 = default;
			Span<double> values2 = default;
			int num4 = default;
			int num5 = default;
			int count = default;
			int num12 = default;
			Span<double> values3 = default;
			int num9 = default;
			int num10 = default;
			double num18 = default;
			Span<double> values = default;
			double num19 = default;
			int num20 = default;
			Vector<double> vector4 = default;
			double num6 = default;
			double num7 = default;

			int num = ids.Length;
			if (HasDummy)
			{
				goto IL_000f;
			}
			goto IL_03d1;
			IL_000f:
			int num2 = 1184062719;
			goto IL_0014;
			IL_0014:
			while (true)
			{
				uint num3;
				switch ((num3 = (uint)(num2 ^ 0x6B18AE19)) % 35)
				{
				case 10u:
					break;
				default:
					return;
				case 11u:
					num8 = Z[num15] - num11;
					num2 = (int)(num3 * 1854682999) ^ -1250482885;
					continue;
				case 18u:
					values2 = new double[num4];
					num2 = (int)(num3 * 1840625038) ^ -1930980942;
					continue;
				case 28u:
					goto IL_00f9;
				case 27u:
					goto IL_0112;
				case 3u:
					return;
				case 7u:
				{
					num5 = 0;
					int num16;
					int num17;
					if (count >= 2)
					{
						num16 = -258840787;
						num17 = num16;
					}
					else
					{
						num16 = -1282889221;
						num17 = num16;
					}
					num2 = num16 ^ ((int)num3 * -915510348);
					continue;
				}
				case 16u:
					num12 = 0;
					num2 = ((int)num3 * -2136053008) ^ -1758525921;
					continue;
				case 23u:
					num2 = ((int)num3 * -181463891) ^ 0x59458471;
					continue;
				case 1u:
					values3[num9] = Y[num10] - num18;
					num2 = ((int)num3 * -1132660440) ^ 0x325C147E;
					continue;
				case 17u:
					num10 = ids[num5 + num9];
					values[num9] = X[num10] - num19;
					num2 = 1609052882;
					continue;
				case 21u:
					num2 = (int)(num3 * 896422063) ^ -1070815844;
					continue;
				case 2u:
					outDist[num5 + num20] = vector4[num20];
					num20++;
					num2 = 1990548970;
					continue;
				case 9u:
					num20 = 0;
					num2 = (int)((num3 * 818972668) ^ 0x247BBBA7);
					continue;
				case 31u:
					num4 = count;
					num2 = ((int)num3 * -1114263790) ^ 0x5A19B712;
					continue;
				case 6u:
					num15 = ids[num5];
					num6 = X[num15] - num19;
					num2 = 1807077297;
					continue;
				case 14u:
					num9 = 0;
					num2 = 269327862;
					continue;
				case 0u:
				{
					Vector<double> vector = new Vector<double>(values);
					Vector<double> vector2 = new Vector<double>(values3);
					Vector<double> vector3 = new Vector<double>(values2);
					vector4 = Vector.SquareRoot(vector * vector + vector2 * vector2 + vector3 * vector3);
					num2 = ((int)num3 * -1330341425) ^ 0x7025A289;
					continue;
				}
				case 8u:
					goto IL_02bd;
				case 12u:
					values3 = new double[num4];
					num2 = ((int)num3 * -969150070) ^ -799628674;
					continue;
				case 20u:
					num2 = ((int)num3 * -931899447) ^ -1723431815;
					continue;
				case 13u:
					num7 = Y[num15] - num18;
					num2 = (int)((num3 * 359400593) ^ 0xA5E33E8);
					continue;
				case 33u:
				{
					int num13;
					int num14;
					if (num >= count)
					{
						num13 = -1266212009;
						num14 = num13;
					}
					else
					{
						num13 = -1444256033;
						num14 = num13;
					}
					num2 = num13 ^ ((int)num3 * -189754666);
					continue;
				}
				case 29u:
					num5++;
					num2 = (int)((num3 * 847449932) ^ 0x6AA3268B);
					continue;
				case 5u:
					outDist[num12] = Dist(c1, ids[num12]);
					num12++;
					num2 = 691402175;
					continue;
				case 25u:
					num5 += count;
					num2 = ((int)num3 * -1563563072) ^ 0x519BA53C;
					continue;
				case 4u:
					values2[num9] = Z[num10] - num11;
					num9++;
					num2 = ((int)num3 * -847983906) ^ -310777447;
					continue;
				case 32u:
					goto IL_03d1;
				case 30u:
					outDist[num5] = Wrapper3(num6 * num6 + num7 * num7 + num8 * num8);
					num2 = (int)(num3 * 1738126541) ^ -713696049;
					continue;
				case 19u:
					goto IL_042a;
				case 34u:
					num2 = (int)(num3 * 177745616) ^ -2074613045;
					continue;
				case 15u:
					num4 = count;
					num2 = (int)(num3 * 2120498660) ^ -718467050;
					continue;
				case 26u:
					goto IL_046e;
				case 22u:
					values = new double[num4];
					num4 = count;
					num2 = (int)(num3 * 1544360983) ^ -671342002;
					continue;
				case 24u:
					return;
				}
				break;
				IL_046e:
				int num21;
				if (num5 >= num)
				{
					num2 = 228037771;
					num21 = num2;
				}
				else
				{
					num2 = 804759469;
					num21 = num2;
				}
				continue;
				IL_00f9:
				int num22;
				if (num12 >= num)
				{
					num2 = 1446872755;
					num22 = num2;
				}
				else
				{
					num2 = 1516366450;
					num22 = num2;
				}
				continue;
				IL_02bd:
				int num23;
				if (num5 + count <= num)
				{
					num2 = 471722954;
					num23 = num2;
				}
				else
				{
					num2 = 1705384607;
					num23 = num2;
				}
				continue;
				IL_042a:
				int num24;
				if (num20 >= count)
				{
					num2 = 1263099500;
					num24 = num2;
				}
				else
				{
					num2 = 832912414;
					num24 = num2;
				}
				continue;
				IL_0112:
				int num25;
				if (num9 >= count)
				{
					num2 = 680377836;
					num25 = num2;
				}
				else
				{
					num2 = 332282701;
					num25 = num2;
				}
			}
			goto IL_000f;
			IL_03d1:
			num19 = X[c1];
			num18 = Y[c1];
			num11 = Z[c1];
			count = Vector<double>.Count;
			num2 = 1680163516;
			goto IL_0014;
		}

		private int[] GreedyConstruction()
		{
			HashSet<long> hashSet = new HashSet<long>();
			Generated___c__DisplayClass21_0 obj = default;
			int num79 = default;
			int num84 = default;
			int[] array4 = default;
			List<(double, int, int)> list = default;
			bool flag = default;
			int num85 = default;
			int num83 = default;
			int num82 = default;
			int num8 = default;
			long item5 = default;
			int item = default;
			int item2 = default;
			bool[] array2 = default;
			int num32 = default;
			int num49 = default;
			int num56 = default;
			int num53 = default;
			int num15 = default;
			int num23 = default;
			int num42 = default;
			int num22 = default;
			double num31 = default;
			double num30 = default;
			(int, int) tuple2 = default;
			(int, int) tuple5 = default;
			int index2 = default;
			(int, int) tuple3 = default;
			int index = default;
			int num36 = default;
			int num41 = default;
			int num34 = default;
			bool[] array3 = default;
			int num39 = default;
			int num14 = default;
			int num19 = default;
			int num44 = default;
			(int, int) tuple7 = default;
			int num16 = default;
			int num46 = default;
			int num33 = default;
			int num71 = default;
			(int, int) tuple8 = default;
			int i = default;
			int j = default;
			int num57 = default;
			int item4 = default;
			int item3 = default;
			int num24 = default;
			int[] array = default;
			int a = default;
			int b = default;
			while (true)
			{
				int num = -894155504;
				while (true)
				{
					int num81;
					long num86;
					uint num2;
					switch ((num2 = (uint)(num ^ -12786660)) % 32)
					{
					case 8u:
						break;
					case 2u:
						obj.adj[num79, 0] = -1;
						num = -601799098;
						continue;
					case 13u:
					{
						int num90;
						if (num84 >= array4.Length)
						{
							num = -1868664903;
							num90 = num;
						}
						else
						{
							num = -183581539;
							num90 = num;
						}
						continue;
					}
					case 15u:
						num79 = 0;
						num = (int)((num2 * 59758520) ^ 0x2387BCAC);
						continue;
					case 7u:
						list.Add((flag ? double.NegativeInfinity : Dist(num85, num83), num85, num83));
						num = -847803799;
						continue;
					case 1u:
						num83 = array4[num84];
						num = -1370459769;
						continue;
					case 22u:
						num = (int)(num2 * 1979203473) ^ -1450573972;
						continue;
					case 25u:
						if (num85 != DummyIndex)
						{
							num = (int)(num2 * 81171953) ^ -1630298359;
							continue;
						}
						num81 = 1;
						goto IL_03cb;
					case 18u:
						obj.parent[num82] = num82;
						num = -2090862446;
						continue;
					case 31u:
					{
						int num91;
						if (num82 < N)
						{
							num = -688564466;
							num91 = num;
						}
						else
						{
							num = -2019143850;
							num91 = num;
						}
						continue;
					}
					case 23u:
						num = (int)(num2 * 1746131788) ^ -705804731;
						continue;
					case 26u:
						obj.adj[num79, 1] = -1;
						num79++;
						num = (int)((num2 * 2078548267) ^ 0x5B749C7A);
						continue;
					case 17u:
						list.Sort(((double d, int a, int b) x, (double d, int a, int b) y) => x.d.CompareTo(y.d));
						num = -802643768;
						continue;
					case 24u:
					{
						int num80;
						if (num79 >= N)
						{
							num = -907652596;
							num80 = num;
						}
						else
						{
							num = -19368962;
							num80 = num;
						}
						continue;
					}
					case 3u:
						array4 = Neighbors[num85];
						num = -59875977;
						continue;
					case 29u:
						num86 = ((long)num83 << 32) | (uint)num85;
						goto IL_0248;
					case 16u:
						num8 = 0;
						num = (int)(num2 * 1092324046) ^ -1291163592;
						continue;
					case 30u:
					{
						int num92;
						int num93;
						if (hashSet.Add(item5))
						{
							num92 = -313722141;
							num93 = num92;
						}
						else
						{
							num92 = -254168859;
							num93 = num92;
						}
						num = num92 ^ (int)(num2 * 1788044602);
						continue;
					}
					case 27u:
					{
						int num88;
						int num89;
						if (num85 != num83)
						{
							num88 = -1752737971;
							num89 = num88;
						}
						else
						{
							num88 = -202828296;
							num89 = num88;
						}
						num = num88 ^ (int)(num2 * 407758915);
						continue;
					}
					case 6u:
					{
						int num87;
						if (num85 < N)
						{
							num = -1397773153;
							num87 = num;
						}
						else
						{
							num = -1927910099;
							num87 = num;
						}
						continue;
					}
					case 5u:
						num85++;
						num = (int)(num2 * 441092937) ^ -1680661641;
						continue;
					case 10u:
						obj.adj = new int[N, 2];
						num = (int)((num2 * 2399955) ^ 0x72C52D6D);
						continue;
					case 21u:
						num84++;
						num = -795788655;
						continue;
					case 0u:
						if (num85 < num83)
						{
							num86 = ((long)num85 << 32) | (uint)num83;
							goto IL_0248;
						}
						num = (int)((num2 * 537154660) ^ 0x3F0DFE81);
						continue;
					case 20u:
						obj.degree = new int[N];
						obj.parent = new int[N];
						num82 = 0;
						num = (int)(num2 * 1060911955) ^ -2041644407;
						continue;
					case 9u:
						num = ((int)num2 * -685399442) ^ -1536889059;
						continue;
					case 12u:
						list = new List<(double, int, int)>();
						num85 = 0;
						num = (int)(num2 * 900510167) ^ -1740558914;
						continue;
					case 11u:
						num84 = 0;
						num = (int)(num2 * 799358324) ^ -1793459273;
						continue;
					case 28u:
						num81 = ((num83 == DummyIndex) ? 1 : 0);
						goto IL_03cb;
					case 14u:
						num82++;
						num = ((int)num2 * -247485454) ^ -179877697;
						continue;
					case 19u:
						if (!FreeEnd)
						{
							num81 = 0;
							goto IL_03cb;
						}
						num = (int)(num2 * 1633344042) ^ -64119461;
						continue;
					default:
						{
							using (List<(double, int, int)>.Enumerator enumerator = list.GetEnumerator())
							{
								while (true)
								{
									IL_0458:
									int num3;
									int num4;
									if (!enumerator.MoveNext())
									{
										num3 = -2042277361;
										num4 = num3;
									}
									else
									{
										num3 = -414014566;
										num4 = num3;
									}
									while (true)
									{
										switch ((num2 = (uint)(num3 ^ -12786660)) % 10)
										{
										case 5u:
											num3 = -414014566;
											continue;
										default:
											goto end_IL_041a;
										case 1u:
											break;
										case 2u:
										{
											int num11;
											int num12;
											if (num8 < N - 1)
											{
												num11 = -2355729;
												num12 = num11;
											}
											else
											{
												num11 = -943879332;
												num12 = num11;
											}
											num3 = num11 ^ ((int)num2 * -2121860541);
											continue;
										}
										case 9u:
										{
											int num7;
											if (obj.degree[item] < 2)
											{
												num3 = -851023834;
												num7 = num3;
											}
											else
											{
												num3 = -496467655;
												num7 = num3;
											}
											continue;
										}
										case 7u:
											Generated__GreedyConstruction_g__Connect_21_2(item, item2, ref obj);
											num8++;
											num3 = ((int)num2 * -1205679779) ^ 0x12C0DB4A;
											continue;
										case 8u:
										{
											int num9;
											int num10;
											if (Generated__GreedyConstruction_g__Find_21_1(item, ref obj) == Generated__GreedyConstruction_g__Find_21_1(item2, ref obj))
											{
												num9 = -339414695;
												num10 = num9;
											}
											else
											{
												num9 = -1498844557;
												num10 = num9;
											}
											num3 = num9 ^ (int)(num2 * 784034365);
											continue;
										}
										case 0u:
										{
											int num5;
											int num6;
											if (obj.degree[item2] < 2)
											{
												num5 = -460008264;
												num6 = num5;
											}
											else
											{
												num5 = -399202691;
												num6 = num5;
											}
											num3 = num5 ^ (int)(num2 * 978118794);
											continue;
										}
										case 4u:
										{
											(double, int, int) current = enumerator.Current;
											item = current.Item2;
											item2 = current.Item3;
											num3 = -1329522310;
											continue;
										}
										case 6u:
											goto end_IL_041a;
										case 3u:
											goto end_IL_041a;
										}
										goto IL_0458;
										continue;
										end_IL_041a:
										break;
									}
									break;
								}
							}
							List<(int, int)> list2 = new List<(int, int)>();
							while (true)
							{
								int num13 = -1107974714;
								while (true)
								{
									int num47;
									int num21;
									int num37;
									int num69;
									int num52;
									int num29;
									int num45;
									int num35;
									int num73;
									int num43;
									switch ((num2 = (uint)(num13 ^ -12786660)) % 88)
									{
									case 41u:
										break;
									case 28u:
										array2[num32] = true;
										num49 = obj.adj[num32, 0];
										num56 = obj.adj[num32, 1];
										num13 = (int)((num2 * 1065906131) ^ 0x52F685C9);
										continue;
									case 60u:
										num47 = -1;
										goto IL_073c;
									case 15u:
										num53 = obj.adj[num15, 0];
										num13 = -1942496628;
										continue;
									case 18u:
										num23++;
										num13 = ((int)num2 * -1849003682) ^ -371541925;
										continue;
									case 8u:
										num42++;
										num13 = -1102128115;
										continue;
									case 26u:
									{
										int num38;
										if (num22 >= list2.Count)
										{
											num13 = -219533426;
											num38 = num13;
										}
										else
										{
											num13 = -282460361;
											num38 = num13;
										}
										continue;
									}
									case 42u:
									{
										int num67;
										int num68;
										if (num31 < num30)
										{
											num67 = -1660013478;
											num68 = num67;
										}
										else
										{
											num67 = -628576015;
											num68 = num67;
										}
										num13 = num67 ^ (int)(num2 * 824234601);
										continue;
									}
									case 29u:
									{
										int num61;
										if (list2.Count != 1)
										{
											num13 = -841944821;
											num61 = num13;
										}
										else
										{
											num13 = -1969622236;
											num61 = num13;
										}
										continue;
									}
									case 20u:
									{
										int num78;
										if (list2.Count > 1)
										{
											num13 = -896869000;
											num78 = num13;
										}
										else
										{
											num13 = -1981099255;
											num78 = num13;
										}
										continue;
									}
									case 19u:
										tuple2 = list2[num22];
										if (tuple2.Item1 != tuple2.Item2)
										{
											num13 = -61564780;
											continue;
										}
										num21 = 1;
										goto IL_1007;
									case 83u:
										tuple5 = list2[index2];
										num13 = ((int)num2 * -1859620456) ^ 0x7CAE1843;
										continue;
									case 84u:
									{
										int num66;
										if (Token.IsCancellationRequested)
										{
											num13 = -1981099255;
											num66 = num13;
										}
										else
										{
											num13 = -1066610115;
											num66 = num13;
										}
										continue;
									}
									case 5u:
									{
										int num54;
										int num55;
										if (num53 != -1)
										{
											num54 = -359247739;
											num55 = num54;
										}
										else
										{
											num54 = -463774798;
											num55 = num54;
										}
										num13 = num54 ^ ((int)num2 * -1366975584);
										continue;
									}
									case 31u:
										tuple3 = list2[index];
										if (num36 != 0)
										{
											num13 = ((int)num2 * -1146183555) ^ 0x1053539E;
											continue;
										}
										(num37, _) = tuple5;
										goto IL_0cd0;
									case 13u:
										num41 = num34;
										num13 = (int)(num2 * 1305003554) ^ -1249027447;
										continue;
									case 47u:
										array3[num15] = true;
										num13 = ((int)num2 * -1058250638) ^ 0x6377637D;
										continue;
									case 22u:
										num69 = tuple3.Item2;
										goto IL_08f7;
									case 81u:
									{
										int num74;
										int num75;
										if (num49 == -1)
										{
											num74 = -1045308138;
											num75 = num74;
										}
										else
										{
											num74 = -13045227;
											num75 = num74;
										}
										num13 = num74 ^ (int)(num2 * 1909984896);
										continue;
									}
									case 73u:
									{
										int num70;
										if (num42 < N)
										{
											num13 = -83971837;
											num70 = num13;
										}
										else
										{
											num13 = -634674673;
											num70 = num13;
										}
										continue;
									}
									case 27u:
										num39 = num32;
										num13 = -1248210973;
										continue;
									case 9u:
										num30 = double.MaxValue;
										index2 = -1;
										num13 = ((int)num2 * -407649014) ^ 0x64139A04;
										continue;
									case 62u:
									{
										int num65;
										if (num14 != -1)
										{
											num13 = -1210221823;
											num65 = num13;
										}
										else
										{
											num13 = -1812497788;
											num65 = num13;
										}
										continue;
									}
									case 65u:
										num15 = num19;
										num13 = (int)((num2 * 841904241) ^ 0x7F1655AA);
										continue;
									case 55u:
									{
										int num62;
										if (array2[num42])
										{
											num13 = -267658932;
											num62 = num13;
										}
										else
										{
											num13 = -2104740067;
											num62 = num13;
										}
										continue;
									}
									case 58u:
										num44++;
										num13 = ((int)num2 * -1834254892) ^ -1605385180;
										continue;
									case 87u:
										num13 = (int)((num2 * 1531198385) ^ 0x38B14D27);
										continue;
									case 77u:
										if (tuple7.Item1 != tuple7.Item2)
										{
											num13 = ((int)num2 * -174244390) ^ 0x185251DE;
											continue;
										}
										num52 = 1;
										goto IL_0b38;
									case 74u:
										(num29, _) = tuple3;
										goto IL_0a26;
									case 23u:
										if (num44 != 0)
										{
											num13 = -1368075558;
											continue;
										}
										(num45, _) = tuple7;
										goto IL_0b54;
									case 61u:
										num42 = 0;
										num13 = ((int)num2 * -1472982528) ^ 0x697FE60D;
										continue;
									case 59u:
									{
										int num27;
										int num28;
										if (obj.degree[num19] < 2)
										{
											num27 = 597018731;
											num28 = num27;
										}
										else
										{
											num27 = 282254317;
											num28 = num27;
										}
										num13 = num27 ^ (int)(num2 * 1548295634);
										continue;
									}
									case 6u:
									{
										int num20;
										if (array3[num19])
										{
											num13 = -1318078581;
											num20 = num13;
										}
										else
										{
											num13 = -1595644249;
											num20 = num13;
										}
										continue;
									}
									case 51u:
										array2 = new bool[N];
										num13 = ((int)num2 * -2098841181) ^ -36225565;
										continue;
									case 11u:
										num16 = 0;
										num13 = (int)((num2 * 124764333) ^ 0x4A4524C7);
										continue;
									case 21u:
										num35 = tuple2.Item2;
										goto IL_0af7;
									case 4u:
										num13 = ((int)num2 * -218058483) ^ -48359719;
										continue;
									case 17u:
									{
										int num77;
										if (num19 < N)
										{
											num13 = -912541950;
											num77 = num13;
										}
										else
										{
											num13 = -1299130781;
											num77 = num13;
										}
										continue;
									}
									case 24u:
										num52 = 2;
										goto IL_0b38;
									case 38u:
										num45 = tuple7.Item2;
										goto IL_0b54;
									case 70u:
										num22++;
										num13 = ((int)num2 * -592139194) ^ -1630333214;
										continue;
									case 49u:
										num46 = num42;
										num13 = (int)((num2 * 411439381) ^ 0x27270075);
										continue;
									case 14u:
										num19 = 0;
										num13 = ((int)num2 * -46892703) ^ -1224317890;
										continue;
									case 0u:
										num13 = (int)(num2 * 941181617) ^ -781700217;
										continue;
									case 56u:
										num73 = -1;
										goto IL_0bc5;
									case 63u:
										num19++;
										num13 = -1900031835;
										continue;
									case 44u:
										index = num22;
										num36 = num44;
										num13 = ((int)num2 * -1859631145) ^ 0x3009045;
										continue;
									case 53u:
										if (num14 != num33)
										{
											num73 = num14;
											goto IL_0bc5;
										}
										num13 = (int)(num2 * 630201260) ^ -413561608;
										continue;
									case 50u:
									{
										int num76;
										if (num56 == -1)
										{
											num13 = -1249253136;
											num76 = num13;
										}
										else
										{
											num13 = -982866292;
											num76 = num13;
										}
										continue;
									}
									case 36u:
										num13 = (int)((num2 * 1340823035) ^ 0x59D5B39B);
										continue;
									case 33u:
										if (num53 != num33)
										{
											num73 = num53;
											goto IL_0bc5;
										}
										num13 = (int)((num2 * 2116248040) ^ 0x4DD12F3A);
										continue;
									case 71u:
										num34++;
										num13 = -1816689978;
										continue;
									case 39u:
										num32 = num46;
										num16++;
										num13 = (int)((num2 * 416642944) ^ 0x464DA478);
										continue;
									case 66u:
										array3 = new bool[N];
										num13 = ((int)num2 * -1707477158) ^ -593535306;
										continue;
									case 52u:
										num23 = 0;
										num13 = (int)(num2 * 2123176619) ^ -1215494920;
										continue;
									case 25u:
										num37 = tuple5.Item2;
										goto IL_0cd0;
									case 82u:
									{
										int num72;
										if (num34 < num71)
										{
											num13 = -1504377023;
											num72 = num13;
										}
										else
										{
											num13 = -1001740010;
											num72 = num13;
										}
										continue;
									}
									case 30u:
										(num43, _) = tuple5;
										goto IL_0d06;
									case 37u:
										if (!array2[num49])
										{
											num47 = num49;
											goto IL_073c;
										}
										num13 = ((int)num2 * -890885383) ^ -1213761845;
										continue;
									case 76u:
									{
										int num63;
										int num64;
										if (num46 != -1)
										{
											num63 = -70400473;
											num64 = num63;
										}
										else
										{
											num63 = -1032464039;
											num64 = num63;
										}
										num13 = num63 ^ (int)(num2 * 1518806842);
										continue;
									}
									case 72u:
									{
										tuple8 = list2[0];
										int num59;
										int num60;
										if (tuple8.Item1 != tuple8.Item2)
										{
											num59 = 1852902071;
											num60 = num59;
										}
										else
										{
											num59 = 1608088307;
											num60 = num59;
										}
										num13 = num59 ^ (int)(num2 * 1133724073);
										continue;
									}
									case 86u:
										num36 = 0;
										num41 = 0;
										num13 = (int)((num2 * 42897433) ^ 0x68253CFE);
										continue;
									case 46u:
										num31 = Dist(i, j);
										num13 = (int)(num2 * 512341156) ^ -731596074;
										continue;
									case 32u:
									{
										int num58;
										if (num44 < num57)
										{
											num13 = -1732595117;
											num58 = num13;
										}
										else
										{
											num13 = -982661014;
											num58 = num13;
										}
										continue;
									}
									case 3u:
										tuple7 = list2[num23];
										num13 = -1619063719;
										continue;
									case 34u:
										list2.RemoveAt(index2);
										list2.Add((item4, item3));
										num13 = ((int)num2 * -1583359958) ^ -1647542196;
										continue;
									case 64u:
										if (!array2[num56])
										{
											num47 = num56;
											goto IL_073c;
										}
										num13 = (int)(num2 * 755101365) ^ -1226373344;
										continue;
									case 1u:
									{
										int num50;
										int num51;
										if (num49 == num39)
										{
											num50 = -259853026;
											num51 = num50;
										}
										else
										{
											num50 = -417675247;
											num51 = num50;
										}
										num13 = num50 ^ ((int)num2 * -1590913976);
										continue;
									}
									case 43u:
										array3[num24] = true;
										num33 = num15;
										num15 = num24;
										num13 = ((int)num2 * -1528184654) ^ -2104076123;
										continue;
									case 67u:
										Generated__GreedyConstruction_g__Connect_21_2(tuple8.Item1, tuple8.Item2, ref obj);
										num13 = ((int)num2 * -1457020353) ^ -1656191898;
										continue;
									case 35u:
									{
										int num48;
										if (num23 >= list2.Count)
										{
											num13 = -888271257;
											num48 = num13;
										}
										else
										{
											num13 = -720381505;
											num48 = num13;
										}
										continue;
									}
									case 79u:
										index2 = num23;
										num13 = (int)((num2 * 930256220) ^ 0x64243B74);
										continue;
									case 69u:
										if (num36 == 0)
										{
											num43 = tuple5.Item2;
											goto IL_0d06;
										}
										num13 = (int)((num2 * 2132031507) ^ 0x1724E21D);
										continue;
									case 12u:
									{
										int num40;
										if (num16 < N)
										{
											num13 = -1282323340;
											num40 = num13;
										}
										else
										{
											num13 = -170209092;
											num40 = num13;
										}
										continue;
									}
									case 7u:
										array = new int[N];
										num13 = -1104828361;
										continue;
									case 78u:
										num32 = 0;
										num39 = -1;
										num13 = ((int)num2 * -1583430192) ^ 0x7510AC67;
										continue;
									case 85u:
										if (num34 != 0)
										{
											num13 = -412976311;
											continue;
										}
										(num35, _) = tuple2;
										goto IL_0af7;
									case 57u:
										num33 = -1;
										num13 = (int)(num2 * 1490765367) ^ -2038492118;
										continue;
									case 48u:
										array[num16] = num32;
										num13 = -1865053560;
										continue;
									case 2u:
										index = -1;
										num13 = ((int)num2 * -82088351) ^ 0x5C896630;
										continue;
									case 68u:
										num30 = num31;
										num13 = ((int)num2 * -498145374) ^ 0x534088C3;
										continue;
									case 75u:
									{
										int num25;
										int num26;
										if (num24 == -1)
										{
											num25 = -348024270;
											num26 = num25;
										}
										else
										{
											num25 = -587709273;
											num26 = num25;
										}
										num13 = num25 ^ (int)(num2 * 1999760656);
										continue;
									}
									case 10u:
										num22 = num23 + 1;
										num13 = (int)(num2 * 1705701844) ^ -165351986;
										continue;
									case 54u:
										list2.Add((num19, num15));
										num13 = -1318078581;
										continue;
									case 40u:
										num21 = 2;
										goto IL_1007;
									case 45u:
									{
										int num17;
										int num18;
										if (num16 < N - 1)
										{
											num17 = -831380989;
											num18 = num17;
										}
										else
										{
											num17 = -170272027;
											num18 = num17;
										}
										num13 = num17 ^ (int)(num2 * 330736722);
										continue;
									}
									case 80u:
										num14 = obj.adj[num15, 1];
										num13 = ((int)num2 * -656853338) ^ 0x5D78E0E9;
										continue;
									default:
										{
											return array;
										}
										IL_0a26:
										item3 = num29;
										Generated__GreedyConstruction_g__Connect_21_2(a, b, ref obj);
										list2.RemoveAt(index);
										num13 = -198840042;
										continue;
										IL_073c:
										num46 = num47;
										num13 = -814361544;
										continue;
										IL_0cd0:
										a = num37;
										num13 = -256981679;
										continue;
										IL_0b54:
										i = num45;
										num34 = 0;
										num13 = -1816689978;
										continue;
										IL_0b38:
										num57 = num52;
										num13 = -1711415250;
										continue;
										IL_0d06:
										item4 = num43;
										if (num41 != 0)
										{
											num13 = -976562726;
											continue;
										}
										(num69, _) = tuple3;
										goto IL_08f7;
										IL_08f7:
										b = num69;
										if (num41 != 0)
										{
											num13 = -1578002874;
											continue;
										}
										num29 = tuple3.Item2;
										goto IL_0a26;
										IL_0bc5:
										num24 = num73;
										num13 = -1980966657;
										continue;
										IL_1007:
										num71 = num21;
										num44 = 0;
										num13 = -1962120596;
										continue;
										IL_0af7:
										j = num35;
										num13 = -245769070;
										continue;
									}
									break;
								}
							}
						}
						IL_03cb:
						flag = (byte)num81 != 0;
						num = -524577349;
						continue;
						IL_0248:
						item5 = num86;
						num = -151445406;
						continue;
					}
					break;
				}
			}
		}

		private void TwoOptOrOptPass(int[] tour, int[] pos)
		{
			Generated___c__DisplayClass22_0 obj = default;
			obj.tour = tour;
			int num5 = default;
			int num6 = default;
			int num10 = default;
			bool flag = default;
			int num3 = default;
			while (true)
			{
				int num = 533358741;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ 0xBD3C92C)) % 18)
					{
					case 16u:
						break;
					default:
						return;
					case 0u:
					{
						int num7;
						int num8;
						if (num5++ < num6)
						{
							num7 = -1071892243;
							num8 = num7;
						}
						else
						{
							num7 = -647331822;
							num8 = num7;
						}
						num = num7 ^ (int)(num2 * 490593734);
						continue;
					}
					case 8u:
						obj.inQueue[num10] = false;
						num = (int)((num2 * 443623898) ^ 0x1F12248E);
						continue;
					case 11u:
					{
						int num9;
						if (!flag)
						{
							num = 1744664683;
							num9 = num;
						}
						else
						{
							num = 1559944432;
							num9 = num;
						}
						continue;
					}
					case 1u:
					{
						int num12;
						if (Token.IsCancellationRequested)
						{
							num = 356831802;
							num12 = num;
						}
						else
						{
							num = 245243309;
							num12 = num;
						}
						continue;
					}
					case 10u:
					{
						flag = Generated__TwoOptOrOptPass_g__TryTwoOpt_22_4(num10, ref obj);
						int num13;
						int num14;
						if (!flag)
						{
							num13 = -644427870;
							num14 = num13;
						}
						else
						{
							num13 = -1326240899;
							num14 = num13;
						}
						num = num13 ^ (int)(num2 * 2052036213);
						continue;
					}
					case 14u:
						num6 = 300 * N + 20000;
						num = (int)((num2 * 2145819939) ^ 0x399E4011);
						continue;
					case 4u:
						flag = Generated__TwoOptOrOptPass_g__TryOrOpt_22_5(num10, ref obj);
						num = ((int)num2 * -1007852328) ^ -351343181;
						continue;
					case 7u:
						obj.pos = pos;
						obj.Generated___4__this = this;
						obj.inQueue = new bool[N];
						num = ((int)num2 * -1950607677) ^ -1569371986;
						continue;
					case 15u:
						obj.queue = new Queue<int>(N);
						num3 = 0;
						num = (int)(num2 * 630196088) ^ -942762509;
						continue;
					case 6u:
						obj.queue.Enqueue(obj.tour[num3]);
						obj.inQueue[obj.tour[num3]] = true;
						num = 1905556915;
						continue;
					case 5u:
					{
						int num11;
						if (num3 >= N)
						{
							num = 1647444481;
							num11 = num;
						}
						else
						{
							num = 8460980;
							num11 = num;
						}
						continue;
					}
					case 9u:
						num10 = obj.queue.Dequeue();
						num = ((int)num2 * -174768557) ^ -1910373859;
						continue;
					case 17u:
						obj.distBuf = new double[Wrapper15(4, Opt.NeighborListSize + 4)];
						num5 = 0;
						num = ((int)num2 * -1169892927) ^ -506898689;
						continue;
					case 12u:
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num10, ref obj);
						num = ((int)num2 * -1289304445) ^ 0x146B2FFF;
						continue;
					case 3u:
					{
						int num4;
						if (obj.queue.Count <= 0)
						{
							num = 356831802;
							num4 = num;
						}
						else
						{
							num = 1518126064;
							num4 = num;
						}
						continue;
					}
					case 13u:
						num3++;
						num = ((int)num2 * -268220976) ^ -1712031237;
						continue;
					case 2u:
						return;
					}
					break;
				}
			}
		}

		private void ReverseSegment(int[] tour, int[] pos, int i, int j)
		{
Trace.Add(new {op="two",i,j,tour=(int[])tour.Clone()});
			int num = (j - i + N) % N + 1;
			int num6 = default;
			int num7 = default;
			int num4 = default;
			int num13 = default;
			int num5 = default;
			while (true)
			{
				int num2 = 640397944;
				while (true)
				{
					uint num3;
					switch ((num3 = (uint)(num2 ^ 0x28071E9C)) % 14)
					{
					case 8u:
						break;
					default:
						return;
					case 13u:
						num6 = (num6 + 1) % N;
						num2 = (int)(num3 * 1520594706) ^ -207416839;
						continue;
					case 3u:
						num7++;
						num2 = (int)((num3 * 1202833200) ^ 0x4415BEFA);
						continue;
					case 2u:
					{
						int num12 = (tour[num6] = tour[num4]);
						tour[num4] = num13;
						pos[num12] = num6;
						pos[num13] = num4;
						num2 = ((int)num3 * -370900281) ^ 0xD2182C7;
						continue;
					}
					case 7u:
					{
						int num9 = (i - 1 + N) % N;
						i = num5;
						j = num9;
						num = N - num;
						num2 = (int)(num3 * 798437936) ^ -1553932214;
						continue;
					}
					case 0u:
						num7 = 0;
						num2 = ((int)num3 * -994511060) ^ 0x3DA8184A;
						continue;
					case 6u:
						num13 = tour[num6];
						num2 = 1609424590;
						continue;
					case 10u:
					{
						int num10;
						int num11;
						if (num * 2 > N)
						{
							num10 = -418925305;
							num11 = num10;
						}
						else
						{
							num10 = -270682642;
							num11 = num10;
						}
						num2 = num10 ^ (int)(num3 * 563653261);
						continue;
					}
					case 4u:
					{
						int num8;
						if (num7 >= num / 2)
						{
							num2 = 813314163;
							num8 = num2;
						}
						else
						{
							num2 = 1248263332;
							num8 = num2;
						}
						continue;
					}
					case 12u:
						num6 = i;
						num2 = 11585405;
						continue;
					case 9u:
						num5 = (j + 1) % N;
						num2 = (int)((num3 * 2068516190) ^ 0x4FAB7F7F);
						continue;
					case 5u:
						num4 = (num4 - 1 + N) % N;
						num2 = (int)((num3 * 155423255) ^ 0x157871E8);
						continue;
					case 1u:
						num4 = j;
						num2 = (int)((num3 * 2098623567) ^ 0x39090DE3);
						continue;
					case 11u:
						return;
					}
					break;
				}
			}
		}

		private void ApplyOrOpt(int[] tour, int[] pos, int segStartPos, int segLen, int insertAfterCity, bool reversed)
		{
Trace.Add(new{op="or",segStartPos,segLen,insertAfterCity,reversed,tour=(int[])tour.Clone()});
			int[] array = new int[segLen];
			int num10 = default;
			int num7 = default;
			int num6 = default;
			int num8 = default;
			int num5 = default;
			int num3 = default;
			int[] array2 = default;
			int num4 = default;
			while (true)
			{
				int num = -1708184648;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ -1142889899)) % 25)
					{
					case 21u:
						break;
					default:
						return;
					case 6u:
						array[num10] = tour[(segStartPos + num10) % N];
						num10++;
						num = -610686059;
						continue;
					case 4u:
						num7++;
						num = -1163262512;
						continue;
					case 11u:
						num7 = 0;
						num = ((int)num2 * -1027835842) ^ 0x1EEAD138;
						continue;
					case 7u:
						num6 = (segStartPos + segLen) % N;
						num = ((int)num2 * -63646689) ^ 0x2BB3C630;
						continue;
					case 0u:
						num8++;
						num = ((int)num2 * -1860298356) ^ 0x1387B23;
						continue;
					case 23u:
						num10 = 0;
						num = ((int)num2 * -1725173527) ^ -492456948;
						continue;
					case 2u:
					{
						int num15;
						if (num8 >= N)
						{
							num = -587380805;
							num15 = num;
						}
						else
						{
							num = -1641996821;
							num15 = num;
						}
						continue;
					}
					case 14u:
						num8 = 0;
						num = ((int)num2 * -1918942288) ^ -1014899387;
						continue;
					case 15u:
						Array.Reverse(array);
						num = ((int)num2 * -1483997918) ^ -1352954129;
						continue;
					case 22u:
					{
						int num11;
						if (num10 >= segLen)
						{
							num = -1387573340;
							num11 = num;
						}
						else
						{
							num = -2122553153;
							num11 = num;
						}
						continue;
					}
					case 9u:
					{
						int num16;
						int num17;
						if (num5 == insertAfterCity)
						{
							num16 = -576332198;
							num17 = num16;
						}
						else
						{
							num16 = -213847016;
							num17 = num16;
						}
						num = num16 ^ ((int)num2 * -1616952823);
						continue;
					}
					case 12u:
					{
						int num13;
						int num14;
						if (reversed)
						{
							num13 = 493358365;
							num14 = num13;
						}
						else
						{
							num13 = 1856276013;
							num14 = num13;
						}
						num = num13 ^ (int)(num2 * 950725998);
						continue;
					}
					case 24u:
					{
						int num12;
						if (num7 >= N - segLen)
						{
							num = -889395697;
							num12 = num;
						}
						else
						{
							num = -1685540847;
							num12 = num;
						}
						continue;
					}
					case 13u:
					{
						int num9;
						if (num3 >= segLen)
						{
							num = -420692973;
							num9 = num;
						}
						else
						{
							num = -250408054;
							num9 = num;
						}
						continue;
					}
					case 10u:
						pos[tour[num8]] = num8;
						num = -686767897;
						continue;
					case 5u:
						Wrapper0((Array)array2, (Array)tour, N);
						num = ((int)num2 * -750261652) ^ 0x5A6CF291;
						continue;
					case 16u:
						num5 = tour[(num6 + num7) % N];
						num = -645024437;
						continue;
					case 18u:
						num = (int)(num2 * 956110296) ^ -438216523;
						continue;
					case 1u:
						array2[num4++] = array[num3];
						num3++;
						num = -1674313990;
						continue;
					case 8u:
						num = (int)((num2 * 1986861762) ^ 0x2C1060DB);
						continue;
					case 19u:
						array2[num4++] = num5;
						num = ((int)num2 * -1700887055) ^ -697215720;
						continue;
					case 17u:
						array2 = new int[N];
						num4 = 0;
						num = -589695710;
						continue;
					case 3u:
						num3 = 0;
						num = (int)((num2 * 878431616) ^ 0x683EC8FA);
						continue;
					case 20u:
						return;
					}
					break;
				}
			}
		}

		private void DoubleBridge(int[] tour, int[] pos, Random rng)
		{
			int n = N;
			int num8 = default;
			int num7 = default;
			int num10 = default;
			int num5 = default;
			int num12 = default;
			int num4 = default;
			int num9 = default;
			int[] array = default;
			int num3 = default;
			int num11 = default;
			while (true)
			{
				int num = -291436950;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ -1924768667)) % 28)
					{
					case 18u:
						break;
					default:
						return;
					case 10u:
					{
						int num16;
						if (num8 >= num7)
						{
							num = -1115210836;
							num16 = num;
						}
						else
						{
							num = -1050533002;
							num16 = num;
						}
						continue;
					}
					case 2u:
						num10 = num5 + 1 + Wrapper2(rng, n - num5 - 1);
						num = (int)((num2 * 231233080) ^ 0x77340194);
						continue;
					case 17u:
						num = ((int)num2 * -1882326614) ^ 0x6241C987;
						continue;
					case 0u:
					{
						int num14;
						if (num12 < n)
						{
							num = -2007195209;
							num14 = num;
						}
						else
						{
							num = -174995152;
							num14 = num;
						}
						continue;
					}
					case 20u:
						num4 = num7;
						num = ((int)num2 * -2074465036) ^ -1480753420;
						continue;
					case 9u:
					{
						int num15;
						if (num9 >= n)
						{
							num = -1552616859;
							num15 = num;
						}
						else
						{
							num = -1793501362;
							num15 = num;
						}
						continue;
					}
					case 7u:
						array[num3++] = tour[num9];
						num9++;
						num = -1378740244;
						continue;
					case 12u:
					{
						int num13;
						if (num11 < num10)
						{
							num = -1361230978;
							num13 = num;
						}
						else
						{
							num = -887960115;
							num13 = num;
						}
						continue;
					}
					case 26u:
						pos[tour[num12]] = num12;
						num = -511316438;
						continue;
					case 4u:
						num5 = num7 + 1 + Wrapper2(rng, n - num7 - 2);
						num = (int)((num2 * 1536727981) ^ 0x10D2AA6B);
						continue;
					case 19u:
						num7 = 1 + Wrapper2(rng, n - 3);
						num = ((int)num2 * -391860352) ^ -282973879;
						continue;
					case 3u:
						array[num3++] = tour[num8];
						num = -1447831189;
						continue;
					case 15u:
						num12++;
						num = ((int)num2 * -673333913) ^ 0x1AE61DEC;
						continue;
					case 14u:
						num = (int)((num2 * 414315794) ^ 0x2619D051);
						continue;
					case 16u:
						Wrapper0((Array)array, (Array)tour, n);
						num12 = 0;
						num = (int)((num2 * 1874626460) ^ 0x6C221A4C);
						continue;
					case 27u:
						array[num3++] = tour[num11];
						num11++;
						num = -1073538427;
						continue;
					case 21u:
						array = new int[n];
						num3 = 0;
						num8 = 0;
						num = ((int)num2 * -158365083) ^ -1248836181;
						continue;
					case 25u:
						num11 = num5;
						num = ((int)num2 * -1030831000) ^ 0x30892657;
						continue;
					case 8u:
						num9 = num10;
						num = ((int)num2 * -1071343372) ^ -1756779829;
						continue;
					case 22u:
						num8++;
						num = (int)((num2 * 1718932693) ^ 0x521E2CD);
						continue;
					case 24u:
					{
						int num6;
						if (num4 >= num5)
						{
							num = -1789775539;
							num6 = num;
						}
						else
						{
							num = -843380726;
							num6 = num;
						}
						continue;
					}
					case 11u:
						num = ((int)num2 * -1985466980) ^ 0x7E2D595F;
						continue;
					case 1u:
						num4++;
						num = ((int)num2 * -1096010390) ^ -1137433633;
						continue;
					case 5u:
						num = (int)((num2 * 1442972656) ^ 0x59F7CC55);
						continue;
					case 6u:
						num = (int)(num2 * 1142661927) ^ -1875135410;
						continue;
					case 23u:
						array[num3++] = tour[num4];
						num = -1432772864;
						continue;
					case 13u:
						return;
					}
					break;
				}
			}
		}

		private double TourLength(int[] tour)
		{
			int num7 = default;
			double num8 = default;
			int num4 = default;
			int num9 = default;
			bool flag = default;
			int num10 = default;
			double num3 = default;

			if (FreeEnd)
			{
				goto IL_000b;
			}
			goto IL_0094;
			IL_000b:
			int num = 1358092205;
			goto IL_0010;
			IL_0010:
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0x660FBE24)) % 24)
				{
				case 12u:
					break;
				case 8u:
					num7++;
					num = 1582446923;
					continue;
				case 6u:
					goto IL_0094;
				case 7u:
					return num8;
				case 2u:
					num8 += Dist(num4, num9);
					num = 1645815212;
					continue;
				case 4u:
					goto IL_00ce;
				case 22u:
					num4 = tour[num7];
					num = 1603172926;
					continue;
				case 14u:
					flag = false;
					num = ((int)num2 * -63864318) ^ -1418476695;
					continue;
				case 18u:
					num9 = tour[(num7 + 1) % N];
					num = ((int)num2 * -866159523) ^ 0x14005C85;
					continue;
				case 15u:
				{
					int num13;
					int num14;
					if (flag)
					{
						num13 = 1101753662;
						num14 = num13;
					}
					else
					{
						num13 = 2143805419;
						num14 = num13;
					}
					num = num13 ^ ((int)num2 * -207010373);
					continue;
				}
				case 23u:
					goto IL_0151;
				case 1u:
					num7 = 0;
					num = ((int)num2 * -2106953860) ^ -674534859;
					continue;
				case 10u:
					return num8 + FreeEndPenalty;
				case 13u:
					num = ((int)num2 * -1233264462) ^ -996430079;
					continue;
				case 9u:
					num8 = 0.0;
					num = (int)(num2 * 1281432833) ^ -1838619021;
					continue;
				case 16u:
					flag = true;
					num = ((int)num2 * -1435980266) ^ 0x12146648;
					continue;
				case 3u:
					goto IL_01e4;
				case 20u:
					num = ((int)num2 * -760753784) ^ -1778608564;
					continue;
				case 5u:
				{
					int num11;
					int num12;
					if (num9 == DummyIndex)
					{
						num11 = 443248038;
						num12 = num11;
					}
					else
					{
						num11 = 1338951935;
						num12 = num11;
					}
					num = num11 ^ ((int)num2 * -1740398531);
					continue;
				}
				case 0u:
					num10++;
					num = (int)((num2 * 492697963) ^ 0x1EF1E630);
					continue;
				case 11u:
					num3 += Dist(tour[num10], tour[(num10 + 1) % N]);
					num = 1110536828;
					continue;
				case 19u:
				{
					int num5;
					int num6;
					if (num4 == DummyIndex)
					{
						num5 = 1479145722;
						num6 = num5;
					}
					else
					{
						num5 = 375573100;
						num6 = num5;
					}
					num = num5 ^ ((int)num2 * -1496390257);
					continue;
				}
				case 21u:
					goto IL_0299;
				default:
					return num3;
				}
				break;
				IL_0299:
				int num15 = num4;
				goto IL_029f;
				IL_01e4:
				if (num4 != DummyIndex)
				{
					num = 1488493433;
					continue;
				}
				num15 = num9;
				goto IL_029f;
				IL_0151:
				int num16;
				if (num7 < N)
				{
					num = 1556388522;
					num16 = num;
				}
				else
				{
					num = 2087791251;
					num16 = num;
				}
				continue;
				IL_00ce:
				int num17;
				if (num10 < N)
				{
					num = 500321271;
					num17 = num;
				}
				else
				{
					num = 1659045605;
					num17 = num;
				}
				continue;
				IL_029f:
				int num18;
				if (num15 == Start)
				{
					num = 1904201692;
					num18 = num;
				}
				else
				{
					num = 1645815212;
					num18 = num;
				}
			}
			goto IL_000b;
			IL_0094:
			num3 = 0.0;
			num10 = 0;
			num = 1879631608;
			goto IL_0010;
		}

		private double PathLength(int[] order)
		{
			double num = 0.0;
			int num4 = default;
			while (true)
			{
				int num2 = 714611048;
				while (true)
				{
					uint num3;
					switch ((num3 = (uint)(num2 ^ 0x3D262855)) % 7)
					{
					case 0u:
						break;
					case 5u:
						num4++;
						num2 = ((int)num3 * -1943538903) ^ -1546958745;
						continue;
					case 3u:
						num2 = ((int)num3 * -1950045174) ^ -990352804;
						continue;
					case 2u:
						num4 = 0;
						num2 = ((int)num3 * -1728857602) ^ -1486353169;
						continue;
					case 4u:
					{
						int num5;
						if (num4 + 1 < order.Length)
						{
							num2 = 1272497972;
							num5 = num2;
						}
						else
						{
							num2 = 851656007;
							num5 = num2;
						}
						continue;
					}
					case 6u:
						num += Dist(order[num4], order[num4 + 1]);
						num2 = 954150654;
						continue;
					default:
						return num;
					}
					break;
				}
			}
		}

		private double RealLength(int[] tour)
		{
			double num = 0.0;
			int num7 = default;
			int num8 = default;
			int num4 = default;
			while (true)
			{
				int num2 = 250892596;
				while (true)
				{
					uint num3;
					switch ((num3 = (uint)(num2 ^ 0x14F5AF21)) % 11)
					{
					case 6u:
						break;
					case 7u:
					{
						int num12;
						int num13;
						if (num7 == DummyIndex)
						{
							num12 = -764674250;
							num13 = num12;
						}
						else
						{
							num12 = -590026884;
							num13 = num12;
						}
						num2 = num12 ^ (int)(num3 * 1255373266);
						continue;
					}
					case 8u:
						num7 = tour[(num8 + 1) % N];
						num2 = (int)((num3 * 1834934869) ^ 0x91E6E62);
						continue;
					case 4u:
						num8++;
						num2 = 549984314;
						continue;
					case 10u:
					{
						int num9;
						if (num8 >= N)
						{
							num2 = 2004380334;
							num9 = num2;
						}
						else
						{
							num2 = 1407052706;
							num9 = num2;
						}
						continue;
					}
					case 0u:
						num += Dist(num4, num7);
						num2 = 1877880182;
						continue;
					case 9u:
						num8 = 0;
						num2 = ((int)num3 * -653422321) ^ 0x7BD0D001;
						continue;
					case 5u:
						num4 = tour[num8];
						num2 = 1272507617;
						continue;
					case 2u:
					{
						int num10;
						int num11;
						if (HasDummy)
						{
							num10 = 274785636;
							num11 = num10;
						}
						else
						{
							num10 = 991415027;
							num11 = num10;
						}
						num2 = num10 ^ ((int)num3 * -727949115);
						continue;
					}
					case 3u:
					{
						int num5;
						int num6;
						if (num4 != DummyIndex)
						{
							num5 = 1714413813;
							num6 = num5;
						}
						else
						{
							num5 = 642059906;
							num6 = num5;
						}
						num2 = num5 ^ ((int)num3 * -240586318);
						continue;
					}
					default:
						return num;
					}
					break;
				}
			}
		}

		private (int[] tour, double len, bool timedOut) RunChain(Random rng, TimeSpan budget, double lowerBound)
		{
			Generated___c__DisplayClass29_0 obj = default;
			obj.Generated___4__this = this;
			obj.lowerBound = lowerBound;
			int[] array = GreedyConstruction();
			Stopwatch stopwatch = default;
			int[] array3 = default;
			int num4 = default;
			int num10 = default;
			double num3 = default;
			double num19 = default;
			int num15 = default;
			int[] array2 = default;
			bool item = default;
			while (true)
			{
				int num = -159089911;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ -444861903)) % 28)
					{
					case 3u:
						break;
					case 23u:
					{
						int num8;
						int num9;
						if (!(Wrapper20(stopwatch) >= budget))
						{
							num8 = -1396937637;
							num9 = num8;
						}
						else
						{
							num8 = -1514080010;
							num9 = num8;
						}
						num = num8 ^ (int)(num2 * 2006980658);
						continue;
					}
					case 7u:
					{
						int num13;
						int num14;
						if (budget != TimeSpan.Zero)
						{
							num13 = 1381216378;
							num14 = num13;
						}
						else
						{
							num13 = 1536461093;
							num14 = num13;
						}
						num = num13 ^ ((int)num2 * -66578376);
						continue;
					}
					case 11u:
					{
						int num18;
						if (Token.IsCancellationRequested)
						{
							num = -1181026626;
							num18 = num;
						}
						else
						{
							num = -2142141214;
							num18 = num;
						}
						continue;
					}
					case 26u:
						array3[array[num4]] = num4;
						num4++;
						num = -196114265;
						continue;
					case 20u:
						array3 = new int[N];
						num = (int)((num2 * 381165660) ^ 0x43B2D77B);
						continue;
					case 14u:
					{
						int num23;
						if (num10 >= N)
						{
							num = -1639851426;
							num23 = num;
						}
						else
						{
							num = -878069028;
							num23 = num;
						}
						continue;
					}
					case 21u:
						num10++;
						num = ((int)num2 * -1513017630) ^ 0x60266D69;
						continue;
					case 0u:
						num3 = num19;
						num = (int)((num2 * 1729022395) ^ 0x286A1494);
						continue;
					case 4u:
					{
						DoubleBridge(array, array3, rng);
						TwoOptOrOptPass(array, array3);
						num19 = TourLength(array);
						int num20;
						if (num19 < num3 - 1E-09)
						{
							num = -1293639347;
							num20 = num;
						}
						else
						{
							num = -149393289;
							num20 = num;
						}
						continue;
					}
					case 9u:
						num10 = 0;
						num = ((int)num2 * -251436894) ^ -1538538946;
						continue;
					case 19u:
					{
						int num6;
						int num7;
						if (!Generated__RunChain_g__GoodEnough_29_0(num3, ref obj))
						{
							num6 = -625434675;
							num7 = num6;
						}
						else
						{
							num6 = -1563793097;
							num7 = num6;
						}
						num = num6 ^ ((int)num2 * -442038905);
						continue;
					}
					case 2u:
						num = (int)((num2 * 1434113773) ^ 0x2CDB10C1);
						continue;
					case 27u:
					{
						int num21;
						int num22;
						if (++num15 >= Opt.StagnationLimit)
						{
							num21 = -1969076742;
							num22 = num21;
						}
						else
						{
							num21 = -1983872822;
							num22 = num21;
						}
						num = num21 ^ (int)(num2 * 244087548);
						continue;
					}
					case 17u:
						Wrapper0((Array)array, (Array)array2, N);
						num = (int)((num2 * 240741469) ^ 0x425AE898);
						continue;
					case 24u:
					{
						num15 = 0;
						int num16;
						int num17;
						if (!Generated__RunChain_g__GoodEnough_29_0(num3, ref obj))
						{
							num16 = -2004203090;
							num17 = num16;
						}
						else
						{
							num16 = -222219143;
							num17 = num16;
						}
						num = num16 ^ (int)(num2 * 1534206920);
						continue;
					}
					case 13u:
						item = true;
						num = ((int)num2 * -278013585) ^ -712107935;
						continue;
					case 1u:
						stopwatch = Wrapper19();
						num15 = 0;
						num = ((int)num2 * -1389281388) ^ -1116748502;
						continue;
					case 8u:
					{
						item = false;
						int num11;
						int num12;
						if (N < 8)
						{
							num11 = -1845807454;
							num12 = num11;
						}
						else
						{
							num11 = -318421950;
							num12 = num11;
						}
						num = num11 ^ (int)(num2 * 1729531165);
						continue;
					}
					case 5u:
						num = ((int)num2 * -1717646985) ^ 0x78E04BC8;
						continue;
					case 6u:
						num4 = 0;
						num = ((int)num2 * -1905194917) ^ -478172767;
						continue;
					case 22u:
						Wrapper0((Array)array2, (Array)array, N);
						num = -202289916;
						continue;
					case 25u:
						array3[array[num10]] = num10;
						num = -2111059464;
						continue;
					case 18u:
						num3 = TourLength(array);
						array2 = (int[])Wrapper6((Array)array);
						num = (int)((num2 * 1681772973) ^ 0x5754196B);
						continue;
					case 12u:
						num = ((int)num2 * -1190925473) ^ 0x1E339426;
						continue;
					case 16u:
						TwoOptOrOptPass(array, array3);
						num = (int)(num2 * 1055134670) ^ -1984559241;
						continue;
					case 10u:
					{
						int num5;
						if (num4 < N)
						{
							num = -159731489;
							num5 = num;
						}
						else
						{
							num = -1089972131;
							num5 = num;
						}
						continue;
					}
					default:
						return (tour: array2, len: num3, timedOut: item);
					}
					break;
				}
			}
		}

		private int[] ExtractOrder(int[] tour)
		{
			List<int> list3 = default;
			List<int> list2 = default;
			int num7 = default;
			int num12 = default;
			int[] array = default;
			int num10 = default;
			int num11 = default;
			int num4 = default;
			int num3 = default;

			if (HasDummy)
			{
				goto IL_000b;
			}
			goto IL_0346;
			IL_000b:
			int num = 1538476770;
			goto IL_0010;
			IL_0010:
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0x84BEE90)) % 26)
				{
				case 15u:
					break;
				case 18u:
					return list3.ToArray();
				case 22u:
					list2 = new List<int>(N - 1);
					num7 = 1;
					num = ((int)num2 * -2082669882) ^ 0x3961EB73;
					continue;
				case 21u:
					goto IL_00cc;
				case 2u:
					num12 = list2.IndexOf(Start);
					num = ((int)num2 * -900250428) ^ -1431524935;
					continue;
				case 13u:
				{
					int num8;
					int num9;
					if (!FreeEnd)
					{
						num8 = -2143531423;
						num9 = num8;
					}
					else
					{
						num8 = -187068455;
						num9 = num8;
					}
					num = num8 ^ ((int)num2 * -1342798012);
					continue;
				}
				case 19u:
					goto IL_012e;
				case 16u:
					num7++;
					num = ((int)num2 * -655898775) ^ 0x5A628CFF;
					continue;
				case 24u:
					array[num10] = tour[(num11 + num10) % N];
					num = 443002431;
					continue;
				case 20u:
					num10 = 0;
					num = ((int)num2 * -1073024751) ^ 0x8CDC39A;
					continue;
				case 5u:
					list2.Add(tour[(num4 + num7) % N]);
					num = 574811168;
					continue;
				case 9u:
					list2.Reverse();
					num = ((int)num2 * -774833547) ^ -891550323;
					continue;
				case 6u:
					num = ((int)num2 * -130097087) ^ -1811205561;
					continue;
				case 23u:
				{
					int num13;
					int num14;
					if (list2[0] == Start)
					{
						num13 = -1211124928;
						num14 = num13;
					}
					else
					{
						num13 = -617264193;
						num14 = num13;
					}
					num = num13 ^ (int)(num2 * 242405065);
					continue;
				}
				case 0u:
					goto IL_020f;
				case 8u:
					return list2.ToArray();
				case 1u:
					list3.Add(list2[(num12 + num3) % list2.Count]);
					num = 6432330;
					continue;
				case 14u:
					array = new int[N];
					num = (int)(num2 * 1075361806) ^ -152412920;
					continue;
				case 25u:
					goto IL_0280;
				case 17u:
					num10++;
					num = (int)(num2 * 17883139) ^ -951733576;
					continue;
				case 11u:
					list3 = new List<int>(list2.Count);
					num3 = 0;
					num = (int)((num2 * 1555345715) ^ 0x574C301F);
					continue;
				case 10u:
				{
					List<int> list = list2;
					int num5;
					int num6;
					if (list[list.Count - 1] != Start)
					{
						num5 = 1292335264;
						num6 = num5;
					}
					else
					{
						num5 = 259369113;
						num6 = num5;
					}
					num = num5 ^ (int)(num2 * 597868538);
					continue;
				}
				case 12u:
					num4 = Array.IndexOf(tour, DummyIndex);
					num = (int)(num2 * 390131994) ^ -1151729650;
					continue;
				case 4u:
					num3++;
					num = ((int)num2 * -1290195362) ^ -1737523384;
					continue;
				case 3u:
					goto IL_0346;
				default:
					return array;
				}
				break;
				IL_0280:
				int num15;
				if (num10 < N)
				{
					num = 93611254;
					num15 = num;
				}
				else
				{
					num = 114952109;
					num15 = num;
				}
				continue;
				IL_012e:
				int num16;
				if (list2[0] != Start)
				{
					num = 1931383113;
					num16 = num;
				}
				else
				{
					num = 752967072;
					num16 = num;
				}
				continue;
				IL_00cc:
				int num17;
				if (num7 >= N)
				{
					num = 1762917059;
					num17 = num;
				}
				else
				{
					num = 1977758709;
					num17 = num;
				}
				continue;
				IL_020f:
				int num18;
				if (num3 < list2.Count)
				{
					num = 1053552751;
					num18 = num;
				}
				else
				{
					num = 693137614;
					num18 = num;
				}
			}
			goto IL_000b;
			IL_0346:
			num11 = Array.IndexOf(tour, Start);
			num = 1405394112;
			goto IL_0010;
		}

		public Result Run()
		{
			int num5 = default;
			int num3 = default;
			double num11 = default;
			(int[] tour, double len, bool timedOut)[] results = default;
			int num4 = default;
			double lowerBound = default;
			int seedBase = default;
			double? estimatedGapPercent = default;
			int[] order = default;
			double length = default;
			while (true)
			{
				int num = 1599120182;
				while (true)
				{
					uint num2;
					int num7;
					double num8;
					switch ((num2 = (uint)(num ^ 0x416A26B6)) % 19)
					{
					case 3u:
						break;
					case 9u:
						num5++;
						num = 888285471;
						continue;
					case 11u:
						num3 = 0;
						num11 = double.MaxValue;
						num5 = 0;
						num = 888285471;
						continue;
					case 14u:
						results = new (int[], double, bool)[num4];
						num = (int)(num2 * 1046630991) ^ -1609900742;
						continue;
					case 1u:
						lowerBound = ComputeLowerBound();
						num = ((int)num2 * -852109658) ^ -1220761295;
						continue;
					case 16u:
						results[0] = RunChain(Wrapper14(seedBase), Opt.TimeLimit, lowerBound);
						num = (int)((num2 * 1996476320) ^ 0x296CE2FB);
						continue;
					case 10u:
						num7 = 1;
						goto IL_012d;
					case 5u:
						estimatedGapPercent = ((lowerBound > 1E-09) ? ((num11 - lowerBound) / lowerBound * 100.0) : 0.0);
						num = 1356275665;
						continue;
					case 18u:
						if (!FreeEnd)
						{
							num = (int)((num2 * 238410367) ^ 0xC330827);
							continue;
						}
						num8 = PathLength(order);
						goto IL_02b3;
					case 15u:
						if (Opt.ParallelRestarts)
						{
							num7 = Wrapper15(1, Opt.ParallelWorkers);
							goto IL_012d;
						}
						num = ((int)num2 * -17374871) ^ 0x74B813AC;
						continue;
					case 17u:
					{
						int num12;
						if (results[num5].len < num11)
						{
							num = 1537608094;
							num12 = num;
						}
						else
						{
							num = 1311035939;
							num12 = num;
						}
						continue;
					}
					case 12u:
						num11 = results[num5].len;
						num3 = num5;
						num = ((int)num2 * -1027994311) ^ -1496361525;
						continue;
					case 7u:
					{
						int num9;
						int num10;
						if (num4 == 1)
						{
							num9 = 759184937;
							num10 = num9;
						}
						else
						{
							num9 = 2005795985;
							num10 = num9;
						}
						num = num9 ^ (int)(num2 * 373645642);
						continue;
					}
					case 0u:
						order = ExtractOrder(results[num3].tour);
						num = (int)((num2 * 1656404414) ^ 0xD91982);
						continue;
					case 6u:
						seedBase = Opt.RandomSeed ?? Wrapper8();
						num = 679512459;
						continue;
					case 4u:
						num8 = RealLength(results[num3].tour);
						goto IL_02b3;
					case 2u:
					{
						int num6;
						if (num5 < num4)
						{
							num = 1691034345;
							num6 = num;
						}
						else
						{
							num = 1025926759;
							num6 = num;
						}
						continue;
					}
					case 13u:
						Wrapper5(0, num4, (Action<int>)((int w) =>
						{
							Random rng = Generated___c__DisplayClass31_0.Wrapper17(seedBase + w * 7919 + 17);
							results[w] = RunChain(rng, Opt.TimeLimit, lowerBound);
						}));
						num = 665656283;
						continue;
					default:
						{
							return new Result(order, length, estimatedGapPercent, results[num3].timedOut);
						}
						IL_012d:
						num4 = num7;
						num = 1931338021;
						continue;
						IL_02b3:
						length = num8;
						num = 752071552;
						continue;
					}
					break;
				}
			}
		}

		[CompilerGenerated]
		private static int Generated__GreedyConstruction_g__Find_21_1(int x, ref Generated___c__DisplayClass21_0 P_1)
		{
			while (true)
			{
				int num;
				int num2;
				if (P_1.parent[x] == x)
				{
					num = -136626512;
					num2 = num;
				}
				else
				{
					num = -543120636;
					num2 = num;
				}
				while (true)
				{
					uint num3;
					switch ((num3 = (uint)(num ^ -641988248)) % 5)
					{
					case 4u:
						num = -543120636;
						continue;
					case 2u:
						P_1.parent[x] = P_1.parent[P_1.parent[x]];
						num = -739423943;
						continue;
					case 3u:
						break;
					case 1u:
						x = P_1.parent[x];
						num = ((int)num3 * -506636781) ^ 0x21B1B93D;
						continue;
					default:
						return x;
					}
					break;
				}
			}
		}

		[CompilerGenerated]
		private static void Generated__GreedyConstruction_g__Connect_21_2(int a, int b, ref Generated___c__DisplayClass21_0 P_2)
		{
			if (P_2.adj[a, 0] == -1)
			{
				goto IL_0013;
			}
			goto IL_0102;
			IL_0013:
			int num = -990076723;
			goto IL_0018;
			IL_0018:
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ -706905323)) % 9)
				{
				case 0u:
					break;
				default:
					return;
				case 3u:
					P_2.adj[b, 0] = a;
					num = ((int)num2 * -219450607) ^ -51790196;
					continue;
				case 6u:
					goto IL_006e;
				case 4u:
					P_2.degree[b]++;
					P_2.parent[Generated__GreedyConstruction_g__Find_21_1(a, ref P_2)] = Generated__GreedyConstruction_g__Find_21_1(b, ref P_2);
					num = (int)(num2 * 691522142) ^ -420186144;
					continue;
				case 8u:
					P_2.adj[a, 0] = b;
					num = ((int)num2 * -396160725) ^ 0x283B5CB2;
					continue;
				case 2u:
					P_2.degree[a]++;
					num = -1366024857;
					continue;
				case 1u:
					goto IL_0102;
				case 5u:
					P_2.adj[b, 1] = a;
					num = -873486667;
					continue;
				case 7u:
					return;
				}
				break;
				IL_006e:
				int num3;
				if (P_2.adj[b, 0] == -1)
				{
					num = -2139326020;
					num3 = num;
				}
				else
				{
					num = -42443795;
					num3 = num;
				}
			}
			goto IL_0013;
			IL_0102:
			P_2.adj[a, 1] = b;
			num = -857073158;
			goto IL_0018;
		}

		[CompilerGenerated]
		private int Generated__TwoOptOrOptPass_g__NextOf_22_0(int c, ref Generated___c__DisplayClass22_0 P_1)
		{
			return P_1.tour[(P_1.pos[c] + 1) % N];
		}

		[CompilerGenerated]
		private int Generated__TwoOptOrOptPass_g__PrevOf_22_1(int c, ref Generated___c__DisplayClass22_0 P_1)
		{
			return P_1.tour[(P_1.pos[c] - 1 + N) % N];
		}

		[CompilerGenerated]
		private void Generated__TwoOptOrOptPass_g__Enqueue_22_2(int c, ref Generated___c__DisplayClass22_0 P_1)
		{
			if (P_1.inQueue[c])
			{
				return;
			}
			while (true)
			{
				int num = -1448770681;
				while (true)
				{
					uint num2;
					switch ((num2 = (uint)(num ^ -1237860600)) % 3)
					{
					case 0u:
						break;
					default:
						return;
					case 2u:
						goto IL_002c;
					case 1u:
						return;
					}
					break;
					IL_002c:
					P_1.inQueue[c] = true;
					P_1.queue.Enqueue(c);
					num = ((int)num2 * -1592930656) ^ -1295621381;
				}
			}
		}

		[CompilerGenerated]
		private bool Generated__TwoOptOrOptPass_g__InSegment_22_3(int city, int segStartPos, int segLen, ref Generated___c__DisplayClass22_0 P_3)
		{
			return (P_3.pos[city] - segStartPos + N) % N < segLen;
		}

		[CompilerGenerated]
		private bool Generated__TwoOptOrOptPass_g__TryTwoOpt_22_4(int c1, ref Generated___c__DisplayClass22_0 P_1)
		{
			int num = Generated__TwoOptOrOptPass_g__PrevOf_22_1(c1, ref P_1);
			int num8 = default;
			int[] array = default;
			int num5 = default;
			int num6 = default;
			int num11 = default;
			double num14 = default;
			double num13 = default;
			int num9 = default;
			int num10 = default;
			while (true)
			{
				int num2 = 888513197;
				while (true)
				{
					uint num3;
					int num15;
					int num4;
					int num12;
					switch ((num3 = (uint)(num2 ^ 0x13D8D691)) % 33)
					{
					case 23u:
						break;
					case 22u:
						num2 = ((int)num3 * -1856732753) ^ -1265292001;
						continue;
					case 26u:
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num8, ref P_1);
						num2 = ((int)num3 * -532017479) ^ -1414126145;
						continue;
					case 9u:
						P_1.distBuf = new double[array.Length];
						num2 = ((int)num3 * -1401563675) ^ 0x57915632;
						continue;
					case 18u:
					{
						int num28;
						int num29;
						if (num5 == c1)
						{
							num28 = -979019998;
							num29 = num28;
						}
						else
						{
							num28 = -36831976;
							num29 = num28;
						}
						num2 = num28 ^ ((int)num3 * -108328712);
						continue;
					}
					case 17u:
					{
						num5 = array[num6];
						int num22;
						if (num5 == num11)
						{
							num2 = 238595818;
							num22 = num2;
						}
						else
						{
							num2 = 796576310;
							num22 = num2;
						}
						continue;
					}
					case 31u:
					{
						int num25;
						int num26;
                        if (num14 + Dist(num5, num8) - (num13 + Dist(num11, num8)) > 1E-10)
						{
							num25 = -1531112721;
							num26 = num25;
						}
						else
						{
							num25 = -1155191382;
							num26 = num25;
						}
						num2 = num25 ^ ((int)num3 * -180095816);
						continue;
					}
					case 11u:
					{
						int num20;
						int num21;
						if (num8 == num5)
						{
							num20 = 704938432;
							num21 = num20;
						}
						else
						{
							num20 = 1104133731;
							num21 = num20;
						}
						num2 = num20 ^ (int)(num3 * 1033537734);
						continue;
					}
					case 16u:
						return true;
					case 24u:
						num15 = num9;
						goto IL_01b9;
					case 32u:
						num2 = (int)(num3 * 910783141) ^ -879121071;
						continue;
					case 14u:
						num10 = 0;
						num2 = ((int)num3 * -1160806035) ^ -1492097217;
						continue;
					case 19u:
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(c1, ref P_1);
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num11, ref P_1);
						num2 = 1926749903;
						continue;
					case 27u:
						num14 = Dist(c1, num11);
						num6 = 0;
						num2 = ((int)num3 * -611056549) ^ 0x3B1140E3;
						continue;
					case 25u:
						num13 = P_1.distBuf[num6];
						num2 = ((int)num3 * -111176796) ^ -892509687;
						continue;
					case 30u:
						BatchDistancesFrom(c1, array, P_1.distBuf);
						num2 = 27299781;
						continue;
					case 13u:
						ReverseSegment(P_1.tour, P_1.pos, P_1.pos[num5], P_1.pos[num11]);
						num2 = (int)((num3 * 1105142475) ^ 0x2A4D8E0D);
						continue;
					case 29u:
						num9 = Generated__TwoOptOrOptPass_g__NextOf_22_0(c1, ref P_1);
						num2 = ((int)num3 * -2062374975) ^ -20216919;
						continue;
					case 6u:
					{
						int num27;
						if (num10 < 2)
						{
							num2 = 1165323538;
							num27 = num2;
						}
						else
						{
							num2 = 1416833894;
							num27 = num2;
						}
						continue;
					}
					case 12u:
					{
						int num23;
						int num24;
						if (num10 == 0)
						{
							num23 = -99074311;
							num24 = num23;
						}
						else
						{
							num23 = -294871891;
							num24 = num23;
						}
						num2 = num23 ^ (int)(num3 * 1207042179);
						continue;
					}
					case 5u:
					{
						int num18;
						int num19;
						if (!(num13 < num14))
						{
							num18 = 604539259;
							num19 = num18;
						}
						else
						{
							num18 = 271739952;
							num19 = num18;
						}
						num2 = num18 ^ (int)(num3 * 588348754);
						continue;
					}
					case 10u:
						ReverseSegment(P_1.tour, P_1.pos, P_1.pos[num11], P_1.pos[num5]);
						num2 = 1031241741;
						continue;
					case 3u:
					{
						int num16;
						int num17;
						if (P_1.distBuf.Length >= array.Length)
						{
							num16 = -2127601556;
							num17 = num16;
						}
						else
						{
							num16 = -701209249;
							num17 = num16;
						}
						num2 = num16 ^ (int)(num3 * 1500040655);
						continue;
					}
					case 28u:
						if (num10 == 0)
						{
							num15 = num;
							goto IL_01b9;
						}
						num2 = 2138915779;
						continue;
					case 0u:
						num2 = (int)((num3 * 211009857) ^ 0x3D130697);
						continue;
					case 4u:
						num6++;
						num2 = 1746119458;
						continue;
					case 20u:
						array = Neighbors[c1];
						num2 = (int)((num3 * 1592276431) ^ 0x24311D83);
						continue;
					case 8u:
						num10++;
						num2 = 486709353;
						continue;
					case 2u:
						if (num10 != 0)
						{
							num2 = ((int)num3 * -2142314301) ^ -728419476;
							continue;
						}
						num4 = Generated__TwoOptOrOptPass_g__PrevOf_22_1(num5, ref P_1);
						goto IL_0423;
					case 15u:
					{
						int num7;
						if (num6 < array.Length)
						{
							num2 = 581097209;
							num7 = num2;
						}
						else
						{
							num2 = 1906390595;
							num7 = num2;
						}
						continue;
					}
					case 1u:
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num5, ref P_1);
						num2 = (int)(num3 * 1693355285) ^ -1963840133;
						continue;
					case 21u:
						num4 = Generated__TwoOptOrOptPass_g__NextOf_22_0(num5, ref P_1);
						goto IL_0423;
					default:
						{
							return false;
						}
						IL_0423:
						num8 = num4;
						if (num8 == c1)
						{
							num2 = 238595818;
							num12 = num2;
						}
						else
						{
							num2 = 1451089718;
							num12 = num2;
						}
						continue;
						IL_01b9:
						num11 = num15;
						num2 = 847688270;
						continue;
					}
					break;
				}
			}
		}

		[CompilerGenerated]
		private bool Generated__TwoOptOrOptPass_g__TryOrOpt_22_5(int c1, ref Generated___c__DisplayClass22_0 P_1)
		{
			int num = 1;
			int num27 = default;
			int num5 = default;
			int num6 = default;
			int num8 = default;
			int num26 = default;
			int[] array = default;
			int num7 = default;
			int num13 = default;
			double num15 = default;
			double num31 = default;
			int num14 = default;
			double num18 = default;
			int num9 = default;
			while (true)
			{
				int num2 = 1018012577;
				while (true)
				{
					uint num3;
					object obj;
					switch ((num3 = (uint)(num2 ^ 0x158B96B7)) % 37)
					{
					case 14u:
						break;
					case 21u:
						num27++;
						num2 = ((int)num3 * -1398370579) ^ 0x773B33A9;
						continue;
					case 30u:
						num5 = P_1.pos[c1];
						num2 = 1790843594;
						continue;
					case 3u:
						ApplyOrOpt(P_1.tour, P_1.pos, num5, num, num6, reversed: true);
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num8, ref P_1);
						num2 = (int)(num3 * 734525697) ^ -1975945302;
						continue;
					case 15u:
					{
						int num29;
						int num30;
						if (num < N - 2)
						{
							num29 = -1835402834;
							num30 = num29;
						}
						else
						{
							num29 = -1805714115;
							num30 = num29;
						}
						num2 = num29 ^ ((int)num3 * -403906420);
						continue;
					}
					case 9u:
						num26 = (num5 + num - 1) % N;
						num2 = ((int)num3 * -775270224) ^ 0x4401149E;
						continue;
					case 32u:
					{
						num6 = array[num7];
						int num12;
						if (!Generated__TwoOptOrOptPass_g__InSegment_22_3(num6, num5, num, ref P_1))
						{
							num2 = 1729046110;
							num12 = num2;
						}
						else
						{
							num2 = 783840610;
							num12 = num2;
						}
						continue;
					}
					case 25u:
					{
						double num36 = Dist(num6, num13);
						num15 = num31 - (Dist(num6, c1) + Dist(num14, num13) - num36);
						num18 = num31 - (Dist(num6, num14) + Dist(c1, num13) - num36);
						num2 = (int)(num3 * 692970957) ^ -1770624167;
						continue;
					}
					case 35u:
					{
						int num24;
						int num25;
						if (num6 != num8)
						{
							num24 = 396073333;
							num25 = num24;
						}
						else
						{
							num24 = 682584588;
							num25 = num24;
						}
						num2 = num24 ^ ((int)num3 * -1849839810);
						continue;
					}
					case 31u:
						num7 = 0;
						num2 = ((int)num3 * -145489154) ^ -2115859894;
						continue;
					case 11u:
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num13, ref P_1);
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(c1, ref P_1);
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num14, ref P_1);
						num2 = (int)((num3 * 1518144672) ^ 0x57C3676B);
						continue;
					case 2u:
					{
						num9 = Generated__TwoOptOrOptPass_g__NextOf_22_0(num14, ref P_1);
						int num38;
						int num39;
						if (num8 != num14)
						{
							num38 = 1842317897;
							num39 = num38;
						}
						else
						{
							num38 = 229276115;
							num39 = num38;
						}
						num2 = num38 ^ (int)(num3 * 1595165039);
						continue;
					}
					case 7u:
					{
						int num32;
						int num33;
						if (num9 == c1)
						{
							num32 = -1171049785;
							num33 = num32;
						}
						else
						{
							num32 = -1555116687;
							num33 = num32;
						}
						num2 = num32 ^ ((int)num3 * -1460485085);
						continue;
					}
					case 16u:
						num31 = Dist(num8, c1) + Dist(num14, num9) - Dist(num8, num9);
						num2 = (int)((num3 * 1409637522) ^ 0x19E966F);
						continue;
					case 18u:
					{
						int num22;
						int num23;
						if (num15 < num18)
						{
							num22 = -1888255491;
							num23 = num22;
						}
						else
						{
							num22 = -483359092;
							num23 = num22;
						}
						num2 = num22 ^ (int)(num3 * 766023542);
						continue;
					}
					case 24u:
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num14, ref P_1);
						return true;
					case 0u:
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num8, ref P_1);
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num9, ref P_1);
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num6, ref P_1);
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num13, ref P_1);
						num2 = (int)((num3 * 823101434) ^ 0x73490B77);
						continue;
					case 27u:
					{
						int num16;
						int num17;
						if (num15 > 1E-10)
						{
							num16 = -1249002271;
							num17 = num16;
						}
						else
						{
							num16 = -1490819078;
							num17 = num16;
						}
						num2 = num16 ^ (int)(num3 * 2097309933);
						continue;
					}
					case 26u:
						num7++;
						num2 = 1713918002;
						continue;
					case 29u:
						ApplyOrOpt(P_1.tour, P_1.pos, num5, num, num6, reversed: false);
						num2 = ((int)num3 * -683663556) ^ -1334742178;
						continue;
					case 22u:
						obj = Neighbors[num14];
						goto IL_03a8;
					case 10u:
					{
						int num37;
						if (num7 < array.Length)
						{
							num2 = 1687856799;
							num37 = num2;
						}
						else
						{
							num2 = 1619964696;
							num37 = num2;
						}
						continue;
					}
					case 6u:
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(c1, ref P_1);
						num2 = (int)((num3 * 1706434246) ^ 0x2C9C9860);
						continue;
					case 23u:
						num2 = (int)(num3 * 1490578073) ^ -2112188171;
						continue;
					case 34u:
					{
						int num34;
						int num35;
						if (!(num31 <= 1E-10))
						{
							num34 = -531818524;
							num35 = num34;
						}
						else
						{
							num34 = -1005139216;
							num35 = num34;
						}
						num2 = num34 ^ (int)(num3 * 472450776);
						continue;
					}
					case 17u:
						num27 = 0;
						num2 = ((int)num3 * -1702596082) ^ -832783200;
						continue;
					case 19u:
						num++;
						num2 = 1182418643;
						continue;
					case 12u:
						if (num27 == 0)
						{
							obj = Neighbors[c1];
							goto IL_03a8;
						}
						num2 = 315371923;
						continue;
					case 13u:
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num9, ref P_1);
						Generated__TwoOptOrOptPass_g__Enqueue_22_2(num6, ref P_1);
						num2 = ((int)num3 * -1663352701) ^ 0x60D5D59B;
						continue;
					case 20u:
					{
						int num28;
						if (num27 >= 2)
						{
							num2 = 131324208;
							num28 = num2;
						}
						else
						{
							num2 = 689927477;
							num28 = num2;
						}
						continue;
					}
					case 28u:
						num14 = P_1.tour[num26];
						num8 = Generated__TwoOptOrOptPass_g__PrevOf_22_1(c1, ref P_1);
						num2 = ((int)num3 * -2090437848) ^ 0x485BFF92;
						continue;
					case 5u:
					{
						num13 = Generated__TwoOptOrOptPass_g__NextOf_22_0(num6, ref P_1);
						int num20;
						int num21;
						if (Generated__TwoOptOrOptPass_g__InSegment_22_3(num13, num5, num, ref P_1))
						{
							num20 = -1685467802;
							num21 = num20;
						}
						else
						{
							num20 = -1907997056;
							num21 = num20;
						}
						num2 = num20 ^ (int)(num3 * 1793926275);
						continue;
					}
					case 36u:
						return true;
					case 4u:
					{
						int num19;
						if (num18 <= 1E-10)
						{
							num2 = 783840610;
							num19 = num2;
						}
						else
						{
							num2 = 1032282923;
							num19 = num2;
						}
						continue;
					}
					case 1u:
					{
						int num10;
						int num11;
						if (num8 == num9)
						{
							num10 = -1828358805;
							num11 = num10;
						}
						else
						{
							num10 = -1097530444;
							num11 = num10;
						}
						num2 = num10 ^ (int)(num3 * 2043113035);
						continue;
					}
					case 33u:
					{
						int num4;
						if (num > 3)
						{
							num2 = 487965737;
							num4 = num2;
						}
						else
						{
							num2 = 1048462528;
							num4 = num2;
						}
						continue;
					}
					default:
						{
							return false;
						}
						IL_03a8:
						array = (int[])obj;
						num2 = 945565299;
						continue;
					}
					break;
				}
			}
		}

		[CompilerGenerated]
		private bool Generated__RunChain_g__GoodEnough_29_0(double len, ref Generated___c__DisplayClass29_0 P_1)
		{
			if (Opt.TargetGapPercent.HasValue)
			{
				while (true)
				{
					int num = -1736012881;
					while (true)
					{
						uint num2;
						switch ((num2 = (uint)(num ^ -557848800)) % 4)
						{
						case 0u:
							break;
						case 3u:
						{
							int num3;
							int num4;
							if (P_1.lowerBound > 1E-09)
							{
								num3 = 240162671;
								num4 = num3;
							}
							else
							{
								num3 = 2083914192;
								num4 = num3;
							}
							num = num3 ^ (int)(num2 * 400164075);
							continue;
						}
						case 2u:
							return (len - P_1.lowerBound) / P_1.lowerBound * 100.0 <= Opt.TargetGapPercent.Value;
						default:
							goto end_IL_0018;
						}
						break;
					}
					continue;
					end_IL_0018:
					break;
				}
			}
			return false;
		}

		static double Wrapper1(double P_0, double P_1)
		{
			return Math.Min(P_0, P_1);
		}

		static double Wrapper18(double P_0, double P_1)
		{
			return Math.Max(P_0, P_1);
		}

		static double Wrapper3(double P_0)
		{
			return Math.Sqrt(P_0);
		}

		static int Wrapper12(int P_0, int P_1)
		{
			return Math.Min(P_0, P_1);
		}

		static int Wrapper15(int P_0, int P_1)
		{
			return Math.Max(P_0, P_1);
		}

		static void Wrapper0(Array P_0, Array P_1, int P_2)
		{
			Array.Copy(P_0, P_1, P_2);
		}

		static int Wrapper2(Random P_0, int P_1)
		{
			return P_0.Next(P_1);
		}

		static object Wrapper6(Array P_0)
		{
			return P_0.Clone();
		}

		static Stopwatch Wrapper19()
		{
			return Stopwatch.StartNew();
		}

		static TimeSpan Wrapper20(Stopwatch P_0)
		{
			return P_0.Elapsed;
		}

		static int Wrapper8()
		{
			return Environment.TickCount;
		}

		public static Random Wrapper14(int P_0)
		{
			return new Random(P_0);
		}

		static ParallelLoopResult Wrapper5(int P_0, int P_1, Action<int> P_2)
		{
			return Parallel.For(P_0, P_1, P_2);
		}
	}

	[StructLayout(LayoutKind.Auto)]
	[CompilerGenerated]
	private struct Generated___c__DisplayClass4_0
	{
		public IReadOnlyList<Vector3> points;
	}

	[MethodImpl(MethodImplOptions.NoInlining)]
	public static Result Solve(IReadOnlyList<Vector3> points, int start, int? end = null, Options? options = null, CancellationToken cancellationToken = default(CancellationToken))
	{
		int count = default;
		int[] array = default;
		Result result = default;

		if (points == null)
		{
			goto IL_0006;
		}
		goto IL_008a;
		IL_0006:
		int num = -170550880;
		goto IL_000b;
		IL_000b:
		while (true)
		{
			uint num2;
			int num9;
			bool flag;
			bool freeEnd;
			switch ((num2 = (uint)(num ^ -1081990331)) % 20)
			{
			case 19u:
				break;
			case 11u:
				throw Wrapper11("start");
			case 13u:
				goto IL_008a;
			case 18u:
				if (end.HasValue)
				{
					num = -1716764831;
					continue;
				}
				num9 = 0;
				goto IL_01b5;
			case 17u:
				goto IL_00b1;
			case 10u:
				throw Wrapper7("Need at least 2 points.", "points");
			case 7u:
				goto IL_00f6;
			case 9u:
				throw Wrapper11("end");
			case 12u:
			{
				int num5;
				int num6;
				if (count >= 2)
				{
					num5 = -1550858678;
					num6 = num5;
				}
				else
				{
					num5 = -1969710649;
					num6 = num5;
				}
				num = num5 ^ ((int)num2 * -1254348598);
				continue;
			}
			case 16u:
				options = new Options();
				num = ((int)num2 * -2027954946) ^ -1075518281;
				continue;
			case 1u:
				throw Wrapper4("points");
			case 0u:
			{
				int num10;
				int num11;
				if (end.Value >= 0)
				{
					num10 = -1902711033;
					num11 = num10;
				}
				else
				{
					num10 = -1256479196;
					num11 = num10;
				}
				num = num10 ^ ((int)num2 * -762194484);
				continue;
			}
			case 8u:
				num9 = ((end.Value == start) ? 1 : 0);
				goto IL_01b5;
			case 14u:
			{
				int num7;
				int num8;
				if (start < count)
				{
					num7 = -2043195990;
					num8 = num7;
				}
				else
				{
					num7 = -1650604200;
					num8 = num7;
				}
				num = num7 ^ (int)(num2 * 1495653189);
				continue;
			}
			case 15u:
				array = new int[result.Order.Length + 1];
				Wrapper9((Array)result.Order, (Array)array, result.Order.Length);
				num = ((int)num2 * -1188023048) ^ 0x154F87D7;
				continue;
			case 6u:
				array[^1] = start;
				num = ((int)num2 * -1110598030) ^ 0x60E50306;
				continue;
			case 4u:
				goto IL_02a1;
			case 3u:
				return new Result(array, result.Length, result.EstimatedGapPercent, result.HitTimeLimit);
			case 2u:
			{
				int num3;
				int num4;
				if (end.Value < count)
				{
					num3 = -1536758647;
					num4 = num3;
				}
				else
				{
					num3 = -683364904;
					num4 = num3;
				}
				num = num3 ^ ((int)num2 * -1937413802);
				continue;
			}
			default:
				{
					return result;
				}
				IL_01b5:
				flag = end.HasValue && end.Value != start;
				freeEnd = !end.HasValue && options.FreeEnd;
				result = ((count > options.ExactThreshold) ? new Engine(points, start, end, flag, freeEnd, options, cancellationToken).Run() : SolveExact(points, start, flag ? end : ((int?)null), freeEnd));
				if (num9 != 0)
				{
					num = -162830034;
					continue;
				}
				goto default;
			}
			break;
			IL_02a1:
			int num12;
			if (options != null)
			{
				num = -463617433;
				num12 = num;
			}
			else
			{
				num = -200332323;
				num12 = num;
			}
			continue;
			IL_00f6:
			int num13;
			if (start < 0)
			{
				num = -1107005594;
				num13 = num;
			}
			else
			{
				num = -743420061;
				num13 = num;
			}
			continue;
			IL_00b1:
			int num14;
			if (!end.HasValue)
			{
				num = -597239867;
				num14 = num;
			}
			else
			{
				num = -1491354783;
				num14 = num;
			}
		}
		goto IL_0006;
		IL_008a:
		count = points.Count;
		num = -1796845431;
		goto IL_000b;
	}

	public static Task<Result> SolveAsync(IReadOnlyList<Vector3> points, int start, int? end = null, Options? options = null, CancellationToken cancellationToken = default(CancellationToken))
	{
		CancellationToken cancellationToken2 = default;
		while (true)
		{
			int num = 748274861;
			while (true)
			{
				uint num2;
				switch ((num2 = (uint)(num ^ 0x568164C7)) % 3)
				{
				case 2u:
					break;
				case 1u:
					goto IL_0044;
				default:
					return Task.Run(() => Solve(points, start, end, options, cancellationToken2), cancellationToken2);
				}
				break;
				IL_0044:
				cancellationToken2 = cancellationToken;
				num = (int)(num2 * 536204735) ^ -1485775140;
			}
		}
	}

	private static Result SolveExact(IReadOnlyList<Vector3> points, int start, int? end, bool freeEnd)
	{
		Generated___c__DisplayClass4_0 obj = default;
		obj.points = points;
		int count = obj.points.Count;
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
					num17 = num18 + Generated__SolveExact_g__D_4_0(num20, num16, ref obj);
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
					num12 = array2[num13, num4] + (freeEnd ? 0.0 : Generated__SolveExact_g__D_4_0(num4, start, ref obj));
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
	}

	[CompilerGenerated]
	private static double Generated__SolveExact_g__D_4_0(int a, int b, ref Generated___c__DisplayClass4_0 P_2)
	{
		return Vector3.Distance(P_2.points[a], P_2.points[b]);
	}

	static ArgumentNullException Wrapper4(string P_0)
	{
		return new ArgumentNullException(P_0);
	}

	static ArgumentException Wrapper7(string P_0, string P_1)
	{
		return new ArgumentException(P_0, P_1);
	}

	static ArgumentOutOfRangeException Wrapper11(string P_0)
	{
		return new ArgumentOutOfRangeException(P_0);
	}

	static void Wrapper9(Array P_0, Array P_1, int P_2)
	{
		Array.Copy(P_0, P_1, P_2);
	}
}
